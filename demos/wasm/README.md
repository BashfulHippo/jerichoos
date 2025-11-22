# JerichoOS WASM Demos - Canonical Test Suite

This directory contains WebAssembly demo modules that serve as the canonical test suite for JerichoOS. Every demo that passes on x86-64 becomes a "must pass" requirement for ARM64 validation.

## Purpose

These demos validate:
- ✅ WASM runtime integration (wasmi)
- ✅ Host function calls (print, syscall)
- ✅ Capability system integration
- ✅ Security isolation between modules
- ✅ Cross-architecture compatibility (x86-64 → ARM64)

## Demos

### 01_add.wat - Pure Computation
**Tests:** Basic WASM execution, parameters, recursion

Functions:
- `add(a, b)` - Adds two numbers
- `mul(a, b)` - Multiplies two numbers
- `factorial(n)` - Computes factorial recursively

**Expected Results:**
```
add(2, 3) = 5
mul(7, 6) = 42
factorial(5) = 120
```

### 02_hello.wat - Host Function Calls
**Tests:** Host imports, function boundary crossing

Functions:
- `main()` - Prints 42, 100, 255 via host
- `print_range(start, end)` - Prints range of numbers

**Expected Results:**
```
[WASM] Print called: 42
[WASM] Print called: 100
[WASM] Print called: 255
```

### 03_syscall.wat - Capability System
**Tests:** Syscall bridge, capability validation, security

Functions:
- `test_syscall()` - Valid syscall (should succeed)
- `test_allocate(size)` - Allocate memory (requires capability)
- `test_unauthorized()` - Unauthorized access (should fail)

**Expected Results:**
```
[WASM] Syscall 1 (1, 4096, 10) - SUCCESS
[WASM] Syscall 2 (1024, 0, 0) - SUCCESS (has capability)
[WASM] Syscall 0 (99, 8192, 100) - DENIED (no capability)
```

## Building

### Prerequisites
```bash
sudo apt-get install wabt  # WebAssembly Binary Toolkit
```

### Compile WAT to WASM
```bash
chmod +x compile.sh
./compile.sh
```

This generates `.wasm` binaries from `.wat` text files.

## Running in JerichoOS

The compiled `.wasm` files are embedded in the kernel and executed during boot:

```rust
// In src/main.rs (x86-64)
fn test_wasm_execution() {
    // Demo 1: Pure computation
    let wasm_bytes = include_bytes!("../demos/wasm/01_add.wasm");
    let mut module = WasmModule::from_bytes(wasm_bytes).unwrap();

    let result = module.call_function("add", &[Value::I32(2), Value::I32(3)]).unwrap();
    assert_eq!(result, Some(Value::I32(5)));

    serial_println!("[DEMO 1] ✅ PASS - add(2, 3) = 5");
}
```

## Integration Testing

Each demo includes:
1. **Compilation test** - WAT → WASM conversion
2. **Loading test** - Parse and validate WASM binary
3. **Execution test** - Call functions and verify results
4. **Capability test** - Verify security isolation

## ARM64 Validation

Once all demos pass on x86-64:
1. Copy compiled `.wasm` files to ARM64 build
2. Run same test suite on ARM64
3. Verify identical results
4. **Pass criteria:** 100% parity with x86-64

## Adding New Demos

1. Write `.wat` file in WebAssembly text format
2. Document purpose and expected results
3. Add to `compile.sh`
4. Create corresponding Rust test in `src/main.rs`
5. Verify on x86-64 first
6. Cross-validate on ARM64

## Binary Size

| Demo | WAT (text) | WASM (binary) | Compression |
|------|-----------|---------------|-------------|
| 01_add.wat | ~1.2 KB | ~100 bytes | 92% |
| 02_hello.wat | ~1.5 KB | ~150 bytes | 90% |
| 03_syscall.wat | ~2.0 KB | ~200 bytes | 90% |

WASM's binary format is extremely compact!

## Resources

- [WebAssembly Spec](https://webassembly.github.io/spec/core/)
- [wasmi Documentation](https://docs.rs/wasmi/)
- [WAT Reference](https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format)
