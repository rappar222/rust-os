.PHONY: all build iso vdi run run-iso run-headless check clean help

all: build

help:
	@echo "RustOS Build System"
	@echo "==================="
	@echo "  make build         - Build kernel and create BIOS image, UEFI image, VDI, and .iso"
	@echo "  make vdi           - Build VirtualBox hard disk (rustos.vdi)"
	@echo "  make iso           - Build bootable ISO image (rustos.iso)"
	@echo "  make run           - Run RustOS in QEMU emulator (BIOS drive mode)"
	@echo "  make run-iso       - Run RustOS in QEMU emulator (CD-ROM ISO mode)"
	@echo "  make run-headless  - Run RustOS in headless mode (Serial stdio)"
	@echo "  make check         - Typecheck kernel without building disk image"
	@echo "  make clean         - Remove build artifacts and disk images"

build:
	cargo run --package runner -- --no-run

vdi: build
	@echo "VirtualBox VDI ready at rustos.vdi"

iso: build
	@echo "Bootable ISO ready at rustos.iso"

run:
	cargo run --package runner

run-iso:
	cargo run --package runner -- --iso

run-headless:
	cargo run --package runner -- --headless

check:
	cargo check --package kernel --target x86_64-unknown-none

clean:
	cargo clean
