use crate::inst::Instruction;
use anyhow::{Ok, anyhow, bail};

pub enum RetCode {
    Cont,
    Halt,
}

pub struct Machine {
    register: [i32; 32],
    memory: Vec<u8>,
    pc: usize,
}

impl Machine {
    pub fn new(memory_size: usize) -> Self {
        Self {
            register: [0; 32],
            memory: vec![0; memory_size],
            pc: 0,
        }
    }

    pub fn load_bin(&mut self, bin: &[u8]) {
        self.memory[..bin.len()].copy_from_slice(bin);
    }

    fn rr(&self, xx: usize) -> i32 {
        self.register[xx]
    }

    fn rru(&self, xx: usize) -> u32 {
        self.register[xx] as u32
    }

    fn wr(&mut self, xx: usize, val: i32) {
        if xx == 0 {
            return;
        }
        self.register[xx] = val;
    }

    fn wru(&mut self, xx: usize, val: u32) {
        self.wr(xx, val as i32)
    }

    fn advance_pc(&mut self) {
        self.pc = (self.pc as u32).wrapping_add(4) as usize;
    }

    fn jump(&mut self, offset: i32) {
        self.pc = (self.pc as u32).wrapping_add(offset as u32) as usize;
    }

    fn effective_addr(&self, rs1: usize, imm: i32) -> usize {
        (self.rru(rs1).wrapping_add(imm as u32)) as usize
    }

    fn memory_range(&self, addr: usize, size: usize) -> anyhow::Result<std::ops::Range<usize>> {
        let end = addr
            .checked_add(size)
            .filter(|&end| end <= self.memory.len())
            .ok_or_else(|| {
                anyhow!("memory access out of bounds: address {addr:#x}, size {size}")
            })?;
        Ok(addr..end)
    }

    fn parse_inst(&self) -> anyhow::Result<Instruction> {
        let range = self.memory_range(self.pc, 4)?;
        let word = u32::from_le_bytes(self.memory[range].try_into()?);
        Instruction::try_from(word)
    }

    pub fn exec_inst(&mut self, inst: &Instruction) -> anyhow::Result<RetCode> {
        use Instruction::*;
        match inst {
            Add { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1).wrapping_add(self.rr(*rs2)));
                self.advance_pc();
            }
            Sub { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1).wrapping_sub(self.rr(*rs2)));
                self.advance_pc();
            }
            Xor { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1) ^ self.rr(*rs2));
                self.advance_pc();
            }
            Or { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1) | self.rr(*rs2));
                self.advance_pc();
            }
            And { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1) & self.rr(*rs2));
                self.advance_pc();
            }
            Sll { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1) << (self.rru(*rs2) & 0x1f));
                self.advance_pc();
            }
            Srl { rd, rs1, rs2 } => {
                self.wr(*rd, (self.rru(*rs1) >> (self.rru(*rs2) & 0x1f)) as i32);
                self.advance_pc();
            }
            Sra { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1) >> (self.rru(*rs2) & 0x1f));
                self.advance_pc();
            }
            Slt { rd, rs1, rs2 } => {
                if self.rr(*rs1) < self.rr(*rs2) {
                    self.wr(*rd, 1);
                } else {
                    self.wr(*rd, 0);
                }
                self.advance_pc();
            }
            Sltu { rd, rs1, rs2 } => {
                if self.rru(*rs1) < self.rru(*rs2) {
                    self.wr(*rd, 1);
                } else {
                    self.wr(*rd, 0);
                }
                self.advance_pc();
            }
            Addi { rd, rs1, imm } => {
                self.wr(*rd, self.rr(*rs1).wrapping_add(*imm));
                self.advance_pc();
            }
            Xori { rd, rs1, imm } => {
                self.wr(*rd, self.rr(*rs1) ^ *imm);
                self.advance_pc();
            }
            Ori { rd, rs1, imm } => {
                self.wr(*rd, self.rr(*rs1) | *imm);
                self.advance_pc();
            }
            Andi { rd, rs1, imm } => {
                self.wr(*rd, self.rr(*rs1) & *imm);
                self.advance_pc();
            }
            Slli { rd, rs1, imm } => {
                self.wr(*rd, self.rr(*rs1) << (*imm as u32 & 0x1f));
                self.advance_pc();
            }
            Srli { rd, rs1, imm } => {
                self.wr(*rd, (self.rru(*rs1) >> (*imm as u32 & 0x1f)) as i32);
                self.advance_pc();
            }
            Srai { rd, rs1, imm } => {
                self.wr(*rd, self.rr(*rs1) >> (*imm as u32 & 0x1f));
                self.advance_pc();
            }
            Slti { rd, rs1, imm } => {
                self.wr(*rd, if self.rr(*rs1) < *imm { 1 } else { 0 });
                self.advance_pc();
            }
            Sltiu { rd, rs1, imm } => {
                self.wr(*rd, if self.rru(*rs1) < *imm { 1 } else { 0 });
                self.advance_pc();
            }
            Lb { rd, rs1, imm } => {
                let addr = self.effective_addr(*rs1, *imm);
                let byte = self.memory.get(addr).copied().ok_or_else(|| {
                    anyhow!("memory access out of bounds: address {addr:#x}, size 1")
                })? as i8 as i32;
                self.wr(*rd, byte);
                self.advance_pc();
            }
            Lh { rd, rs1, imm } => {
                let addr = self.effective_addr(*rs1, *imm);
                let range = self.memory_range(addr, 2)?;
                let hw: i16 = i16::from_le_bytes(self.memory[range].try_into()?);
                self.wr(*rd, hw as i32);
                self.advance_pc();
            }
            Lw { rd, rs1, imm } => {
                let addr = self.effective_addr(*rs1, *imm);
                let range = self.memory_range(addr, 4)?;
                let wd: i32 = i32::from_le_bytes(self.memory[range].try_into()?);
                self.wr(*rd, wd);
                self.advance_pc();
            }
            Lbu { rd, rs1, imm } => {
                let addr = self.effective_addr(*rs1, *imm);
                let byte = self.memory.get(addr).copied().ok_or_else(|| {
                    anyhow!("memory access out of bounds: address {addr:#x}, size 1")
                })?;
                self.wr(*rd, byte as i32);
                self.advance_pc();
            }
            Lhu { rd, rs1, imm } => {
                let addr = self.effective_addr(*rs1, *imm);
                let range = self.memory_range(addr, 2)?;
                let hw: u16 = u16::from_le_bytes(self.memory[range].try_into()?);
                self.wr(*rd, hw as i32);
                self.advance_pc();
            }
            Sb { rs2, imm, rs1 } => {
                let addr = self.effective_addr(*rs1, *imm);
                let val = self.rr(*rs2) as u8;
                let byte = self.memory.get_mut(addr).ok_or_else(|| {
                    anyhow!("memory access out of bounds: address {addr:#x}, size 1")
                })?;
                *byte = val;
                self.advance_pc();
            }
            Sh { rs2, imm, rs1 } => {
                let addr = self.effective_addr(*rs1, *imm);
                let val = self.rr(*rs2) as u16;
                let range = self.memory_range(addr, 2)?;
                self.memory[range].copy_from_slice(&val.to_le_bytes());
                self.advance_pc();
            }
            Sw { rs2, imm, rs1 } => {
                let addr = self.effective_addr(*rs1, *imm);
                let val = self.rr(*rs2);
                let range = self.memory_range(addr, 4)?;
                self.memory[range].copy_from_slice(&val.to_le_bytes());
                self.advance_pc();
            }
            Beq { rs1, rs2, imm } => {
                if self.rr(*rs1) == self.rr(*rs2) {
                    self.jump(*imm);
                } else {
                    self.advance_pc();
                }
            }
            Bne { rs1, rs2, imm } => {
                if self.rr(*rs1) != self.rr(*rs2) {
                    self.jump(*imm);
                } else {
                    self.advance_pc();
                }
            }
            Blt { rs1, rs2, imm } => {
                if self.rr(*rs1) < self.rr(*rs2) {
                    self.jump(*imm);
                } else {
                    self.advance_pc();
                }
            }
            Bge { rs1, rs2, imm } => {
                if self.rr(*rs1) >= self.rr(*rs2) {
                    self.jump(*imm);
                } else {
                    self.advance_pc();
                }
            }
            Bltu { rs1, rs2, imm } => {
                if self.rru(*rs1) < self.rru(*rs2) {
                    self.jump(*imm);
                } else {
                    self.advance_pc();
                }
            }
            Bgeu { rs1, rs2, imm } => {
                if self.rru(*rs1) >= self.rru(*rs2) {
                    self.jump(*imm);
                } else {
                    self.advance_pc();
                }
            }
            Jal { rd, imm } => {
                self.wru(*rd, (self.pc as u32).wrapping_add(4));
                self.jump(*imm);
            }
            Jalr { rd, rs1, imm } => {
                let target = self.effective_addr(*rs1, *imm) & !1;
                self.wru(*rd, (self.pc as u32).wrapping_add(4));
                self.pc = target;
            }
            Lui { rd, imm } => {
                self.wru(*rd, *imm);
                self.advance_pc();
            }
            Auipc { rd, imm } => {
                let val = (self.pc as u32).wrapping_add(*imm);
                self.wru(*rd, val);
                self.advance_pc();
            }
            Ecall => bail!("ecall not supported"),
            Ebreak => return Ok(RetCode::Halt),
            Fence => self.advance_pc(),
        }
        Ok(RetCode::Cont)
    }

    pub fn step(&mut self) -> anyhow::Result<RetCode> {
        self.exec_inst(&self.parse_inst()?)
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            if matches!(self.step()?, RetCode::Halt) {
                return Ok(());
            }
        }
    }

    pub fn dump_reg(&self) {
        for i in 0..32 {
            if i % 4 == 0 {
                print!("{:02}: ", i);
            }
            print!("{:08x} ", self.register[i]);
            if i % 4 == 3 {
                println!();
            }
        }
    }

    pub fn debug(&self) {
        self.dump_reg();
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction::*, Machine};

    #[test]
    fn store_halfword_writes_two_bytes() {
        let mut machine = Machine::new(16);
        machine.register[1] = 4;
        machine.register[2] = 0x1234_5678;

        machine
            .exec_inst(&Sh {
                rs2: 2,
                imm: 0,
                rs1: 1,
            })
            .unwrap();

        assert_eq!(&machine.memory[4..6], &[0x78, 0x56]);
        assert_eq!(machine.pc, 4);
    }

    #[test]
    fn store_word_writes_four_bytes() {
        let mut machine = Machine::new(16);
        machine.register[1] = 4;
        machine.register[2] = 0x1234_5678;

        machine
            .exec_inst(&Sw {
                rs2: 2,
                imm: 4,
                rs1: 1,
            })
            .unwrap();

        assert_eq!(&machine.memory[8..12], &[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(machine.pc, 4);
    }

    #[test]
    fn sll_uses_only_the_low_five_shift_bits() {
        let mut machine = Machine::new(16);
        machine.register[1] = 0x8000_0001_u32 as i32;
        machine.register[2] = 0x21;
        machine
            .exec_inst(&Sll {
                rd: 3,
                rs1: 1,
                rs2: 2,
            })
            .unwrap();
        assert_eq!(machine.rru(3), 2);
    }

    #[test]
    fn srl_uses_only_the_low_five_shift_bits() {
        let mut machine = Machine::new(16);
        machine.register[1] = 0x8000_0001_u32 as i32;
        machine.register[2] = 0x21;
        machine
            .exec_inst(&Srl {
                rd: 4,
                rs1: 1,
                rs2: 2,
            })
            .unwrap();
        assert_eq!(machine.rru(4), 0x4000_0000);
    }

    #[test]
    fn sra_uses_only_the_low_five_shift_bits() {
        let mut machine = Machine::new(16);
        machine.register[1] = 0x8000_0001_u32 as i32;
        machine.register[2] = 0x21;
        machine
            .exec_inst(&Sra {
                rd: 5,
                rs1: 1,
                rs2: 2,
            })
            .unwrap();
        assert_eq!(machine.rr(5), 0xc000_0000_u32 as i32);
    }

    #[test]
    fn jalr_reads_source_before_link_write_and_clears_bit_zero() {
        let mut machine = Machine::new(16);
        machine.register[1] = 101;

        machine
            .exec_inst(&Jalr {
                rd: 1,
                rs1: 1,
                imm: 2,
            })
            .unwrap();

        assert_eq!(machine.register[1], 4);
        assert_eq!(machine.pc, 102);
    }

    #[test]
    fn memory_addresses_wrap_as_rv32_addresses() {
        let mut machine = Machine::new(8);
        machine.register[1] = -1;
        machine.register[2] = 0x5a;

        machine
            .exec_inst(&Sb {
                rs2: 2,
                imm: 1,
                rs1: 1,
            })
            .unwrap();

        assert_eq!(machine.memory[0], 0x5a);
    }

    #[test]
    fn out_of_bounds_store_returns_error_without_advancing_pc() {
        let mut machine = Machine::new(4);
        machine.register[1] = 3;
        machine.register[2] = 0x1234;

        assert!(
            machine
                .exec_inst(&Sh {
                    rs2: 2,
                    imm: 0,
                    rs1: 1
                })
                .is_err()
        );
        assert_eq!(machine.pc, 0);
    }

    #[test]
    fn out_of_bounds_load_returns_error_without_advancing_pc() {
        let mut machine = Machine::new(4);
        machine.register[1] = 3;
        assert!(
            machine
                .exec_inst(&Lw {
                    rd: 3,
                    rs1: 1,
                    imm: 0
                })
                .is_err()
        );
        assert_eq!(machine.pc, 0);
    }

    #[test]
    fn instruction_fetch_out_of_bounds_returns_error() {
        let mut machine = Machine::new(4);
        machine.pc = 4;
        assert!(machine.parse_inst().is_err());
    }

    #[test]
    fn fence_is_a_supported_no_op() {
        let mut machine = Machine::new(4);
        machine.exec_inst(&Fence).unwrap();
        assert_eq!(machine.pc, 4);
    }
}

#[cfg(test)]
mod display_tests {
    use super::Instruction::*;

    #[test]
    fn formats_riscv_assembly_operands() {
        assert_eq!(
            Add {
                rd: 1,
                rs1: 2,
                rs2: 3
            }
            .to_string(),
            "add x1, x2, x3"
        );
        assert_eq!(
            Lw {
                rd: 5,
                rs1: 6,
                imm: -12
            }
            .to_string(),
            "lw x5, -12(x6)"
        );
        assert_eq!(
            Sw {
                rs2: 7,
                imm: 16,
                rs1: 8
            }
            .to_string(),
            "sw x7, 16(x8)"
        );
        assert_eq!(
            Sltiu {
                rd: 9,
                rs1: 10,
                imm: 0xffff_f800
            }
            .to_string(),
            "sltiu x9, x10, -2048"
        );
        assert_eq!(
            Lui {
                rd: 11,
                imm: 0x1234_5000
            }
            .to_string(),
            "lui x11, 0x12345"
        );
        assert_eq!(Ecall.to_string(), "ecall");
    }

    #[test]
    fn displays_decoded_instructions() {
        let instruction = super::Instruction::try_from(0xfe1f_f0ef).unwrap();
        assert_eq!(instruction.to_string(), "jal x1, -32");
    }
}
