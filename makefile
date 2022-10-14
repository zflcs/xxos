TARGET := riscv64gc-unknown-none-elf
MODE := debug
KERNEL_ELF := ./target/$(TARGET)/$(MODE)/kernel
FS_IMG := ./target/$(TARGET)/$(MODE)/fs.img
BOOTLOADER := rustsbi-qemu.bin


debug:
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_ELF),addr=0x80200000 -drive file=$(FS_IMG),if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d