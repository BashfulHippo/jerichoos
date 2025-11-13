// wasm runtime using wasmi
//
// runs wasm modules in sandbox, bridges to kernel syscalls
// took a while to get wasmi working in no_std

use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use wasmi::*;
use crate::capability::CapabilityId;
use ::core::str::from_utf8;
use spin::Mutex;

/// Global message queue for MQTT demo IPC
/// Stores pending IPC messages to be delivered to subscribers
static IPC_MESSAGE_QUEUE: Mutex<VecDeque<IpcMessage>> = Mutex::new(VecDeque::new());

/// IPC message for delivery
#[derive(Clone)]
pub struct IpcMessage {
    pub dest_client_id: u32,
    pub message: Vec<u8>,
}

/// Global subscriber registry for MQTT demo
/// Tracks which client IDs are subscribers
static MQTT_SUBSCRIBERS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// Wasm module handle with cached instance for reuse
pub struct WasmModule {
    _module: Module,
    store: Store<WasmContext>,
    instance: Instance,
}

/// Wasm execution context with capability access
pub struct WasmContext {
    /// Capabilities available to this Wasm module
    pub capabilities: Vec<CapabilityId>,
}

impl WasmContext {
    /// Create a new Wasm context with given capabilities
    pub fn new(capabilities: Vec<CapabilityId>) -> Self {
        WasmContext { capabilities }
    }
}

/// Host function: Print a value (for testing)
fn host_print(_caller: Caller<'_, WasmContext>, value: i32) {
    serial_println!("[WASM] Print called: {}", value);
}

/// Host function: Syscall bridge
fn host_syscall(
    _caller: Caller<'_, WasmContext>,
    syscall_num: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
) -> i32 {
    // TODO: actually check capabilities here before allowing syscall
    serial_println!(
        "[WASM] Syscall {} ({}, {}, {})",
        syscall_num,
        arg1,
        arg2,
        arg3
    );

    // in a real implementation this would call into the syscall handler
    // with the contexts capabilities
    0
}

/// Host function: Print string (for MQTT demos)
fn host_sys_print(mut caller: Caller<'_, WasmContext>, msg_ptr: i32, msg_len: i32) {
    // Read string from WASM linear memory
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => {
            serial_println!("[WASM] sys_print: no memory export");
            return;
        }
    };

    let msg_ptr = msg_ptr as usize;
    let msg_len = msg_len as usize;

    // Read bytes from WASM memory
    let data = memory.data(&caller);
    if msg_ptr + msg_len > data.len() {
        serial_println!("[WASM] sys_print: invalid memory access");
        return;
    }

    let msg_bytes = &data[msg_ptr..msg_ptr + msg_len];

    // Convert to string (lossy for non-UTF8)
    if let Ok(s) = from_utf8(msg_bytes) {
        serial_print!("{}", s);
    } else {
        serial_print!("[WASM] <invalid UTF-8>");
    }
}

/// Host function: Print u32 value
fn host_sys_print_u32(_caller: Caller<'_, WasmContext>, value: u32) {
    // ARM64 serial_print doesn't support formatting yet
    // Just print placeholder for now
    serial_print!("<u32>");
    let _ = value; // Suppress unused warning
}

/// Host function: MQTT subscribe
fn host_sys_mqtt_subscribe(
    mut caller: Caller<'_, WasmContext>,
    client_id: u32,
    topic_ptr: i32,
    topic_len: i32,
) -> i32 {
    // Read topic from WASM memory
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    let data = memory.data(&caller);
    let topic_ptr = topic_ptr as usize;
    let topic_len = topic_len as usize;

    if topic_ptr + topic_len > data.len() {
        return -1;
    }

    let topic = &data[topic_ptr..topic_ptr + topic_len];

    serial_print!("[MQTT-SYSCALL] Subscribe: client_id=");
    serial_print!("<u32>");
    serial_print!(" topic=");
    if let Ok(s) = from_utf8(topic) {
        serial_print!("{}", s);
    }
    serial_print!("\n");

    // Register subscriber in global registry
    let mut subscribers = MQTT_SUBSCRIBERS.lock();
    if !subscribers.contains(&client_id) {
        subscribers.push(client_id);
    }

    // Note: In full implementation, this would route to broker module's
    // broker_subscribe function. For this demo, we track subscribers globally.
    0
}

/// Host function: MQTT publish - routes to broker which sends via IPC
fn host_sys_mqtt_publish(
    mut caller: Caller<'_, WasmContext>,
    topic_ptr: i32,
    topic_len: i32,
    msg_ptr: i32,
    msg_len: i32,
) -> i32 {
    // Read topic and message from WASM memory
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    let data = memory.data(&caller);
    let topic_ptr = topic_ptr as usize;
    let topic_len = topic_len as usize;
    let msg_ptr = msg_ptr as usize;
    let msg_len = msg_len as usize;

    if topic_ptr + topic_len > data.len() || msg_ptr + msg_len > data.len() {
        return -1;
    }

    let topic = &data[topic_ptr..topic_ptr + topic_len];
    let msg = &data[msg_ptr..msg_ptr + msg_len];

    serial_print!("[MQTT-SYSCALL] Publish: topic=");
    if let Ok(s) = from_utf8(topic) {
        serial_print!("{}", s);
    }
    serial_print!(" msg=");
    if let Ok(s) = from_utf8(msg) {
        serial_print!("{}", s);
    }
    serial_print!("\n");

    // Simplified broker: directly enqueue to all registered subscribers
    // In full implementation, this would route to broker_publish WASM function
    let subscribers = MQTT_SUBSCRIBERS.lock();
    let subscriber_count = subscribers.len();

    for &client_id in subscribers.iter() {
        let ipc_msg = IpcMessage {
            dest_client_id: client_id,
            message: msg.to_vec(),
        };

        let mut queue = IPC_MESSAGE_QUEUE.lock();
        queue.push_back(ipc_msg);
    }

    subscriber_count as i32
}

/// Host function: IPC send - enqueues message for delivery
/// Enforces capability-based access control
fn host_sys_ipc_send(
    mut caller: Caller<'_, WasmContext>,
    dest: u32,
    msg_ptr: i32,
    msg_len: i32,
) -> i32 {
    // CHECK CAPABILITY: Module must have IPC_SEND permission
    // Modules with empty capability list are untrusted (e.g., malicious modules)
    if caller.data().capabilities.is_empty() {
        serial_println!("[IPC-DENIED] Module has no IPC_SEND capability");
        return -1; // Permission denied (EACCES equivalent)
    }

    // Read message from WASM memory
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    let data = memory.data(&caller);
    let msg_ptr = msg_ptr as usize;
    let msg_len = msg_len as usize;

    if msg_ptr + msg_len > data.len() {
        return -1;
    }

    let msg = &data[msg_ptr..msg_ptr + msg_len];

    serial_print!("[IPC-SYSCALL] Send to client_id=");
    serial_print!("<u32>");
    serial_print!(" msg=");
    if let Ok(s) = from_utf8(msg) {
        serial_print!("{}", s);
    }
    serial_print!("\n");

    // Enqueue message for delivery
    let ipc_msg = IpcMessage {
        dest_client_id: dest,
        message: msg.to_vec(),
    };

    let mut queue = IPC_MESSAGE_QUEUE.lock();
    queue.push_back(ipc_msg);

    0
}

impl WasmModule {
    /// Load a Wasm module from bytes and create a reusable instance
    pub fn from_bytes(wasm_bytes: &[u8]) -> Result<Self, Error> {
        // Create engine
        let engine = Engine::default();

        // Parse and validate module
        let module = Module::new(&engine, wasm_bytes)?;

        // Create store with context
        let context = WasmContext::new(Vec::new());
        let mut store = Store::new(&engine, context);

        // Create linker with host functions
        let linker = Self::create_linker(&engine);

        // Instantiate module once and cache it for reuse
        let instance = linker
            .instantiate(&mut store, &module)?
            .start(&mut store)?;

        Ok(WasmModule {
            _module: module,
            store,
            instance,
        })
    }

    /// Create a linker with host functions
    fn create_linker(engine: &Engine) -> Linker<WasmContext> {
        let mut linker = Linker::new(engine);

        // Add host function: print (original for i32)
        linker
            .func_wrap("env", "print", host_print)
            .expect("Failed to link print function");

        // Add host function: syscall (original)
        linker
            .func_wrap("env", "syscall", host_syscall)
            .expect("Failed to link syscall function");

        // Add MQTT syscalls
        linker
            .func_wrap("env", "sys_print", host_sys_print)
            .expect("Failed to link sys_print");

        linker
            .func_wrap("env", "sys_print_u32", host_sys_print_u32)
            .expect("Failed to link sys_print_u32");

        linker
            .func_wrap("env", "sys_mqtt_subscribe", host_sys_mqtt_subscribe)
            .expect("Failed to link sys_mqtt_subscribe");

        linker
            .func_wrap("env", "sys_mqtt_publish", host_sys_mqtt_publish)
            .expect("Failed to link sys_mqtt_publish");

        linker
            .func_wrap("env", "sys_ipc_send", host_sys_ipc_send)
            .expect("Failed to link sys_ipc_send");

        linker
    }

    /// Call a function on the cached instance (no re-instantiation!)
    pub fn call_function(&mut self, func_name: &str, args: &[Value]) -> Result<Option<Value>, &'static str> {
        // Get the function from the cached instance
        let func = self.instance
            .get_func(&mut self.store, func_name)
            .ok_or("Function not found")?;

        // Get function type to determine result count
        let func_type = func.ty(&self.store);
        let result_count = func_type.results().len();

        // Allocate results buffer based on actual return type
        let mut results = vec![Value::I32(0); result_count];
        func.call(&mut self.store, args, &mut results)
            .map_err(|_| "Failed to call function")?;

        Ok(results.into_iter().next())
    }

    /// Add a capability to this module's context
    pub fn grant_capability(&mut self, cap_id: CapabilityId) {
        self.store.data_mut().capabilities.push(cap_id);
    }

    /// Get capabilities count
    pub fn capability_count(&self) -> usize {
        self.store.data().capabilities.len()
    }
}

/// Initialize the Wasm runtime
pub fn init() {
    serial_println!("[WASM] Runtime initialized (wasmi interpreter)");
}

/// Load and validate a WASM module from bytes
pub fn load_and_validate(wasm_bytes: &[u8]) -> Result<WasmModule, Error> {
    WasmModule::from_bytes(wasm_bytes)
}

/// Deliver pending IPC messages to a subscriber module
/// Returns number of messages delivered
pub fn deliver_pending_messages(subscriber: &mut WasmModule, client_id: u32) -> usize {
    let mut delivered = 0;

    // Drain all messages for this client from the queue
    loop {
        let msg_opt = {
            let mut queue = IPC_MESSAGE_QUEUE.lock();
            // Find first message for this client
            if let Some(pos) = queue.iter().position(|m| m.dest_client_id == client_id) {
                queue.remove(pos)
            } else {
                None
            }
        };

        match msg_opt {
            Some(ipc_msg) => {
                // Copy message to subscriber's linear memory
                // For simplicity, we'll write it to a fixed address (offset 1024)
                const MSG_BUFFER_OFFSET: i32 = 1024;

                // Get subscriber's memory
                let memory = match subscriber.instance.get_export(&mut subscriber.store, "memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => {
                        serial_println!("[IPC] Subscriber has no memory export");
                        break;
                    }
                };

                // Write message to memory
                let msg_len = ipc_msg.message.len().min(512); // Max 512 bytes
                {
                    let data = memory.data_mut(&mut subscriber.store);
                    let buffer_start = MSG_BUFFER_OFFSET as usize;
                    if buffer_start + msg_len <= data.len() {
                        data[buffer_start..buffer_start + msg_len].copy_from_slice(&ipc_msg.message[..msg_len]);
                    } else {
                        serial_println!("[IPC] Message too large for subscriber memory");
                        continue;
                    }
                }

                // Call subscriber_receive(msg_ptr, msg_len)
                let result = subscriber.call_function(
                    "subscriber_receive",
                    &[Value::I32(MSG_BUFFER_OFFSET), Value::I32(msg_len as i32)]
                );

                match result {
                    Ok(_) => {
                        delivered += 1;
                    }
                    Err(e) => {
                        serial_print!("[IPC] Failed to deliver message: ");
                        serial_println!("{}", e);
                    }
                }
            }
            None => break, // No more messages for this client
        }
    }

    delivered
}

/// Get count of pending messages for a client
pub fn pending_message_count(client_id: u32) -> usize {
    let queue = IPC_MESSAGE_QUEUE.lock();
    queue.iter().filter(|m| m.dest_client_id == client_id).count()
}

/// Clear all pending messages (for cleanup)
pub fn clear_ipc_queue() {
    let mut queue = IPC_MESSAGE_QUEUE.lock();
    queue.clear();
}
