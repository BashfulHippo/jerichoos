# JerichoOS

a microkernel with capability-based security and webassembly runtime

## what is this

basically i wanted to see if i could build an OS that runs wasm code directly on hardware with actual security (not just process isolation but real unforgeable capabilities like seL4)

turns out you can, and it actually works on both x86-64 and arm64

## why would you do this

mostly learning - wanted to understand:
- how microkernels actually work
- capability security (way cooler than unix permissions)
- wasm outside the browser
- bare metal rust development
- dual platform support

also edge computing kinda needs something like this - current solutions are either too heavy (docker) or too unsafe (traditional rtos)

## does it work

yeah. boots in ~100ms, runs 5 different wasm demos including an mqtt broker, syscalls are faster than linux apparently

see the benchmark results below or just run `./demo_x86.sh`

## features

- capability tokens for resource access (cant forge them, cant escalate privileges)
- wasm runtime using wasmi
- x86-64 and arm64 support
- preemptive scheduler
- ipc messaging
- ~5mb kernel size

## benchmarks

tested on x86-64:

| metric | result |
|--------|--------|
| syscall latency | 94 ns |
| ipc throughput | 11.9M msg/sec |
| boot time | ~100ms |
| kernel size | ~5 MB |

both platforms pass all 5 demo tests

## quick start

need: rust nightly, qemu

```bash
# x86
./demo_x86.sh

# arm64
./build_arm64.sh
./run_arm64.sh
```

## architecture

```
wasm modules (untrusted code)
    ↓
wasmi runtime
    ↓
capability layer ← can only access what you have tokens for
    ↓
syscall interface
    ↓
microkernel (scheduler, ipc, memory)
    ↓
hardware (x86-64 or arm64)
```

## demos

1. pure computation - factorial calc in wasm
2. host functions - wasm calling kernel functions
3. syscalls - wasm using capability-protected syscalls
4. mqtt broker - pub/sub messaging between wasm modules
5. security - trying to access resources without proper caps (fails correctly)

## how it works

capabilities are unforgeable tokens that grant specific rights (read/write/execute). you literally cannot access memory or ipc endpoints without the right token. even if theres a bug in your wasm code it cant escape the sandbox

scheduler does preemptive multitasking, context switches measured at under 1us on x86-64

wasm integration was tricky - had to make wasmi work in no_std environment and bridge it to the capability system

## building

```bash
# install rust nightly
rustup toolchain install nightly
rustup default nightly

# install qemu
sudo apt-get install qemu-system-x86 qemu-system-aarch64  # ubuntu/debian
# brew install qemu  # macos

# build and run
cargo build --release
./demo_x86.sh
```

## status

| feature | x86-64 | arm64 |
|---------|--------|-------|
| boot | ✓ | ✓ |
| interrupts | ✓ | ✓ |
| heap allocator | ✓ | ✓ |
| scheduler | ✓ | ✓ |
| wasm runtime | ✓ | ✓ |
| capabilities | ✓ | ✓ |
| ipc | ✓ | ✓ |
| demos | 5/5 | 5/5 |

## known issues

- mmu disabled on arm64 (causes perf issues, need to debug)
- some demos have manual verification on arm64 (script pattern matching needs fix)
- could optimize context switch more

## what i learned

- seL4 papers are dense but worth reading
- arm64 and x86-64 are more different than expected (especially exceptions)
- wasmi in no_std takes some work but definitely possible
- github actions with arm64 runners is pretty cool
- bootloaders are surprisingly complex
- capability systems are actually way simpler than traditional access control once you get them

## files

- `src/` - kernel source (rust)
- `demos/wasm/` - wasm demo modules
- `docs/` - technical docs
- `demo_x86.sh` / `demo_arm64.sh` - run all demos
- `bench_x86.sh` / `bench_arm64.sh` - run benchmarks

## references

- seL4 whitepaper for capability design
- osdev wiki for hardware specs
- phil-opp's blog for rust os dev basics
- wasmi docs for wasm integration

## license

MIT
