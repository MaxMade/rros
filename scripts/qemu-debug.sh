#!/bin/bash

KERNEL=target/riscv64gc-unknown-none-elf/debug/rros

# Check if file exists
if [ ! -f "$KERNEL" ]; then
    echo "Warning: \"$KERNEL\" does not exist." 1>&2
	echo "Did you forget to run \"cargo build\"?" 1>&2
	exit 1
fi

# Start QEMU
 qemu-system-riscv64 \
   -nographic \
   -machine virt \
   -smp 4 \
   -m 2G \
   -kernel $KERNEL \
   -bios default \
   "$@"
