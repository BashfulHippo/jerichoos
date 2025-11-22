# JerichoOS WASM Demos Makefile
#
# Compiles WebAssembly Text (WAT) files to binary WASM format
#
# Usage:
#   make          - Compile all demos
#   make clean    - Remove compiled binaries
#   make check    - Verify wabt tools are installed

.PHONY: all clean check

# WASM files to generate
WASM_FILES = 01_add.wasm 02_hello.wasm 03_syscall.wasm

all: check $(WASM_FILES)
	@echo "✅ All WASM demos compiled!"
	@ls -lh *.wasm

# Pattern rule: .wat -> .wasm
%.wasm: %.wat
	@echo "Compiling $<..."
	@wat2wasm $< -o $@

check:
	@which wat2wasm > /dev/null || (echo "❌ wat2wasm not found. Install: sudo apt-get install wabt" && exit 1)
	@echo "✓ WABT tools available"

clean:
	@rm -f *.wasm
	@echo "Cleaned compiled WASM files"

# Show file sizes
sizes: all
	@echo ""
	@echo "Binary sizes:"
	@for f in $(WASM_FILES); do \
		size=$$(stat -c%s $$f 2>/dev/null || stat -f%z $$f 2>/dev/null); \
		echo "  $$f: $$size bytes"; \
	done
