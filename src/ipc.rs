//! Inter-Process Communication (IPC) for JerichoOS
//!
//! Provides capability-based message passing between tasks

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;
use crate::capability::{CapabilityId, Rights};
use crate::task::TaskId;

/// Maximum message size in bytes
pub const MAX_MESSAGE_SIZE: usize = 4096;

/// IPC Message
#[derive(Debug, Clone)]
pub struct Message {
    /// Sender task ID
    pub sender: TaskId,

    /// Message data (up to MAX_MESSAGE_SIZE)
    pub data: Vec<u8>,

    /// Optional capability being transferred
    pub transferred_cap: Option<CapabilityId>,
}

impl Message {
    /// Create a new message
    pub fn new(sender: TaskId, data: Vec<u8>) -> Result<Self, IpcError> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(IpcError::MessageTooLarge);
        }

        Ok(Message {
            sender,
            data,
            transferred_cap: None,
        })
    }

    /// Create a message with capability transfer
    pub fn with_capability(
        sender: TaskId,
        data: Vec<u8>,
        cap: CapabilityId,
    ) -> Result<Self, IpcError> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(IpcError::MessageTooLarge);
        }

        Ok(Message {
            sender,
            data,
            transferred_cap: Some(cap),
        })
    }
}

/// IPC Endpoint - a message queue with capability-based access control
pub struct IpcEndpoint {
    /// Endpoint ID (corresponds to capability)
    id: CapabilityId,

    /// Message queue
    messages: VecDeque<Message>,

    /// Tasks waiting to receive messages
    waiting_tasks: Vec<TaskId>,

    /// Maximum queue size
    max_queue_size: usize,
}

impl IpcEndpoint {
    /// Create a new IPC endpoint
    pub fn new(id: CapabilityId) -> Self {
        IpcEndpoint {
            id,
            messages: VecDeque::new(),
            waiting_tasks: Vec::new(),
            max_queue_size: 16,  // Max 16 pending messages
        }
    }

    /// Send a message to this endpoint
    pub fn send(&mut self, message: Message) -> Result<(), IpcError> {
        if self.messages.len() >= self.max_queue_size {
            return Err(IpcError::QueueFull);
        }

        self.messages.push_back(message);

        // Verbose logging only in debug builds
        #[cfg(debug_assertions)]
        serial_println!("[IPC] Message queued to endpoint {} ({} in queue)",
            self.id.value(), self.messages.len());

        Ok(())
    }

    /// Receive a message from this endpoint (non-blocking)
    pub fn try_receive(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }

    /// Check if there are pending messages
    pub fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    /// Add a task to the waiting list
    pub fn add_waiter(&mut self, task: TaskId) {
        if !self.waiting_tasks.contains(&task) {
            self.waiting_tasks.push(task);
        }
    }

    /// Get and clear all waiting tasks
    pub fn take_waiters(&mut self) -> Vec<TaskId> {
        core::mem::take(&mut self.waiting_tasks)
    }

    /// Get endpoint ID
    pub fn id(&self) -> CapabilityId {
        self.id
    }
}

/// Global IPC endpoint registry
static IPC_REGISTRY: Mutex<Option<IpcRegistry>> = Mutex::new(None);

/// IPC Endpoint Registry
pub struct IpcRegistry {
    endpoints: Vec<IpcEndpoint>,
}

impl IpcRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        IpcRegistry {
            endpoints: Vec::new(),
        }
    }

    /// Create a new endpoint
    pub fn create_endpoint(&mut self, cap_id: CapabilityId) -> CapabilityId {
        let endpoint = IpcEndpoint::new(cap_id);
        self.endpoints.push(endpoint);

        // Verbose logging only in debug builds
        #[cfg(debug_assertions)]
        serial_println!("[IPC] Created endpoint with capability {}", cap_id.value());

        cap_id
    }

    /// Get mutable reference to endpoint
    fn get_endpoint_mut(&mut self, cap_id: CapabilityId) -> Option<&mut IpcEndpoint> {
        self.endpoints.iter_mut().find(|ep| ep.id() == cap_id)
    }

    /// Get reference to endpoint
    fn get_endpoint(&self, cap_id: CapabilityId) -> Option<&IpcEndpoint> {
        self.endpoints.iter().find(|ep| ep.id() == cap_id)
    }
}

/// IPC Error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    /// Message too large
    MessageTooLarge,

    /// Queue is full
    QueueFull,

    /// Endpoint not found
    EndpointNotFound,

    /// Permission denied
    PermissionDenied,

    /// No message available
    NoMessage,
}

/// Initialize the IPC system
pub fn init() {
    *IPC_REGISTRY.lock() = Some(IpcRegistry::new());
    serial_println!("[IPC] IPC system initialized");
}

/// Create a new IPC endpoint
pub fn create_endpoint(cap_id: CapabilityId) -> Result<CapabilityId, IpcError> {
    let mut registry = IPC_REGISTRY.lock();
    let registry = registry.as_mut().ok_or(IpcError::EndpointNotFound)?;
    Ok(registry.create_endpoint(cap_id))
}

/// Send a message through an endpoint (requires WRITE rights)
pub fn send_message(
    sender: TaskId,
    endpoint_cap: CapabilityId,
    data: Vec<u8>,
) -> Result<(), IpcError> {
    // TODO: Check sender has WRITE rights to endpoint_cap

    let message = Message::new(sender, data)?;

    let mut registry = IPC_REGISTRY.lock();
    let registry = registry.as_mut().ok_or(IpcError::EndpointNotFound)?;

    let endpoint = registry.get_endpoint_mut(endpoint_cap)
        .ok_or(IpcError::EndpointNotFound)?;

    endpoint.send(message)?;

    // Wake up any waiting tasks
    let waiters = endpoint.take_waiters();
    drop(registry);  // Drop lock before scheduler operations

    for task_id in waiters {
        crate::scheduler::SCHEDULER.lock()
            .as_mut()
            .unwrap()
            .unblock_task(task_id);
    }

    Ok(())
}

/// Receive a message from an endpoint (requires READ rights)
/// Returns None if no message available (non-blocking)
pub fn try_receive_message(
    receiver: TaskId,
    endpoint_cap: CapabilityId,
) -> Result<Option<Message>, IpcError> {
    // TODO: Check receiver has READ rights to endpoint_cap

    let mut registry = IPC_REGISTRY.lock();
    let registry = registry.as_mut().ok_or(IpcError::EndpointNotFound)?;

    let endpoint = registry.get_endpoint_mut(endpoint_cap)
        .ok_or(IpcError::EndpointNotFound)?;

    Ok(endpoint.try_receive())
}

/// Receive a message from an endpoint (blocking)
/// Blocks current task until a message arrives
pub fn receive_message_blocking(
    receiver: TaskId,
    endpoint_cap: CapabilityId,
) -> Result<Message, IpcError> {
    loop {
        // Try to receive non-blocking first
        match try_receive_message(receiver, endpoint_cap)? {
            Some(msg) => return Ok(msg),
            None => {
                // No message available, register as waiter and block
                {
                    let mut registry = IPC_REGISTRY.lock();
                    let registry = registry.as_mut().ok_or(IpcError::EndpointNotFound)?;

                    let endpoint = registry.get_endpoint_mut(endpoint_cap)
                        .ok_or(IpcError::EndpointNotFound)?;

                    endpoint.add_waiter(receiver);
                }

                // Block current task
                serial_println!("[IPC] Task {} blocking on endpoint {}",
                    receiver.value(), endpoint_cap.value());

                crate::scheduler::SCHEDULER.lock()
                    .as_mut()
                    .unwrap()
                    .block_current();

                // When we wake up, try again
            }
        }
    }
}
