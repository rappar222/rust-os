.PHONY: all build run run-headless check clean help

all: build

help:
	@echo "RustOS Build System"
	@echo "==================="
	@echo "  make build         - Build kernel and create bootable BIOS disk image"
	@echo "  make run           - Run RustOS in QEMU emulator (GUI + Serial)"
	@echo "  make run-headless  - Run RustOS in headless mode (Serial stdio)"
	@echo "  make check         - Typecheck kernel without building disk image"
	@echo "  make clean         - Remove build artifacts and disk images"

build:
	cargo run --package runner -- --no-run

run:
	cargo run --package runner

run-headless:
	cargo run --package runner -- --headless

check:
	cargo check --package kernel --target x86_64-unknown-none

clean:
	cargo clean
