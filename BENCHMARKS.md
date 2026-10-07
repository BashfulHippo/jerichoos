# Benchmarks

All numbers come from QEMU runs. Treat them as relative indicators when comparing kernel changes on the same host and QEMU version, never as hardware performance.

## What each benchmark in `src/benchmark.rs` measures

| Function | What the loop does | Status |
|---|---|---|
| `benchmark_context_switches` (x86-64) | calls `scheduler::task_yield()` N times, timed with `rdtsc` | real context-switch path; ~450 ns observed |
| `benchmark_syscall_latency` | calls the capability lookup (`id()`, `rights()`) N times | scaffold — times the lookup step only; full trap round trip is planned |
| `benchmark_ipc_throughput` | loop scaffold; `ipc::send_message` is not called yet | scaffold — IPC send/receive timing is planned |
| `estimate_memory_footprint` | returns a fixed estimate | scaffold |

All cycle counts are converted with a hard-coded 3 GHz clock (`cycles_to_ns`), which QEMU does not guarantee.

## Sizes

- Kernel heap: fixed 8 MB (`src/allocator.rs`); smaller heaps (512 KB–2 MB) failed at the MQTT subscriber demo.
- CI fails the build if the boot image or ARM64 binary exceeds 10 MB.

## How to re-run

```bash
./bench_x86.sh
./bench_arm64.sh
```

ARM64 benchmark execution works, but numeric formatting in UART output is limited, so ARM64 numbers are not reported.

## Planned

- Syscall latency: time a full user-mode trap round trip (`syscall`/`int 0x80` on x86-64, `svc` on AArch64).
- IPC: time real `ipc::send_message` + receive pairs between two tasks.
- Calibrate the TSC (CPUID leaf 0x15 or against the PIT) instead of assuming 3 GHz.
