use std::fmt;

use crate::{
    mem::Mem,
    register::{
        ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
        STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    },
};

use super::{
    instruction::{EffectiveAddressingMode, OperationSize},
    Cpu, StatusRegisterResult,
};

// TODO: No need for this any longer
pub struct EffectiveAddressValue<T> {
    pub value: T,
}

pub struct SetEffectiveAddressValueResult {
    pub status_register_result: StatusRegisterResult,
}

pub struct EffectiveAddressDebug {
    pub format: String,
}

impl fmt::Display for EffectiveAddressDebug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format)
    }
}

pub struct EffectiveAddressingData {
    pub instr_word: u16,
    pub operation_size: OperationSize,
    pub ea_mode: EffectiveAddressingMode,
}

impl EffectiveAddressingData {
    pub fn create(
        instr_word: u16,
        operation_size: OperationSize,
        ea_mode: EffectiveAddressingMode,
    ) -> EffectiveAddressingData {
        EffectiveAddressingData {
            instr_word,
            operation_size,
            ea_mode,
        }
    }

    pub fn get_address(&self, pc: &mut ProgramCounter, reg: &mut Register, mem: &Mem) -> u32 {
        match self.ea_mode {
            EffectiveAddressingMode::DRegDirect {
                ea_register: register,
            } => {
                // Dn
                panic!("Cannot get Effective Address for 'Data Register Direct' EA mode");
            }
            EffectiveAddressingMode::ARegDirect {
                ea_register: register,
            } => {
                // An
                panic!("Cannot get Effective Address for 'Address Register Direct' EA mode");
            }
            EffectiveAddressingMode::ARegIndirect {
                ea_register,
                ea_address,
            } => {
                // (An)
                ea_address
            }
            EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                operation_size,
                ea_register,
            } => {
                // (An)+
                reg.get_a_reg_long(ea_register)
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                operation_size,
                ea_register,
            } => {
                // (-An)
                let (ea_address, _) = reg
                    .get_a_reg_long(ea_register)
                    .overflowing_sub(operation_size.size_in_bytes());
                ea_address
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement {
                ea_register,
                ea_address,
                ea_displacement,
            } => {
                // (d16,An)
                ea_address
            }
            EffectiveAddressingMode::ARegIndirectWithIndexOrMemoryIndirect {
                ea_register,
                ea_address,
                extension_word,
                displacement,
                register_type,
                register,
                index_size,
                scale_factor,
            } => {
                // ARegIndirectWithIndex8BitDisplacement (d8, An, Xn.SIZE*SCALE)
                // ARegIndirectWithIndexBaseDisplacement (bd, An, Xn.SIZE*SCALE)
                // MemoryIndirectPostIndexed             ([bd, An], Xn.SIZE*SCALE,od)
                // MemoryIndirectPreIndexed              ([bd, An, Xn.SIZE*SCALE],od)
                ea_address
            }
            EffectiveAddressingMode::PcIndirectWithDisplacement {
                ea_address,
                displacement,
            } => {
                // (d16,PC)
                ea_address
            }
            EffectiveAddressingMode::AbsoluteShortAddressing {
                ea_address,
                displacement,
            } => {
                // (xxx).W
                ea_address
            }
            EffectiveAddressingMode::AbsolutLongAddressing { ea_address } => {
                // (xxx).L
                ea_address
            }
            EffectiveAddressingMode::PcIndirectWithIndexOrPcMemoryIndirect {
                ea_register,
                ea_address,
                extension_word,
                displacement,
                register_type,
                register,
                index_size,
                scale_factor,
            } => {
                // PcIndirectWithIndex8BitDisplacement (d8, PC, Xn.SIZE*SCALE)
                // PcIndirectWithIndexBaseDisplacement (bd, PC, Xn.SIZE*SCALE)
                // PcMemoryInderectPostIndexed         ([bd, PC], Xn.SIZE*SCALE,od)
                // PcMemoryInderectPreIndexed          ([bd, PC, Xn.SIZE*SCALE],od)
                ea_address
            }
            EffectiveAddressingMode::ImmediateDataByte { .. }
            | EffectiveAddressingMode::ImmediateDataWord { .. }
            | EffectiveAddressingMode::ImmediateDataLong { .. } => {
                // #<xxx>
                panic!("Cannot get Effective Address for 'Data Register Direct' EA mode");
            }
        }
    }

    pub fn get_value_byte(
        &self,
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &Mem,
        apply_increment_decrement: bool,
    ) -> u8 {
        match self.ea_mode {
            EffectiveAddressingMode::DRegDirect { ea_register } => {
                // Dn
                reg.get_d_reg_byte(ea_register)
            }
            EffectiveAddressingMode::ARegDirect { ea_register } => {
                // An
                reg.get_a_reg_byte(ea_register)
            }
            EffectiveAddressingMode::ImmediateDataByte { data } => {
                // #<xxx>
                data
            }
            _ => {
                let ea = self.get_address(pc, reg, mem);
                let result = mem.get_byte(ea);
                if apply_increment_decrement {
                    match self.ea_mode {
                        EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                            operation_size,
                            ea_register,
                        } => reg.increment_a_reg(ea_register, self.operation_size),
                        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                            operation_size,
                            ea_register,
                        } => reg.decrement_a_reg(ea_register, self.operation_size),
                        _ => (),
                    }
                }
                result
            }
        }
    }

    pub fn get_value_word(
        &self,
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &Mem,
        apply_increment_decrement: bool,
    ) -> u16 {
        match self.ea_mode {
            EffectiveAddressingMode::DRegDirect { ea_register } => {
                // Dn
                reg.get_d_reg_word(ea_register)
            }
            EffectiveAddressingMode::ARegDirect { ea_register } => {
                // An
                reg.get_a_reg_word(ea_register)
            }
            EffectiveAddressingMode::ImmediateDataWord { data } => {
                // #<xxx>
                data
            }
            _ => {
                let ea = self.get_address(pc, reg, mem);
                let result = mem.get_word(ea);
                if apply_increment_decrement {
                    match self.ea_mode {
                        EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                            operation_size,
                            ea_register,
                        } => reg.increment_a_reg(ea_register, self.operation_size),
                        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                            operation_size,
                            // operation_size,
                            ea_register,
                            // ea_address,
                        } => reg.decrement_a_reg(ea_register, self.operation_size),
                        _ => (),
                    }
                }
                result
            }
        }
    }

    pub fn get_value_long(
        &self,
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &Mem,
        apply_increment_decrement: bool,
    ) -> u32 {
        match self.ea_mode {
            EffectiveAddressingMode::DRegDirect { ea_register } => {
                // Dn
                reg.get_d_reg_long(ea_register)
            }
            EffectiveAddressingMode::ARegDirect { ea_register } => {
                // An
                reg.get_a_reg_long(ea_register)
            }
            EffectiveAddressingMode::ImmediateDataLong { data } => {
                // #<xxx>
                data
            }
            _ => {
                let ea = self.get_address(pc, reg, mem);
                let result = mem.get_long(ea);
                if apply_increment_decrement {
                    match self.ea_mode {
                        EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                            operation_size,
                            ea_register,
                        } => reg.increment_a_reg(ea_register, self.operation_size),
                        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                            operation_size,
                            // operation_size,
                            ea_register,
                            // ea_address,
                        } => reg.decrement_a_reg(ea_register, self.operation_size),
                        _ => (),
                    }
                }
                result
            }
        }
    }

    pub fn set_value_byte(
        &self,
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &mut Mem,
        value: u8,
        apply_increment_decrement: bool,
    ) -> SetEffectiveAddressValueResult {
        match self.ea_mode {
            EffectiveAddressingMode::DRegDirect {
                ea_register: register,
            } => {
                // Dn
                reg.set_d_reg_byte(register, value);
            }
            EffectiveAddressingMode::ARegDirect {
                ea_register: register,
            } => {
                // An
                reg.set_a_reg_long(register, value as u32);
            }
            EffectiveAddressingMode::ImmediateDataByte { .. }
            | EffectiveAddressingMode::ImmediateDataWord { .. }
            | EffectiveAddressingMode::ImmediateDataLong { .. } => {
                // #<xxx>
                panic!("set_ea_value_word invalid EffectiveAddressingMode::ImmediateData");
            }
            _ => {
                let ea = self.get_address(pc, reg, mem);
                mem.set_byte(ea, value);
                if apply_increment_decrement {
                    match self.ea_mode {
                        EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                            operation_size,
                            ea_register,
                        } => reg.increment_a_reg(ea_register, self.operation_size),
                        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                            operation_size,
                            ea_register,
                            // ea_address,
                        } => reg.decrement_a_reg(ea_register, self.operation_size),
                        _ => (),
                    }
                }
            }
        };

        let value_signed = Cpu::get_signed_byte_from_byte(value);

        let mut status_register = 0x0000;

        match value_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i8::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        SetEffectiveAddressValueResult {
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn set_value_word(
        &self,
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &mut Mem,
        value: u16,
        apply_increment_decrement: bool,
    ) -> SetEffectiveAddressValueResult {
        match self.ea_mode {
            EffectiveAddressingMode::DRegDirect {
                ea_register: register,
            } => {
                // Dn
                reg.set_d_reg_word(register, value);
            }
            EffectiveAddressingMode::ARegDirect {
                ea_register: register,
            } => {
                // An
                reg.set_a_reg_long(register, value as u32);
            }
            EffectiveAddressingMode::ImmediateDataByte { .. }
            | EffectiveAddressingMode::ImmediateDataWord { .. }
            | EffectiveAddressingMode::ImmediateDataLong { .. } => {
                // #<xxx>
                panic!("set_ea_value_word invalid EffectiveAddressingMode::ImmediateData");
            }
            _ => {
                let ea = self.get_address(pc, reg, mem);
                mem.set_word(ea, value);
                if apply_increment_decrement {
                    match self.ea_mode {
                        EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                            operation_size,
                            ea_register,
                        } => reg.increment_a_reg(ea_register, self.operation_size),
                        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                            operation_size,
                            ea_register,
                            // ea_address,
                        } => reg.decrement_a_reg(ea_register, self.operation_size),
                        _ => (),
                    }
                }
            }
        };

        let value_signed = Cpu::get_signed_word_from_word(value);

        let mut status_register = 0x0000;

        match value_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i16::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        SetEffectiveAddressValueResult {
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn set_value_long(
        &self,
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &mut Mem,
        value: u32,
        apply_increment_decrement: bool,
    ) -> SetEffectiveAddressValueResult {
        match self.ea_mode {
            EffectiveAddressingMode::DRegDirect {
                ea_register: register,
            } => {
                // Dn
                reg.set_d_reg_long(register, value);
            }
            EffectiveAddressingMode::ARegDirect {
                ea_register: register,
            } => {
                // An
                reg.set_a_reg_long(register, value);
            }
            EffectiveAddressingMode::ImmediateDataByte { .. }
            | EffectiveAddressingMode::ImmediateDataWord { .. }
            | EffectiveAddressingMode::ImmediateDataLong { .. } => {
                // #<xxx>
                panic!("set_ea_value_word invalid EffectiveAddressingMode::ImmediateData");
            }
            _ => {
                let ea = self.get_address(pc, reg, mem);
                mem.set_long(ea, value);
                if apply_increment_decrement {
                    match self.ea_mode {
                        EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                            operation_size,
                            ea_register,
                        } => {
                            reg.increment_a_reg(ea_register, self.operation_size);
                        }
                        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                            operation_size,
                            ea_register,
                        } => {
                            reg.decrement_a_reg(ea_register, self.operation_size);
                        }
                        _ => (),
                    }
                }
            }
        };

        let value_signed = Cpu::get_signed_long_from_long(value);

        let mut status_register = 0x0000;

        match value_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i32::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        SetEffectiveAddressValueResult {
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }
}
