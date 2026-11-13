use anyhow::{Ok, bail};

#[derive(Debug, PartialEq)]
enum Instruction {
    /// ADD rd = rs1 + rs2
    Add { rd: usize, rs1: usize, rs2: usize },
    /// SUB rd = rs1 - rs2
    Sub { rd: usize, rs1: usize, rs2: usize },
    /// XOR rd = rs1 ^ rs2
    Xor { rd: usize, rs1: usize, rs2: usize },
    /// OR rd = rs1 | rs2
    Or { rd: usize, rs1: usize, rs2: usize },
    /// AND rd = rs1 & rs2
    And { rd: usize, rs1: usize, rs2: usize },
    /// Shift Left Logical rd = rs1 << rs2
    Sll { rd: usize, rs1: usize, rs2: usize },
    /// Shift Right Logical rd = rs1 >> rs2
    Srl { rd: usize, rs1: usize, rs2: usize },
    /// Shift Right Arith* rd = rs1 >> rs2
    Sra { rd: usize, rs1: usize, rs2: usize },
    /// Set Less Than rd = (rs1 < rs2)?1:0
    Slt { rd: usize, rs1: usize, rs2: usize },
    /// Set Less Than(U) rd = (rs1 < rs2)?1:0
    Sltu { rd: usize, rs1: usize, rs2: usize },
    /// ADD Imm rd = rs1 + imm
    Addi { rd: usize, rs1: usize, imm: i32 },
    /// XOR Imm rd = rs1 ^ imm
    Xori { rd: usize, rs1: usize, imm: i32 },
    /// OR Imm rd = rs1 | imm
    Ori { rd: usize, rs1: usize, imm: i32 },
    /// AND Imm rd = rs1 & imm
    Andi { rd: usize, rs1: usize, imm: i32 },
    /// Shift Left Logical Imm rd = rs1 << imm
    Slli { rd: usize, rs1: usize, imm: i32 },
    /// Shift Right Logical Imm rd = rs1 >> imm
    Srli { rd: usize, rs1: usize, imm: i32 },
    /// Shift right Arith Imm rd = rs1 >> imm
    Srai { rd: usize, rs1: usize, imm: i32 },
    /// Set Less Than Imm rd = (rs1 < imm)?1:0
    Slti { rd: usize, rs1: usize, imm: i32 },
    /// Set Less Than Imm(U) rd = (rs1 < imm)?1:0
    Sltiu { rd: usize, rs1: usize, imm: u32 },
    /// Load Byte rd = M[rs1+imm][0:7]
    Lb { rd: usize, rs1: usize, imm: i32 },
    /// Load Half rd = M[rs1+imm][0:15]
    Lh { rd: usize, rs1: usize, imm: i32 },
    /// Load Word rd = M[rs1+imm][0:31]
    Lw { rd: usize, rs1: usize, imm: i32 },
    /// Load Byte(U) rd = M[rs1+imm][0:7]
    Lbu { rd: usize, rs1: usize, imm: i32 },
    /// Load Half(U) rd = M[rs1+imm][0:15]
    Lhu { rd: usize, rs1: usize, imm: i32 },
    /// Store Byte M[rs1+imm][0:7] = rs2[0:7]
    Sb { rs2: usize, imm: i32, rs1: usize },
    /// Store Half M[rs1+imm][0:15] = rs2[0:15]
    Sh { rs2: usize, imm: i32, rs1: usize },
    /// Store Word M[rs1+imm][0:31] = rs2[0:31]
    Sw { rs2: usize, imm: i32, rs1: usize },
    /// Branch == if(rs1==rs2) PC+=imm
    Beq { rs1: usize, rs2: usize, imm: i32 },
    /// Branch != if(rs1!=rs2) PC+=imm
    Bne { rs1: usize, rs2: usize, imm: i32 },
    /// Branch < if(rs1<rs2) PC+=imm
    Blt { rs1: usize, rs2: usize, imm: i32 },
    /// Branch > if(rs1>rs2) PC+=imm
    Bge { rs1: usize, rs2: usize, imm: i32 },
    /// Branch <(U) if(rs1<rs2) PC+=imm
    Bltu { rs1: usize, rs2: usize, imm: i32 },
    /// Branch >=(U) if(rs1>=rs2) PC+=imm
    Bgeu { rs1: usize, rs2: usize, imm: i32 },
    /// Jump and Link rd = PC+4; PC+=imm
    Jal { rd: usize, imm: i32 },
    /// Jump and Link Reg rd=PC+4; PC=rs1+imm
    Jalr { rd: usize, rs1: usize, imm: i32 },
    /// Load Upper Imm rd = imm << 12
    Lui { rd: usize, imm: u32 },
    /// Add Upper Imm to PC rd=PC+(imm<<12)
    Auipc { rd: usize, imm: u32 },
    /// Env Call Transfer control to OS
    Ecall,
    /// Env Break Transfer control to debugger
    Ebreak,
}

impl TryFrom<u32> for Instruction {
    type Error = anyhow::Error;
    fn try_from(value: u32) -> anyhow::Result<Self> {
        use Instruction::*;
        let opcode = value & 0x7f;
        match opcode {
            // Arith - R FMT
            0x33 => {
                let funct3 = (value >> 12) & 0x7;
                let funct7 = value >> 25;
                let rd = ((value >> 7) & 0x1f) as usize;
                let rs1 = ((value >> 15) & 0x1f) as usize;
                let rs2 = ((value >> 20) & 0x1f) as usize;
                match (funct3, funct7) {
                    (0x0, 0x00) => Ok(Add { rd, rs1, rs2 }),
                    (0x0, 0x20) => Ok(Sub { rd, rs1, rs2 }),
                    (0x4, 0x00) => Ok(Xor { rd, rs1, rs2 }),
                    (0x6, 0x00) => Ok(Or { rd, rs1, rs2 }),
                    (0x7, 0x00) => Ok(And { rd, rs1, rs2 }),
                    (0x1, 0x00) => Ok(Sll { rd, rs1, rs2 }),
                    (0x5, 0x00) => Ok(Srl { rd, rs1, rs2 }),
                    (0x5, 0x20) => Ok(Sra { rd, rs1, rs2 }),
                    (0x2, 0x00) => Ok(Slt { rd, rs1, rs2 }),
                    (0x3, 0x00) => Ok(Sltu { rd, rs1, rs2 }),
                    _ => bail!(
                        "invalid (funct3, funct7):({:x},{:x}), opcode:{:b}",
                        funct3,
                        funct7,
                        opcode
                    ),
                }
            }
            // Airth Imm - I FMT
            0x13 => {
                let funct3 = (value >> 12) & 0x7;
                let rd = ((value >> 7) & 0x1f) as usize;
                let rs1 = ((value >> 15) & 0x1f) as usize;
                let imm = (value as i32) >> 20;
                match funct3 {
                    0x0 => Ok(Addi { rd, rs1, imm }),
                    0x4 => Ok(Xori { rd, rs1, imm }),
                    0x6 => Ok(Ori { rd, rs1, imm }),
                    0x7 => Ok(Andi { rd, rs1, imm }),
                    0x1 => {
                        if imm >> 5 != 0x00 {
                            bail!("invalid shift left logical imm:{:x}", imm)
                        }
                        Ok(Slli { rd, rs1, imm })
                    }
                    0x5 => match imm >> 5 {
                        0x00 => Ok(Srli { rd, rs1, imm }),
                        0x20 => Ok(Srai {
                            rd,
                            rs1,
                            imm: imm & 0x1f,
                        }),
                        _ => bail!("invalid shift imm:{:x}", imm),
                    },
                    0x2 => Ok(Slti { rd, rs1, imm }),
                    0x3 => Ok(Sltiu {
                        rd,
                        rs1,
                        imm: imm as u32,
                    }),
                    _ => bail!("invalid funct3:{:x}", funct3),
                }
            }
            // Load - I FMT
            0x03 => {
                let funct3 = (value >> 12) & 0x7;
                let rd = ((value >> 7) & 0x1f) as usize;
                let rs1 = ((value >> 15) & 0x1f) as usize;
                let imm = (value as i32) >> 20;
                match funct3 {
                    0x0 => Ok(Lb { rd, rs1, imm }),
                    0x1 => Ok(Lh { rd, rs1, imm }),
                    0x2 => Ok(Lw { rd, rs1, imm }),
                    0x4 => Ok(Lbu { rd, rs1, imm }),
                    0x5 => Ok(Lhu { rd, rs1, imm }),
                    _ => bail!("invalid load funct3:{:x}", funct3),
                }
            }
            // Store - S FMT
            0x23 => {
                let funct3 = (value >> 12) & 0x7;
                let rs2 = ((value >> 20) & 0x1f) as usize;
                let imm = ((value >> 7) & 0x1f) as i32 | ((value as i32) >> 20 & !0b11111);
                let rs1 = ((value >> 15) & 0x1f) as usize;
                match funct3 {
                    0x0 => Ok(Sb { rs2, imm, rs1 }),
                    0x1 => Ok(Sh { rs2, imm, rs1 }),
                    0x2 => Ok(Sw { rs2, imm, rs1 }),
                    _ => bail!("invalid store funct3:{:x}", funct3),
                }
            }
            // Branch - B FMT
            0x63 => {
                let funct3 = (value >> 12) & 0x7;
                let rs2 = ((value >> 20) & 0x1f) as usize;
                let imm_0_4 = (value >> 7) & 0x1e;
                let imm_5_10 = (value >> 25) & 0x3f;
                let imm_11 = value >> 7 & 0x1;
                let mut imm = (value as i32) >> 19 & !0xfff;
                imm |= (imm_0_4 | (imm_5_10 << 5) | imm_11 << 11) as i32;
                let rs1 = ((value >> 15) & 0x1f) as usize;
                match funct3 {
                    0x0 => Ok(Beq { rs1, rs2, imm }),
                    0x1 => Ok(Bne { rs1, rs2, imm }),
                    0x4 => Ok(Blt { rs1, rs2, imm }),
                    0x5 => Ok(Bge { rs1, rs2, imm }),
                    0x6 => Ok(Bltu { rs1, rs2, imm }),
                    0x7 => Ok(Bgeu { rs1, rs2, imm }),
                    _ => bail!("invalid branch funct3:{:x}", funct3),
                }
            }
            // Jump And Link - J FMT
            0x6f => {
                let rd = ((value >> 7) & 0x1f) as usize;
                let imm_1_10 = (value >> 21) & 0x3ff;
                let imm_11 = (value >> 20) & 0x1;
                let imm_12_19 = (value >> 12) & 0xff;
                let mut imm = (value as i32) >> 11 & !0xfffff;
                imm |= (imm_1_10 << 1 | imm_11 << 11 | imm_12_19 << 12) as i32;
                Ok(Jal { rd, imm })
            }
            // Jump And Link Reg - I FMT
            0x67 => {
                let funct3 = (value >> 12) & 0x7;
                if funct3 != 0x0 {
                    bail!("invalid jalr funct3:{}", funct3);
                }
                let rd = ((value >> 7) & 0x1f) as usize;
                let rs1 = ((value >> 15) & 0x1f) as usize;
                let imm = (value as i32) >> 20;
                Ok(Jalr { rd, rs1, imm })
            }
            // Load Upper Imm - U FMT
            0x37 => {
                let rd = ((value >> 7) & 0x1f) as usize;
                let imm = value & 0xffff_f000;
                Ok(Lui { rd, imm })
            }
            // Add Upper Imm to PC - U FMT
            0x17 => {
                let rd = ((value >> 7) & 0x1f) as usize;
                let imm = value & 0xffff_f000;
                Ok(Auipc { rd, imm })
            }
            // Environment
            0x73 => {
                let funct3 = (value >> 12) & 0x7;
                if funct3 != 0x00 {
                    bail!("invalid environment funct3:{:x}", funct3);
                }
                let imm = ((value >> 20) & 0xfff) as i32;
                match imm {
                    0x0 => Ok(Ecall),
                    0x1 => Ok(Ebreak),
                    _ => bail!("invalid environment imm:{:x}", imm),
                }
            }
            _ => bail!("invalid opcode:{:b}", opcode),
        }
    }
}

struct Machine {
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
        self.pc += 4
    }

    fn jump(&mut self, offset: i32) {
        self.pc = self.pc.wrapping_add_signed(offset as isize);
    }

    fn parse_inst(&self) -> anyhow::Result<Instruction> {
        let word = u32::from_le_bytes(self.memory[self.pc..self.pc + 4].try_into()?);
        Instruction::try_from(word)
    }

    fn exec_inst(&mut self, inst: &Instruction) -> anyhow::Result<()> {
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
                self.wr(*rd, self.rr(*rs1) << self.rr(*rs2));
                self.advance_pc();
            }
            Srl { rd, rs1, rs2 } => {
                self.wr(*rd, (self.rru(*rs1) >> self.rr(*rs2)) as i32);
                self.advance_pc();
            }
            Sra { rd, rs1, rs2 } => {
                self.wr(*rd, self.rr(*rs1) >> self.rr(*rs2));
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
                self.wr(*rd, self.rr(*rs1) << *imm);
                self.advance_pc();
            }
            Srli { rd, rs1, imm } => {
                self.wr(*rd, (self.rru(*rs1) >> *imm) as i32);
                self.advance_pc();
            }
            Srai { rd, rs1, imm } => {
                self.wr(*rd, self.rr(*rs1) >> *imm);
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
                let addr = (self.rr(*rs1) + imm) as usize;
                let byte = self.memory[addr] as i8 as i32;
                self.wr(*rd, byte);
                self.advance_pc();
            }
            Lh { rd, rs1, imm } => {
                let addr = (self.rr(*rs1) + imm) as usize;
                let hw: i16 = i16::from_le_bytes(self.memory[addr..addr + 2].try_into()?);
                self.wr(*rd, hw as i32);
                self.advance_pc();
            }
            Lw { rd, rs1, imm } => {
                let addr = (self.rr(*rs1) + imm) as usize;
                let wd: i32 = i32::from_le_bytes(self.memory[addr..addr + 4].try_into()?);
                self.wr(*rd, wd);
                self.advance_pc();
            }
            Lbu { rd, rs1, imm } => {
                let addr = (self.rr(*rs1) + imm) as usize;
                let byte = self.memory[addr];
                self.wr(*rd, byte as i32);
                self.advance_pc();
            }
            Lhu { rd, rs1, imm } => {
                let addr = (self.rr(*rs1) + imm) as usize;
                let hw: u16 = u16::from_le_bytes(self.memory[addr..addr + 2].try_into()?);
                self.wr(*rd, hw as i32);
                self.advance_pc();
            }
            Sb { rs2, imm, rs1 } => {
                let addr = (self.rr(*rs1) + imm) as usize;
                let val = self.rr(*rs2) as u8;
                self.memory[addr] = val;
                self.advance_pc();
            }
            Sh { rs2, imm, rs1 } => {
                let addr = (self.rr(*rs1) + imm) as usize;
                let val = self.rr(*rs2) as u16;
                self.memory[addr..addr + 1].copy_from_slice(&val.to_le_bytes());
                self.advance_pc();
            }
            Sw { rs2, imm, rs1 } => {
                let addr = (self.rr(*rs1) + imm) as usize;
                let val = self.rr(*rs2);
                self.memory[addr..addr + 1].copy_from_slice(&val.to_le_bytes());
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
                self.wru(*rd, self.pc as u32 + 4);
                self.jump(*imm);
            }
            Jalr { rd, rs1, imm } => {
                self.wru(*rd, self.pc as u32 + 4);
                let addr = self.rr(*rs1) as isize + *imm as isize;
                self.pc = addr as usize;
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
            Ebreak => bail!("ebreak not supported"),
        }
        Ok(())
    }

    fn step(&mut self) -> anyhow::Result<()> {
        let inst = self.parse_inst()?;
        self.exec_inst(&inst)?;
        Ok(())
    }

    fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.step()?
        }
    }

    fn dump_reg(&self) {
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

    fn debug(&self) {
        self.dump_reg();
    }
}

fn main() -> anyhow::Result<()> {
    use Instruction::*;
    let mut machine = Machine::new(4096);

    let inst = vec![
        Addi {
            rd: 1,
            rs1: 0,
            imm: 0x1,
        },
        Addi {
            rd: 2,
            rs1: 0,
            imm: 0x11,
        },
        Add {
            rd: 3,
            rs1: 1,
            rs2: 2,
        },
    ];
    for it in inst {
        machine.exec_inst(&it);
    }
    machine.debug();
    Ok(())
}

#[cfg(test)]
mod inst_tests {
    use super::Instruction::{self, *};

    #[test]
    fn validate() {
        let rd = 1;
        let rs1 = 2;
        let rs2 = 3;
        let imm = -2048;
        let maps = vec![
            (
                0b0000000_00011_00010_000_00001_0110011_u32,
                Add { rd, rs1, rs2 },
            ),
            (
                0b0100000_00011_00010_000_00001_0110011,
                Sub { rd, rs1, rs2 },
            ),
            (
                0b0000000_00011_00010_100_00001_0110011,
                Xor { rd, rs1, rs2 },
            ),
            (0b0000000_00011_00010_110_00001_0110011, Or { rd, rs1, rs2 }),
            (
                0b0000000_00011_00010_111_00001_0110011,
                And { rd, rs1, rs2 },
            ),
            (
                0b0000000_00011_00010_001_00001_0110011,
                Sll { rd, rs1, rs2 },
            ),
            (
                0b0000000_00011_00010_101_00001_0110011,
                Srl { rd, rs1, rs2 },
            ),
            (
                0b0100000_00011_00010_101_00001_0110011,
                Sra { rd, rs1, rs2 },
            ),
            (
                0b0000000_00011_00010_010_00001_0110011,
                Slt { rd, rs1, rs2 },
            ),
            (
                0b0000000_00011_00010_011_00001_0110011,
                Sltu { rd, rs1, rs2 },
            ),
            (
                0b1000_0000_0000_00010_000_00001_0010011,
                Addi { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_100_00001_0010011,
                Xori { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_110_00001_0010011,
                Ori { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_111_00001_0010011,
                Andi { rd, rs1, imm },
            ),
            (
                0b0000_0000_1000_00010_001_00001_0010011,
                Slli { rd, rs1, imm: 8 },
            ),
            (
                0b0000_0000_1000_00010_101_00001_0010011,
                Srli { rd, rs1, imm: 8 },
            ),
            (
                0b0100_0000_1000_00010_101_00001_0010011,
                Srai { rd, rs1, imm: 8 },
            ),
            (
                0b1000_0000_0000_00010_010_00001_0010011,
                Slti { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_011_00001_0010011,
                Sltiu {
                    rd,
                    rs1,
                    imm: imm as u32,
                },
            ),
            (
                0b1000_0000_0000_00010_000_00001_0000011,
                Lb { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_001_00001_0000011,
                Lh { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_010_00001_0000011,
                Lw { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_100_00001_0000011,
                Lbu { rd, rs1, imm },
            ),
            (
                0b1000_0000_0000_00010_101_00001_0000011,
                Lhu { rd, rs1, imm },
            ),
            (
                0b1000000_00011_00010_000_00000_0100011,
                Sb { rs2, imm, rs1 },
            ),
            (
                0b1000000_00011_00010_001_00000_0100011,
                Sh { rs2, imm, rs1 },
            ),
            (
                0b1000000_00011_00010_010_00000_0100011,
                Sw { rs2, imm, rs1 },
            ),
            (
                0b1000000_00011_00010_000_00001_1100011,
                Beq { rs1, rs2, imm },
            ),
            (
                0b1000000_00011_00010_001_00001_1100011,
                Bne { rs1, rs2, imm },
            ),
            (
                0b1000000_00011_00010_100_00001_1100011,
                Blt { rs1, rs2, imm },
            ),
            (
                0b1000000_00011_00010_101_00001_1100011,
                Bge { rs1, rs2, imm },
            ),
            (
                0b1000000_00011_00010_110_00001_1100011,
                Bltu { rs1, rs2, imm },
            ),
            (
                0b1000000_00011_00010_111_00001_1100011,
                Bgeu { rs1, rs2, imm },
            ),
            (0b1_0000000000_1_11111111_00001_1101111, Jal { rd, imm }),
            (
                0b100000000000_00010_000_00001_1100111,
                Jalr { rd, rs1, imm },
            ),
            (
                0b00010010001101000101_00001_0110111,
                Lui {
                    rd,
                    imm: 0x12345000,
                },
            ),
            (0b0000_0000_0000_00000000_00000_1110011, Ecall),
            (0b0000_0000_0001_00000000_00000_1110011, Ebreak),
        ];
        for (bcode, inst) in maps {
            assert_eq!(Instruction::try_from(bcode).unwrap(), inst)
        }
    }
}
