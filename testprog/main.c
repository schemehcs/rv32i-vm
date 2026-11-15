#include <stdint.h>

// Global initialized (.data) and uninitialized (.bss) variables
volatile uint32_t g_data_check = 0x12345678;
volatile uint32_t g_bss_check;

// Helper function to test jal / jalr, stack frames, and recursion
uint32_t fibonacci(uint32_t n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main(void) {
    // 1. Validate .data and .bss memory initialization
    if (g_data_check != 0x12345678) return 1;
    g_bss_check = 0xCAFEBABE;
    if (g_bss_check != 0xCAFEBABE) return 2;

    // 2. ALU: Arithmetic & Logical (add, sub, xor, or, and)
    volatile int32_t a = 15;
    volatile int32_t b = -5;
    if ((a + b) != 10) return 3;
    if ((a - b) != 20) return 4;
    if ((a ^ 0xFF) != 0xF0) return 5;
    if ((a | 0xF0) != 0xFF) return 6;
    if ((a & 0x06) != 0x06) return 7;

    // 3. Shifts: Logical vs Arithmetic (sll, srl, sra)
    volatile uint32_t u_val = 0x80000010;
    volatile int32_t  s_val = -16; // 0xFFFFFFF0
    if ((u_val >> 4) != 0x08000001) return 8;  // Tests SRL (zero-fill)
    if ((s_val >> 2) != -4)         return 9;  // Tests SRA (sign-extend)
    if ((1u << 5) != 32)            return 10; // Tests SLL

    // 4. Signed vs Unsigned Comparisons (slt, sltu, blt, bltu, bge, bgeu)
    volatile int32_t  neg = -1;
    volatile uint32_t pos = 1;
    if (!(neg < (int32_t)pos))   return 11; // Signed: -1 < 1 is true
    if (!((uint32_t)neg > pos))  return 12; // Unsigned: 0xFFFFFFFF > 1 is true

    // 5. Sub-word Memory Access & Sign Extension (lb, lbu, lh, lhu, sb, sh, lw, sw)
    volatile uint8_t mem[4] = {0x80, 0x7F, 0xFF, 0x01};
    int8_t  signed_byte   = *(volatile int8_t *)&mem[0];
    uint8_t unsigned_byte = *(volatile uint8_t *)&mem[0];
    int16_t signed_half   = *(volatile int16_t *)&mem[2]; // 0x01FF
    if (signed_byte != -128)     return 13; // Tests LB sign-extension
    if (unsigned_byte != 128)    return 14; // Tests LBU zero-extension
    if (signed_half != 0x01FF)   return 15; // Tests LH little-endian load

    // 6. Function Calls & Stack Management (jal, jalr, sw/lw on sp)
    // fibonacci(7) = 13
    if (fibonacci(7) != 13)      return 16;

    // All tests passed! Write a magic signature to memory and return 0.
    g_bss_check = 0x0000600D;
    return 0;
}
