# TODO: Use the build script feature of cargo??

export TARGET?=x86_64-unknown-uefi

BUILD=build
ISO=$(BUILD)/iso

RUST_TARGET=target/$(TARGET)/release

FIRMWARE_FILE?=firmware/ovmf.fd

QEMU?=qemu-system-x86_64
QEMU_FLAGS=\
	-accel kvm \
	-M q35 \
	-m 1024 \
	-net none \
	-serial mon:stdio \
	-vga std \
	-bios $(FIRMWARE_FILE)

all: $(BUILD)/nex_os.iso

qemu: $(BUILD)/nex_os.iso firmware/ovmf.fd
	sudo ${QEMU} ${QEMU_FLAGS} -cdrom $<

$(BUILD)/nex_os.iso: $(ISO)/boot $(ISO)/EFI/nex_os.efi
	grub-mkrescue -o $@ $(ISO)

$(ISO)/boot: boot
	mkdir -p $(ISO)
	cp -r $< $(ISO)

$(ISO)/EFI/nex_os.efi: $(RUST_TARGET)/nex_os.efi
	mkdir -p $(ISO)/EFI
	cp $< $@

$(RUST_TARGET)/nex_os.efi: src/* Cargo.lock Cargo.toml
	cargo build --release --target $(TARGET)

clean:
	cargo clean
	rm -rf build/
