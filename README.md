# JerichoOS 🛡️

<div align="center">

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/BashfulHippo/jerichoos)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-nightly-orange)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64-purple)](https://github.com/BashfulHippo/jerichoos)

**A capability-based microkernel with WebAssembly sandboxing**

[Features](#features) • [Quick Start](#quick-start) • [Architecture](#architecture) • [Benchmarks](BENCHMARKS.md) • [Docs](docs/)

</div>

---

## Overview

JerichoOS is a research microkernel built in Rust (`no_std`) exploring **capability-based security** and **WebAssembly isolation** in bare-metal environments. It runs untrusted WASM modules with fine-grained access control, demonstrating how modern sandboxing can work at the kernel level.

*Built with meaningful AI assistance. See [AI Development Notes](docs/AI_DEVELOPMENT.md) for the workflow.*

### Why JerichoOS?

- **Capability Security**: Fine-grained access control for kernel services
- **Memory Safe**: Rust with `unsafe` contained to narrow, audited boundaries
- **WASM Runtime**: Run untrusted code safely using `wasmi` interpreter
- **Dual Architecture**: Unified codebase for x86-64 and ARM64
- **Research-Focused**: Designed for learning and experimentation

---

## Features

<table>
<tr>
<td>

**Security**
- Capability-based IPC
- WASM sandbox isolation
- Memory-safe kernel core
- Protected syscall interface

</td>
<td>

**Runtime**
- Preemptive scheduler
- Inter-process messaging
- Host function bridging
- Stack overflow protection

</td>
<td>

**Portability**
- x86-64 support (QEMU)
- ARM64/AArch64 (Cortex-A57)
- Exception level transitions
- Platform abstraction layer

</td>
</tr>
</table>

---

## Current Status

| Component | x86-64 | ARM64 | Notes |
|-----------|:------:|:-----:|-------|
| Kernel Boot | ✅ | ✅ | QEMU virt machine |
| WASM Runtime | ✅ | ✅ | `wasmi` 0.31 |
| Demo Suite | ✅ | ✅ | 5 demos passing |
| Capability System | ✅ | ✅ | IPC enforcement working |
| Scheduler | ✅ | ✅ | Preemptive multitasking |
| CI/CD | ✅ | ✅ | GitHub Actions |

**Latest Verification**: February 8, 2026 ... All tests passing on QEMU 6.2.0

---

## Quick Start

### Prerequisites

```bash
# Install Rust nightly toolchain
rustup toolchain install nightly
rustup component add rust-src llvm-tools-preview

# Install QEMU
# Ubuntu/Debian:
sudo apt install qemu-system-x86 qemu-system-arm

# macOS:
brew install qemu
```

### Run Demos

#### x86-64
```bash
./demo_x86.sh
```

#### ARM64
```bash
./demo_arm64.sh
```

**Expected Output:**
```
✓ Demo 1: Pure Computation        PASS
✓ Demo 2: Host Function Calls     PASS
✓ Demo 3: Syscall & Capability    PASS
✓ Demo 4: MQTT Broker Pub/Sub     PASS
✓ Demo 5: Security & Isolation    PASS

RESULT: PASS
```

---

## Architecture

```text
┌────────────────────────────────────────────┐
│      WASM Modules (Sandboxed)              │
│  ┌───────────┐  ┌──────────┐  ┌──────────┐ │
│  │ mqtt.wasm │  │ app.wasm │  │ test.wasm│ │
│  └───────────┘  └──────────┘  └──────────┘ │
└─────────────┬──────────────────────────────┘
              │
    ┌─────────▼──────────┐
    │  wasmi Interpreter │
    │  (Host Bridge)     │
    └─────────┬──────────┘
              │
    ┌─────────▼────────────┐
    │  Capability Checker  │
    │  (Access Control)    │
    └─────────┬────────────┘
              │
    ┌─────────▼────────────┐
    │  Syscall Layer       │
    │  IPC • Print • MQTT  │
    └─────────┬────────────┘
              │
    ┌─────────▼────────────┐
    │  Scheduler Core      │
    │  Memory • Interrupts │
    └─────────┬────────────┘
              │
    ┌─────────▼───────────┐
    │  Architecture Layer │
    │  x86-64 │ ARM64     │
    └─────────────────────┘
```

### Key Components

- **WASM Runtime**: Sandboxed execution environment with `wasmi` interpreter
- **Capability System**: Token-based access control for kernel resources
- **IPC Layer**: Message passing between isolated modules
- **Scheduler**: Cooperative/preemptive task switching
- **Architecture Abstraction**: Shared kernel logic across x86-64 and ARM64

---

## Demo Suite

The kernel includes five demonstration programs showcasing core features:

| Demo | Description | Key Features |
|------|-------------|--------------|
| **1. Pure Computation** | Arithmetic operations in WASM | Basic execution, stack operations |
| **2. Host Functions** | Calling kernel services | Host bridge, print syscalls |
| **3. Syscall & Capability** | Protected resource access | Capability checks, syscall dispatcher |
| **4. MQTT Pub/Sub** | Message broker simulation | IPC, multi-module coordination |
| **5. Security** | Isolation enforcement | Sandbox escapes, unauthorized IPC |

---

## Benchmarks

Context switch performance (QEMU-based measurements):

| Platform | Context Switch | Notes |
|----------|---------------|-------|
| x86-64 | ~450 ns | TSC-based timing |
| ARM64 | ~850 ns | Generic timer (100 Hz) |

⚠️ **Note**: QEMU benchmarks are indicative only. Real hardware performance will differ.

See [BENCHMARKS.md](BENCHMARKS.md) for detailed methodology and results.

---

## Repository Structure

```
jerichoos/
├── src/
│   ├── arch/
│   │   ├── aarch64/      # ARM64 boot, exceptions, MMU
│   │   └── x86_64/       # x86-64 boot, interrupts, paging
│   ├── capability.rs     # Access control tokens
│   ├── syscall.rs        # System call interface
│   ├── scheduler.rs      # Task management
│   ├── wasm_runtime.rs   # wasmi integration
│   └── demos/            # Demo orchestration
├── demos/wasm/           # WASM test modules (.wat/.wasm)
├── .github/workflows/    # CI pipelines
├── demo_x86.sh           # x86-64 test runner
├── demo_arm64.sh         # ARM64 test runner
└── docs/                 # Design docs and decision records
```

---

## Development

### Build from Source

```bash
# x86-64 kernel
./build_x86.sh

# ARM64 kernel
./build_arm64.sh
```

### Quality Gates

Before committing, ensure all checks pass:

```bash
# Build verification
cargo check --bin jericho_os --release
cargo check --bin jericho_os_arm64 --release \
    --target aarch64-jericho.json \
    -Z build-std=core,compiler_builtins,alloc

# Run test suites
./demo_x86.sh && ./demo_arm64.sh
```

---

## Known Limitations

- **ARM64 UART**: Format string support is limited; numeric values may display as `{}`
- **Memory Management**: Conservative page setup; not production-grade virtual memory
- **Warning Debt**: Some compiler warnings need cleanup
- **MQTT Demos**: Currently use prebuilt `.wasm` artifacts

See [PROJECT_STATUS.md](docs/PROJECT_STATUS.md) for comprehensive limitations and roadmap.

---

## Documentation

- [Project Status](docs/PROJECT_STATUS.md) — Current capabilities and known issues
- [Architecture Decisions](DECISIONS.md) — Design rationale and tradeoffs
- [Benchmarks](BENCHMARKS.md) — Performance measurements and methodology
- [Build Guide](docs/BUILD.md) — Detailed build instructions
- [AI Development Notes](docs/AI_DEVELOPMENT.md) — Philosophy and practice of AI-assisted development

---

## Contributing

JerichoOS is a research/learning project. Contributions are welcome for:

- Bug fixes and stability improvements
- Documentation enhancements
- Performance optimizations
- Architecture-specific improvements

Please ensure all quality gates pass before submitting PRs.

---

## License

MIT License - See [LICENSE](LICENSE) for details.

---

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) — Memory-safe systems programming
- [wasmi](https://github.com/paritytech/wasmi) — WebAssembly interpreter
- [QEMU](https://www.qemu.org/) — Hardware emulation

Special thanks to the Rust embedded and OS dev communities.

---

<div align="center">

**[⬆ Back to Top](#jerichoos-️)**

Made with 🦀 by [BashfulHippo](https://github.com/BashfulHippo)

</div>
