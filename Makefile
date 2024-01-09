# TODO: Use the build script feature of cargo??

export TARGET?=x86_64-unknown-uefi

# Cargo outputs our efi file at this location
RUST_TARGET=target/$(TARGET)/release

BUILD_DIR=build

# Contains all the files that will be packaged in ISO
ISO_DIR=$(BUILD_DIR)/iso

# UEFI firmware file for virtual machines
FIRMWARE_FILE?=firmware/ovmf.fd

# Load qemu with uefi firmware file
QEMU?=qemu-system-x86_64
QEMU_FLAGS=\
	-accel kvm \
	-net none \
	-vga virtio \
	-device virtio-rng-pci \
	-bios $(FIRMWARE_FILE)

all: $(BUILD_DIR)/nex_os.iso

qemu: $(BUILD_DIR)/nex_os.iso $(FIRMWARE_FILE)
	sudo ${QEMU} ${QEMU_FLAGS} -cdrom $<

$(BUILD_DIR)/nex_os.iso: boot/grub/* boot/grub/themes/*/* $(ISO_DIR)/EFI/nex_os.efi
	mkdir -p $(ISO_DIR)
	cp -r boot -t $(ISO_DIR)
	grub-mkrescue -o $@ $(ISO_DIR)

$(ISO_DIR)/EFI/%.efi: $(RUST_TARGET)/%.efi
	mkdir -p $(ISO_DIR)/EFI
	cp $< $@

$(RUST_TARGET)/%.efi: src/* Cargo.lock Cargo.toml
	cargo build --release --target $(TARGET)

clean:
	cargo clean
	rm -rf build/
