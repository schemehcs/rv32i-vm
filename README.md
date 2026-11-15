# rv32i-vm

A small RISC-V RV32I interpreter written in Rust. It loads a flat binary at
address `0`, executes instructions until `EBREAK`, and prints the final integer
registers.

> This is a learning project and work in progress, not a complete system
> emulator. In particular, it does not load ELF files or provide an operating
> system or system calls.

## Requirements

- Rust and Cargo (Rust edition 2024).

To rebuild the sample program in `testprog/`, you will also need the
`riscv64-unknown-elf` GCC and binutils tools, plus `just`.

## Build and test

Run these commands from the repository root:

```sh
cargo build
cargo test
```

## Run a program

Pass the path to a **raw binary**, not an ELF file:

```sh
cargo run -- testprog/program.bin
```

The VM copies the binary to guest address `0`, starts execution at address `0`,
and prints the 32 integer registers after the program halts. The binary must
fit in the VM's 2 MiB of memory and must halt with the `EBREAK` instruction;
there is currently no instruction limit or command-line option for memory
size.

The included sample program is a freestanding RV32I test program. Its C source,
startup assembly, and linker script are in `testprog/`. To rebuild and run it
with the optional toolchain installed:

```sh
just test-bin
```

The sample checks arithmetic, logical operations, shifts, comparisons, memory
accesses, and function calls. A successful run reaches `EBREAK`; the program
does not print a separate pass/fail message.

For command-line usage:

```sh
cargo run -- --help
```

## Supported instructions and behavior

The interpreter decodes, encodes, and executes the RV32I integer arithmetic,
logical, shift, comparison, load/store, branch, jump, and upper-immediate
instructions. It also decodes `FENCE` (as a no-op) and `EBREAK` (to halt).
Instructions are fetched and data is stored in little-endian order.

`ECALL` is decoded but not implemented and returns an error during execution.
Misaligned data accesses are not checked. The binary is loaded as-is at address
`0`; there is no ELF parsing, segment loading, or entry-point initialization.

## License

No license has been specified yet.
