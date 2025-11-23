# `RROS`

*R*ust-based *R*ISC-V *O*perating *S*ystem

## Overview

`RROS` is an minimal operating system kernel written in Rust for the RISC-V architecture.

## Getting Started

To build and experiment with `RROS`:

```sh
git clone https://github.com/MaxMade/rros.git
cd rros
cargo build
```

### Prerequisites

- Rust toolchain (*nightly*)
- RISC-V assembler (e.g., GNU assembler for RISC-V)
- QEMU or compatible emulator for testing

### Usage

```sh
# Start release build in QEMU
./scripts/qemu-release.sh

# Start debug build in QEMU
./scripts/qemu-debug.sh

# Start debug build in QEMU with GDB
./scripts/qemu-gdb.sh
```

# License

This project is released under an open-source license. See the LICENSE file for details.

# Contact

For questions or collaboration, open an issue on GitHub.
