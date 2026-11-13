use anyhow::{Ok, anyhow, bail};

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
    /// Fence memory and I/O operations
    Fence,
}

fn rd_field(word: u32) -> usize {
    ((word >> 7) & 0x1f) as usize
}

fn rs1_field(word: u32) -> usize {
    ((word >> 15) & 0x1f) as usize
}

fn rs2_field(word: u32) -> usize {
    ((word >> 20) & 0x1f) as usize
}

fn funct3_field(word: u32) -> u32 {
    (word >> 12) & 0x7
}

fn sign_extend(value: u32, width: u32) -> i32 {
    ((value << (32 - width)) as i32) >> (32 - width)
}

impl TryFrom<u32> for Instruction {
    type Error = anyhow::Error;
    fn try_from(value: u32) -> anyhow::Result<Self> {
        use Instruction::*;
        let opcode = value & 0x7f;
        match opcode {
            // Fence - I FMT
            0x0f => {
                let funct3 = funct3_field(value);
                let fm = value >> 28;
                let pred = (value >> 24) & 0xf;
                let succ = (value >> 20) & 0xf;
                let rd = rd_field(value);
                let rs1 = rs1_field(value);
                let valid_fm = fm == 0 || (fm == 0b1000 && pred == 0b0011 && succ == 0b0011);
                if funct3 != 0 || rd != 0 || rs1 != 0 || !valid_fm {
                    bail!("invalid fence instruction:{:08x}", value);
                }
                Ok(Fence)
            }
            // Arith - R FMT
            0x33 => {
                let funct3 = funct3_field(value);
                let funct7 = value >> 25;
                let rd = rd_field(value);
                let rs1 = rs1_field(value);
                let rs2 = rs2_field(value);
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
                let funct3 = funct3_field(value);
                let rd = rd_field(value);
                let rs1 = rs1_field(value);
                let imm = sign_extend(value >> 20, 12);
                match funct3 {
                    0x0 => Ok(Addi { rd, rs1, imm }),
                    0x4 => Ok(Xori { rd, rs1, imm }),
                    0x6 => Ok(Ori { rd, rs1, imm }),
                    0x7 => Ok(Andi { rd, rs1, imm }),
                    0x1 => {
                        if value >> 25 != 0 {
                            bail!("invalid shift left logical imm:{:x}", value >> 20);
                        }
                        Ok(Slli { rd, rs1, imm })
                    }
                    0x5 => match value >> 25 {
                        0x00 => Ok(Srli { rd, rs1, imm }),
                        0x20 => Ok(Srai {
                            rd,
                            rs1,
                            imm: ((value >> 20) & 0x1f) as i32,
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
                let funct3 = funct3_field(value);
                let rd = rd_field(value);
                let rs1 = rs1_field(value);
                let imm = sign_extend(value >> 20, 12);
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
                let funct3 = funct3_field(value);
                let rs2 = rs2_field(value);
                let rs1 = rs1_field(value);
                let imm = sign_extend(((value >> 25) << 5) | ((value >> 7) & 0x1f), 12);
                match funct3 {
                    0x0 => Ok(Sb { rs2, imm, rs1 }),
                    0x1 => Ok(Sh { rs2, imm, rs1 }),
                    0x2 => Ok(Sw { rs2, imm, rs1 }),
                    _ => bail!("invalid store funct3:{:x}", funct3),
                }
            }
            // Branch - B FMT
            0x63 => {
                let funct3 = funct3_field(value);
                let rs2 = rs2_field(value);
                let rs1 = rs1_field(value);
                let imm = sign_extend(
                    (((value >> 31) & 1) << 12)
                        | (((value >> 7) & 1) << 11)
                        | (((value >> 25) & 0x3f) << 5)
                        | (((value >> 8) & 0xf) << 1),
                    13,
                );
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
                let rd = rd_field(value);
                let imm = sign_extend(
                    (((value >> 31) & 1) << 20)
                        | (((value >> 12) & 0xff) << 12)
                        | (((value >> 20) & 1) << 11)
                        | (((value >> 21) & 0x3ff) << 1),
                    21,
                );
                Ok(Jal { rd, imm })
            }
            // Jump And Link Reg - I FMT
            0x67 => {
                let funct3 = funct3_field(value);
                if funct3 != 0x0 {
                    bail!("invalid jalr funct3:{}", funct3);
                }
                let rd = rd_field(value);
                let rs1 = rs1_field(value);
                let imm = sign_extend(value >> 20, 12);
                Ok(Jalr { rd, rs1, imm })
            }
            // Load Upper Imm - U FMT
            0x37 => {
                let rd = rd_field(value);
                let imm = value & 0xffff_f000;
                Ok(Lui { rd, imm })
            }
            // Add Upper Imm to PC - U FMT
            0x17 => {
                let rd = rd_field(value);
                let imm = value & 0xffff_f000;
                Ok(Auipc { rd, imm })
            }
            // Environment
            0x73 => {
                let funct3 = funct3_field(value);
                let rd = rd_field(value);
                let rs1 = rs1_field(value);
                if funct3 != 0x00 || rd != 0 || rs1 != 0 {
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

impl From<Instruction> for u32 {
    fn from(inst: Instruction) -> Self {
        use Instruction::*;

        fn encode_r(funct7: u32, rs2: usize, rs1: usize, funct3: u32, rd: usize) -> u32 {
            (funct7 << 25)
                | ((rs2 as u32 & 0x1f) << 20)
                | ((rs1 as u32 & 0x1f) << 15)
                | (funct3 << 12)
                | ((rd as u32 & 0x1f) << 7)
                | 0x33
        }

        fn encode_i(opcode: u32, funct3: u32, rd: usize, rs1: usize, imm: u32) -> u32 {
            ((imm & 0xfff) << 20)
                | ((rs1 as u32 & 0x1f) << 15)
                | (funct3 << 12)
                | ((rd as u32 & 0x1f) << 7)
                | opcode
        }

        fn encode_s(funct3: u32, rs2: usize, imm: i32, rs1: usize) -> u32 {
            let imm = imm as u32 & 0xfff;
            ((imm >> 5) << 25)
                | ((rs2 as u32 & 0x1f) << 20)
                | ((rs1 as u32 & 0x1f) << 15)
                | (funct3 << 12)
                | ((imm & 0x1f) << 7)
                | 0x23
        }

        fn encode_b(funct3: u32, rs1: usize, rs2: usize, imm: i32) -> u32 {
            let imm = imm as u32 & 0x1fff;
            (((imm >> 12) & 1) << 31)
                | (((imm >> 5) & 0x3f) << 25)
                | ((rs2 as u32 & 0x1f) << 20)
                | ((rs1 as u32 & 0x1f) << 15)
                | (funct3 << 12)
                | (((imm >> 1) & 0xf) << 8)
                | (((imm >> 11) & 1) << 7)
                | 0x63
        }

        fn encode_j(rd: usize, imm: i32) -> u32 {
            let imm = imm as u32 & 0x1f_ffff;
            (((imm >> 20) & 1) << 31)
                | (((imm >> 1) & 0x3ff) << 21)
                | (((imm >> 11) & 1) << 20)
                | (((imm >> 12) & 0xff) << 12)
                | ((rd as u32 & 0x1f) << 7)
                | 0x6f
        }

        match inst {
            Add { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 0, rd),
            Sub { rd, rs1, rs2 } => encode_r(0x20, rs2, rs1, 0, rd),
            Xor { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 4, rd),
            Or { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 6, rd),
            And { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 7, rd),
            Sll { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 1, rd),
            Srl { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 5, rd),
            Sra { rd, rs1, rs2 } => encode_r(0x20, rs2, rs1, 5, rd),
            Slt { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 2, rd),
            Sltu { rd, rs1, rs2 } => encode_r(0, rs2, rs1, 3, rd),
            Addi { rd, rs1, imm } => encode_i(0x13, 0, rd, rs1, imm as u32),
            Xori { rd, rs1, imm } => encode_i(0x13, 4, rd, rs1, imm as u32),
            Ori { rd, rs1, imm } => encode_i(0x13, 6, rd, rs1, imm as u32),
            Andi { rd, rs1, imm } => encode_i(0x13, 7, rd, rs1, imm as u32),
            Slli { rd, rs1, imm } => encode_i(0x13, 1, rd, rs1, imm as u32 & 0x1f),
            Srli { rd, rs1, imm } => encode_i(0x13, 5, rd, rs1, imm as u32 & 0x1f),
            Srai { rd, rs1, imm } => encode_i(0x13, 5, rd, rs1, 0x400 | (imm as u32 & 0x1f)),
            Slti { rd, rs1, imm } => encode_i(0x13, 2, rd, rs1, imm as u32),
            Sltiu { rd, rs1, imm } => encode_i(0x13, 3, rd, rs1, imm),
            Lb { rd, rs1, imm } => encode_i(0x03, 0, rd, rs1, imm as u32),
            Lh { rd, rs1, imm } => encode_i(0x03, 1, rd, rs1, imm as u32),
            Lw { rd, rs1, imm } => encode_i(0x03, 2, rd, rs1, imm as u32),
            Lbu { rd, rs1, imm } => encode_i(0x03, 4, rd, rs1, imm as u32),
            Lhu { rd, rs1, imm } => encode_i(0x03, 5, rd, rs1, imm as u32),
            Sb { rs2, imm, rs1 } => encode_s(0, rs2, imm, rs1),
            Sh { rs2, imm, rs1 } => encode_s(1, rs2, imm, rs1),
            Sw { rs2, imm, rs1 } => encode_s(2, rs2, imm, rs1),
            Beq { rs1, rs2, imm } => encode_b(0, rs1, rs2, imm),
            Bne { rs1, rs2, imm } => encode_b(1, rs1, rs2, imm),
            Blt { rs1, rs2, imm } => encode_b(4, rs1, rs2, imm),
            Bge { rs1, rs2, imm } => encode_b(5, rs1, rs2, imm),
            Bltu { rs1, rs2, imm } => encode_b(6, rs1, rs2, imm),
            Bgeu { rs1, rs2, imm } => encode_b(7, rs1, rs2, imm),
            Jal { rd, imm } => encode_j(rd, imm),
            Jalr { rd, rs1, imm } => encode_i(0x67, 0, rd, rs1, imm as u32),
            Lui { rd, imm } => (imm & 0xffff_f000) | ((rd as u32 & 0x1f) << 7) | 0x37,
            Auipc { rd, imm } => (imm & 0xffff_f000) | ((rd as u32 & 0x1f) << 7) | 0x17,
            Ecall => 0x0000_0073,
            Ebreak => 0x0010_0073,
            Fence => 0x0000_000f,
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
            Ebreak => bail!("ebreak not supported"),
            Fence => self.advance_pc(),
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
        machine.exec_inst(&it)?;
    }
    machine.debug();
    Ok(())
}

#[cfg(test)]
mod inst_tests {
    use super::Instruction::{self, *};

    fn assert_round_trip(word: u32, expected: Instruction) {
        assert_eq!(Instruction::try_from(word).unwrap(), expected);
        assert_eq!(u32::from(expected), word);
    }

    #[test]
    fn decodes_add() {
        assert_round_trip(
            0b0000000_00011_00010_000_00001_0110011,
            Add {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_sub() {
        assert_round_trip(
            0b0100000_00011_00010_000_00001_0110011,
            Sub {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_xor() {
        assert_round_trip(
            0b0000000_00011_00010_100_00001_0110011,
            Xor {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_or() {
        assert_round_trip(
            0b0000000_00011_00010_110_00001_0110011,
            Or {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_and() {
        assert_round_trip(
            0b0000000_00011_00010_111_00001_0110011,
            And {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_sll() {
        assert_round_trip(
            0b0000000_00011_00010_001_00001_0110011,
            Sll {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_srl() {
        assert_round_trip(
            0b0000000_00011_00010_101_00001_0110011,
            Srl {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_sra() {
        assert_round_trip(
            0b0100000_00011_00010_101_00001_0110011,
            Sra {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_slt() {
        assert_round_trip(
            0b0000000_00011_00010_010_00001_0110011,
            Slt {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_sltu() {
        assert_round_trip(
            0b0000000_00011_00010_011_00001_0110011,
            Sltu {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );
    }

    #[test]
    fn decodes_addi() {
        assert_round_trip(
            0b1000_0000_0000_00010_000_00001_0010011,
            Addi {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_xori() {
        assert_round_trip(
            0b1000_0000_0000_00010_100_00001_0010011,
            Xori {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_ori() {
        assert_round_trip(
            0b1000_0000_0000_00010_110_00001_0010011,
            Ori {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_andi() {
        assert_round_trip(
            0b1000_0000_0000_00010_111_00001_0010011,
            Andi {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_slli() {
        assert_round_trip(
            0b0000_0000_1000_00010_001_00001_0010011,
            Slli {
                rd: 1,
                rs1: 2,
                imm: 8,
            },
        );
    }

    #[test]
    fn decodes_srli() {
        assert_round_trip(
            0b0000_0000_1000_00010_101_00001_0010011,
            Srli {
                rd: 1,
                rs1: 2,
                imm: 8,
            },
        );
    }

    #[test]
    fn decodes_srai() {
        assert_round_trip(
            0b0100_0000_1000_00010_101_00001_0010011,
            Srai {
                rd: 1,
                rs1: 2,
                imm: 8,
            },
        );
    }

    #[test]
    fn decodes_slti() {
        assert_round_trip(
            0b1000_0000_0000_00010_010_00001_0010011,
            Slti {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_sltiu() {
        assert_round_trip(
            0b1000_0000_0000_00010_011_00001_0010011,
            Sltiu {
                rd: 1,
                rs1: 2,
                imm: 0xffff_f800,
            },
        );
    }

    #[test]
    fn decodes_lb() {
        assert_round_trip(
            0b1000_0000_0000_00010_000_00001_0000011,
            Lb {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_lh() {
        assert_round_trip(
            0b1000_0000_0000_00010_001_00001_0000011,
            Lh {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_lw() {
        assert_round_trip(
            0b1000_0000_0000_00010_010_00001_0000011,
            Lw {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_lbu() {
        assert_round_trip(
            0b1000_0000_0000_00010_100_00001_0000011,
            Lbu {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_lhu() {
        assert_round_trip(
            0b1000_0000_0000_00010_101_00001_0000011,
            Lhu {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_sb() {
        assert_round_trip(
            0b1000000_00011_00010_000_00000_0100011,
            Sb {
                rs2: 3,
                imm: -2048,
                rs1: 2,
            },
        );
    }

    #[test]
    fn decodes_sh() {
        assert_round_trip(
            0b1000000_00011_00010_001_00000_0100011,
            Sh {
                rs2: 3,
                imm: -2048,
                rs1: 2,
            },
        );
    }

    #[test]
    fn decodes_sw() {
        assert_round_trip(
            0b1000000_00011_00010_010_00000_0100011,
            Sw {
                rs2: 3,
                imm: -2048,
                rs1: 2,
            },
        );
    }

    #[test]
    fn decodes_beq() {
        assert_round_trip(
            0b1000000_00011_00010_000_00001_1100011,
            Beq {
                rs1: 2,
                rs2: 3,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_bne() {
        assert_round_trip(
            0b1000000_00011_00010_001_00001_1100011,
            Bne {
                rs1: 2,
                rs2: 3,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_blt() {
        assert_round_trip(
            0b1000000_00011_00010_100_00001_1100011,
            Blt {
                rs1: 2,
                rs2: 3,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_bge() {
        assert_round_trip(
            0b1000000_00011_00010_101_00001_1100011,
            Bge {
                rs1: 2,
                rs2: 3,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_bltu() {
        assert_round_trip(
            0b1000000_00011_00010_110_00001_1100011,
            Bltu {
                rs1: 2,
                rs2: 3,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_bgeu() {
        assert_round_trip(
            0b1000000_00011_00010_111_00001_1100011,
            Bgeu {
                rs1: 2,
                rs2: 3,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_jal() {
        assert_round_trip(
            0b1_0000000000_1_11111111_00001_1101111,
            Jal { rd: 1, imm: -2048 },
        );
    }

    #[test]
    fn decodes_jalr() {
        assert_round_trip(
            0b100000000000_00010_000_00001_1100111,
            Jalr {
                rd: 1,
                rs1: 2,
                imm: -2048,
            },
        );
    }

    #[test]
    fn decodes_lui() {
        assert_round_trip(
            0b00010010001101000101_00001_0110111,
            Lui {
                rd: 1,
                imm: 0x1234_5000,
            },
        );
    }

    #[test]
    fn decodes_auipc() {
        assert_round_trip(
            0b00010010001101000101_00001_0010111,
            Auipc {
                rd: 1,
                imm: 0x1234_5000,
            },
        );
    }

    #[test]
    fn decodes_fence() {
        assert_round_trip(0x0000_000f, Fence);
    }

    #[test]
    fn decodes_fence_tso() {
        assert_eq!(Instruction::try_from(0x8330_000f).unwrap(), Fence);
    }

    #[test]
    fn rejects_invalid_fence_encoding() {
        assert!(Instruction::try_from(0x8110_000f).is_err());
    }

    #[test]
    fn rejects_ecall_with_nonzero_rd() {
        assert!(Instruction::try_from(0x0000_00f3).is_err());
    }

    #[test]
    fn decodes_ecall() {
        assert_round_trip(0b0000_0000_0000_00000000_00000_1110011, Ecall);
    }

    #[test]
    fn decodes_ebreak() {
        assert_round_trip(0b0000_0000_0001_00000000_00000_1110011, Ebreak);
    }
}

#[cfg(test)]
mod execution_tests {
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
