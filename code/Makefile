TARGET := riscv64imac-unknown-none-elf
CARGO  := cargo
CRATES := fee-type owned-type timed-lock payload-type

# ckb-std compiles a small C shim for RISC-V and, through the `cc` crate, looks for
# `riscv64-unknown-elf-gcc`. A machine without that toolchain still builds with clang,
# which targets RISC-V out of the box, so point cc at it when gcc is absent. Found the
# way these things are found: the clean-machine run failed on it.
ifeq ($(shell command -v riscv64-unknown-elf-gcc 2>/dev/null),)
export CC_riscv64imac_unknown_none_elf ?= clang
# Debian and Ubuntu ship `llvm-ar` only with the `llvm` package, and a runner image may
# have just the versioned binary (`llvm-ar-18`); take whichever exists.
export AR_riscv64imac_unknown_none_elf ?= $(shell command -v llvm-ar 2>/dev/null || ls /usr/bin/llvm-ar-* 2>/dev/null | sort -V | tail -1)
endif

.PHONY: build test clean

# Every toy script is built twice: as written (the pitfall) and with the `fixed`
# feature (the correction). The tests load one or the other by name, so a test can
# say "this transaction is accepted by the vulnerable build and refused by the fixed
# one", which is the whole point.
build:
	$(CARGO) build --release --target $(TARGET)
	mkdir -p target/vulnerable target/fixed
	for c in $(CRATES); do cp target/$(TARGET)/release/$$c target/vulnerable/$$c; done
	for c in $(CRATES); do \
	  $(CARGO) build --release --target $(TARGET) -p $$c --features fixed && \
	  cp target/$(TARGET)/release/$$c target/fixed/$$c; \
	done

# Host-target tests that run both builds inside ckb-testtool's mock VM.
test: build
	cd tests && $(CARGO) test -- --nocapture

clean:
	$(CARGO) clean
	cd tests && $(CARGO) clean
