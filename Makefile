# TODO: Use the build script feature of cargo??

BUILD_DIR=build

FIRMWARE_FILE?=firmware/ovmf.fd

QEMU?=qemu-system-x86_64
QEMU_FLAGS=\
	-cpu qemu64 \
	-accel kvm \
	-net none \
	-vga virtio \
	-serial mon:stdio

all: ${BUILD_DIR}/NexOS.img

qemu: ${FIRMWARE_FILE} ${BUILD_DIR}/NexOS.img
	sudo ${QEMU} ${QEMU_FLAGS} -bios ${FIRMWARE_FILE} -drive format=raw,file=${BUILD_DIR}/NexOS.img

${BUILD_DIR}/NexOS.img: ${BUILD_DIR}/BOOTX64.efi
	mkdir -p ${BUILD_DIR}

	# Create 48MB zeroed disk image file
	dd if=/dev/zero of=$@ bs=512 count=93750

	parted $@ -s -a minimal mklabel gpt
	parted $@ -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted $@ -s -a minimal toggle 1 boot
	
	dd if=/dev/zero of=/tmp/part.img bs=512 count=91669
	mformat -i /tmp/part.img -h 32 -t 32 -n 64 -c 1
	mmd -i /tmp/part.img ::/EFI
	mmd -i /tmp/part.img ::/EFI/BOOT
	mcopy -i /tmp/part.img $< ::/EFI/BOOT

	dd if=/tmp/part.img of=$@ bs=512 count=91669 seek=2048 conv=notrunc

${BUILD_DIR}/BOOTX64.efi: 
	mkdir -p ${BUILD_DIR}
	cargo build -p boot --release
	cp target/x86_64-unknown-uefi/release/boot.efi $@

clean:
	cargo clean
	rm -rf build/
