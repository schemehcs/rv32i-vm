clean-bin:
	rm testprog/program*
test-bin: gen-bin
	cargo run -- testprog/program.bin
gen-bin:
    riscv64-unknown-elf-gcc -march=rv32i -mabi=ilp32 -ffreestanding -nostdlib -O1 -Wl,--no-warn-rwx-segments -T testprog/link.ld testprog/start.s testprog/main.c -o testprog/program.elf
    riscv64-unknown-elf-objcopy -O binary testprog/program.elf testprog/program.bin
