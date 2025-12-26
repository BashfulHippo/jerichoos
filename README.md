# JerichoOS

a microkernel with capability-based security and webassembly runtime

## what is this

basically i wanted to see if i could build an OS that runs wasm code directly on hardware with actual security (not just process isolation but real unforgeable capabilities like seL4)

turns out you can, and it actually works on both x86-64 and arm64

## quick start

```bash
./demo_x86.sh
```

## status

both platforms work, 5 demos passing

see DEMO_GUIDE.md for details
