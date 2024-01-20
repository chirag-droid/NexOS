# TODO: Use the build script feature of cargo??

BUILD_DIR=build

FIRMWARE_FILE?=firmware/ovmf.fd

QEMU?=qemu-system-x86_64
QEMU_FLAGS=\
	-cpu qemu64 \
	-net none \
	-vga virtio \
	-m 256M \
	-serial mon:stdio

all: ${BUILD_DIR}/NexOS.img

qemu: ${FIRMWARE_FILE} ${BUILD_DIR}/NexOS.img
	${QEMU} ${QEMU_FLAGS} -bios ${FIRMWARE_FILE} -drive format=raw,file=${BUILD_DIR}/NexOS.img

${BUILD_DIR}/NexOS.img: ${BUILD_DIR}/BOOTX64.efi ${BUILD_DIR}/NexOS/kernel
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
	mcopy -i /tmp/part.img ${BUILD_DIR}/BOOTX64.efi ::/EFI/BOOT

	mmd -i /tmp/part.img ::/NexOS
	mcopy -i /tmp/part.img ${BUILD_DIR}/NexOS/kernel ::/NexOS

	dd if=/tmp/part.img of=$@ bs=512 count=91669 seek=2048 conv=notrunc

${BUILD_DIR}/BOOTX64.efi: boot/* boot/*/*
	mkdir -p ${BUILD_DIR}
	cargo build -p boot --release
	cp target/x86_64-unknown-uefi/release/boot.efi $@

${BUILD_DIR}/NexOS/kernel: kernel/* kernel/*/* linkers/*
	mkdir -p ${BUILD_DIR}/NexOS
	cargo build -p kernel --release
	cp target/x86_64-unknown-none/release/kernel $@

clean:
	cargo clean
	rm -rf build/
