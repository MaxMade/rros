#!/bin/bash

KERNEL="target/riscv64gc-unknown-none-elf/debug/rros"

# Check if file exists
if [ ! -f "$KERNEL" ]; then
    echo "Warning: \"$KERNEL\" does not exist." 1>&2
	echo "Did you forget to run \"cargo build\"?" 1>&2
	exit 1
fi

# Prepare rust-gdb
RUSTC_SYSROOT="$(rustc --print=sysroot)"
GDB_PYTHON_MODULE_DIRECTORY="$RUSTC_SYSROOT/lib/rustlib/etc"
PYTHONPATH="$PYTHONPATH:$GDB_PYTHON_MODULE_DIRECTORY"

# Start QEMU and GDB
tmux new-session -d qemu-system-riscv64 -nographic -machine virt -smp 4 -m 2G -kernel $KERNEL -bios default -S -s
tmux split-window -h riscv64-elf-gdb --directory="$GDB_PYTHON_MODULE_DIRECTORY" -iex "add-auto-load-safe-path $GDB_PYTHON_MODULE_DIRECTORY" $KERNEL -ex "target remote :1234"
tmux attach-session -d
