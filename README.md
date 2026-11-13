# rv32i-vm

A work-in-progress interpreter for the integer instruction set of RISC-V
RV32I. The project currently contains an instruction decoder, an instruction
encoder, and a small execution engine. It is not yet a general-purpose virtual
machine: there is no program loader or command-line interface, and `main`
executes a short hard-coded instruction sequence.

## Current status

Implemented:

- Decoding RV32I integer arithmetic, shifts, comparisons, loads, stores,
  branches, jumps, and upper-immediate instructions.
- Encoding decoded instructions back into 32-bit instruction words.
- Executing those instructions against 32 integer registers and a byte-addressed
  memory.
- Little-endian instruction and data access.
- Bounds-checked memory access that reports errors instead of indexing outside
  the VM's memory.
- `FENCE` and `FENCE.TSO` decoding. `FENCE` currently acts as a no-op in the
  single-hart interpreter.

Known limitations:

- No program loader yet. The current `main` creates a 4096-byte memory and
  executes three hand-constructed instructions; it does not fetch a program
  from a file.
- No ELF parsing, segment loading, entry-point initialization, or command-line
  options.
- `ECALL` and `EBREAK` are decoded but execution returns an unsupported
  instruction error. There is no trap handling, operating-system interface,
  or syscall implementation.
- Misaligned data accesses are not currently trapped or otherwise checked.
- This is a prototype: its VM types and interfaces are private implementation
  details and are not yet exposed as a reusable library API.

## Requirements

- Rust toolchain with Cargo (the package uses Rust edition 2024).

## Build, test, and run

```sh
cargo build
cargo test
cargo run
```

`cargo run` runs the built-in example and prints the register state. It does
not accept or load a RISC-V binary. The tests cover individual instruction
decoding and encoding, along with selected execution behaviors.

## Roadmap

The next step toward a usable VM is loading a guest program into memory. A
loader will need to define how guest addresses map to VM memory and initialize
the program counter from the loaded program's entry point. ELF support is a
possible format for that loader; it is not implemented yet.

Other work needed for a more complete VM includes:

- A command-line interface for selecting a program and configuring memory.
- Trap behavior for invalid instructions, misaligned or out-of-range accesses,
  `ECALL`, and `EBREAK`.
- A defined guest environment (such as a minimal syscall interface) if running
  compiled programs is a goal.
- Broader tests, including end-to-end execution of loaded programs.

## License

No license has been specified yet.
