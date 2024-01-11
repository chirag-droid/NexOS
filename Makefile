# TODO: Use the build script feature of cargo??

BUILD_DIR=build

# Contains all the files that will be packaged in ISO
ISO_DIR=$(BUILD_DIR)/iso

# UEFI firmware file for virtual machines
FIRMWARE_FILE?=firmware/ovmf.fd

# Attach virtuo vga and redirect stdio
QEMU?=qemu-system-x86_64
QEMU_FLAGS=\
	-accel kvm \
	-net none \
	-vga virtio \
	-serial mon:stdio

all: $(BUILD_DIR)/nex_os.iso

qemu: $(BUILD_DIR)/nex_os.iso $(FIRMWARE_FILE)
	sudo $(QEMU) $(QEMU_FLAGS) -bios $(FIRMWARE_FILE) -cdrom $< 

$(BUILD_DIR)/nex_os.iso: grub/* grub/themes/*/* $(ISO_DIR)/EFI/boot.efi $(ISO_DIR)/BIN/libkernel.rlib
	mkdir -p $(ISO_DIR)/boot
	cp -r grub -t $(ISO_DIR)/boot
	grub-mkrescue -o $@ $(ISO_DIR)

$(ISO_DIR)/EFI/boot.efi: boot/* boot/*/* boot/*/*/*
	mkdir -p $(ISO_DIR)/EFI
	cargo build -p boot --release
	cp target/x86_64-unknown-uefi/release/boot.efi $@

$(ISO_DIR)/BIN/libkernel.rlib: kernel/* kernel/*/*
	mkdir -p $(ISO_DIR)/BIN
	cargo build -p kernel --release
	cp target/x86_64-unknown-none/release/libkernel.rlib $@

clean:
	cargo clean
	rm -rf build/
