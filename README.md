# JerichoOS

a minimal microkernel written in rust with capability-based security and a webassembly runtime. runs untrusted wasm modules on bare metal (no standard library).

## what it does

- capability model for access control (inspired by seL4)
- basic scheduler and message passing (ipc)
- runs wasm modules using wasmi
- supports both x86-64 and arm64

this is a learning project, not meant for production use.

## status

both x86-64 and arm64 build, boot in qemu, and run the wasm demo suite. ci workflows are set up for automated testing.

last verified feb 7, 2026 with:
- `cargo check --bin jericho_os --release`
- `cargo check --bin jericho_os_arm64 --release --target aarch64-jericho.json -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem`

## how it's layered

```text
wasm modules (sandboxed)
  -> wasmi runtime
  -> capability checks
  -> syscalls + ipc
  -> scheduler, memory, interrupts
  -> arch layer (x86-64 or arm64)
```

## running it

you'll need:
- rust nightly with `rust-src` and `llvm-tools-preview`
- qemu (`qemu-system-x86`, `qemu-system-aarch64`)
- wabt (`wat2wasm`) if rebuilding .wat demos

x86-64:
```bash
./demo_x86.sh
```

arm64:
```bash
./demo_arm64.sh
```

## benchmarks

see `BENCHMARKS.md` for numbers. keep in mind these are from qemu, not real hardware, so they're mainly useful for comparing changes within the kernel.

## repo structure

- `src/`: kernel code
- `src/arch/aarch64/`: arm64-specific bits
- `demos/wasm/`: wasm modules for the demo suite
- `demo_x86.sh`, `demo_arm64.sh`: test runners
- `bench_x86.sh`, `bench_arm64.sh`: benchmarks
- `.github/workflows/`: ci setup
- `docs/PROJECT_STATUS.md`: current status and limitations

## rebuilding demos

- `01_add.wasm`, `02_hello.wasm`, `03_syscall.wasm` can be rebuilt from `.wat` sources
- mqtt and security demos are prebuilt `.wasm` files (vendored for now)

## known issues

- arm64 uart doesn't format numbers properly yet (shows placeholders)
- arm64 memory setup is basic, not full virtual memory
- still some compiler warnings to clean up

## testing locally

```bash
cargo check --bin jericho_os --release
cargo check --bin jericho_os_arm64 --release --target aarch64-jericho.json -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem
./demo_x86.sh
./demo_arm64.sh
```
