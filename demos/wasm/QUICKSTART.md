# WASM Demos Quick Start

## Prerequisites

Install WABT (WebAssembly Binary Toolkit):

```bash
sudo apt-get install wabt
```

## Build WASM Demos

```bash
cd demos/wasm
make
```

This compiles all `.wat` (text) files to `.wasm` (binary) files.

## Expected Output

```
✓ WABT tools available
Compiling 01_add.wat...
Compiling 02_hello.wat...
Compiling 03_syscall.wat...
✅ All WASM demos compiled!
-rw-rw-r-- 1 user user  98 Dec 26 20:40 01_add.wasm
-rw-rw-r-- 1 user user 142 Dec 26 20:40 02_hello.wasm
-rw-rw-r-- 1 user user 195 Dec 26 20:40 03_syscall.wasm
```

## Test on JerichoOS x86-64

```bash
cd ../..  # Back to project root
cargo run
```

Look for the "WASM Demo Suite" section in the output:

```
╔════════════════════════════════════════════════════╗
║  JerichoOS WASM Demo Suite - Canonical Tests      ║
╚════════════════════════════════════════════════════╝

[DEMO 1] Pure Computation (01_add.wasm)
=========================================
[TEST] add(2, 3) = 5 ✅
[TEST] mul(7, 6) = 42 ✅
[TEST] factorial(5) = 120 ✅
[DEMO 1] ✅ COMPLETE

... (more demos)
```

## Troubleshooting

**Error: "No such file or directory: demos/wasm/01_add.wasm"**
- Solution: Run `make` in `demos/wasm/` first

**Error: "wat2wasm: command not found"**
- Solution: `sudo apt-get install wabt`

**Error: "Failed to load module"**
- Check that .wasm files are valid: `wasm-validate 01_add.wasm`
- Try recompiling: `make clean && make`
