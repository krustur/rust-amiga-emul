use std::fmt;

use crate::{
    cpu::{
        ea::EffectiveAddressingData,
        instruction::{
            ConditionalTest, EffectiveAddressingMode, InstructionError, OperationSize, ScaleFactor,
        },
        step_log::{StepLog, StepLogEntry},
        Cpu, StatusRegisterResult,
    },
    mem::Mem,
};

pub const STATUS_REGISTER_MASK_CARRY: u16 = 0b0000000000000001;
pub const STATUS_REGISTER_MASK_OVERFLOW: u16 = 0b0000000000000010;
pub const STATUS_REGISTER_MASK_ZERO: u16 = 0b0000000000000100;
pub const STATUS_REGISTER_MASK_NEGATIVE: u16 = 0b0000000000001000;
pub const STATUS_REGISTER_MASK_EXTEND: u16 = 0b0000000000010000;

pub const STATUS_REGISTER_MASK_MASTER_INTERRUPT_STATE: u16 = 0b0001000000000000;
pub const STATUS_REGISTER_MASK_SUPERVISOR_STATE: u16 = 0b0010000000000000;

#[derive(Copy, Clone, Debug, std::cmp::PartialEq)]
pub enum RegisterType {
    Data,
    Address,
}

impl RegisterType {
    pub fn get_format(&self) -> char {
        match self {
            RegisterType::Address => 'A',
            RegisterType::Data => 'D',
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProgramCounter {
    address: u32,
    address_next: u32,
    address_jump: Option<u32>,
}

impl ProgramCounter {
    pub fn from_address(address: u32) -> ProgramCounter {
        ProgramCounter {
            address,
            address_next: address,
            address_jump: None,
        }
    }

    pub fn from_address_and_address_next(address: u32, address_next: u32) -> ProgramCounter {
        ProgramCounter {
            address,
            address_next: address,
            address_jump: None,
        }
    }

    pub fn branch_byte(&mut self, displacement: u8) {
        self.address_jump = Some(Cpu::get_address_with_byte_displacement_sign_extended(
            self.address.wrapping_add(2),
            displacement,
        ))
    }

    pub fn get_branch_byte_address(&self, displacement: u8) -> u32 {
        Cpu::get_address_with_byte_displacement_sign_extended(
            self.address.wrapping_add(2),
            displacement,
        )
    }

    pub fn branch_word(&mut self, displacement: u16) {
        self.address_jump = Some(Cpu::get_address_with_word_displacement_sign_extended(
            self.address.wrapping_add(2),
            displacement,
        ))
    }

    pub fn get_branch_word_address(&self, displacement: u16) -> u32 {
        Cpu::get_address_with_word_displacement_sign_extended(
            self.address.wrapping_add(2),
            displacement,
        )
    }

    pub fn branch_long(&mut self, displacement: u32) {
        self.address_jump = Some(Cpu::get_address_with_long_displacement(
            self.address.wrapping_add(2),
            displacement,
        ))
    }

    pub fn get_branch_long_address(&self, displacement: u32) -> u32 {
        Cpu::get_address_with_long_displacement(self.address.wrapping_add(2), displacement)
    }

    pub fn set_long(&mut self, address: u32) {
        self.address_jump = Some(address);
    }

    pub fn jump_long(&mut self, address: u32) {
        self.address_jump = Some(address);
    }

    pub fn skip_byte(&mut self) {
        self.address_next = self.address_next.wrapping_add(1);
    }

    pub fn fetch_next_byte(&mut self, mem: &Mem) -> u8 {
        let word = mem.get_byte_no_log(self.address_next);
        self.address_next = self.address_next.wrapping_add(1);
        word
    }

    pub fn peek_next_word(&self, mem: &Mem) -> u16 {
        let word = mem.get_word_no_log(self.address_next);
        word
    }

    pub fn skip_word(&mut self) {
        self.address_next = self.address_next.wrapping_add(2);
    }

    pub fn fetch_next_word(&mut self, mem: &Mem) -> u16 {
        let word = mem.get_word_no_log(self.address_next);
        self.address_next = self.address_next.wrapping_add(2);
        word
    }

    pub fn fetch_next_long(&mut self, mem: &Mem) -> u32 {
        let word = mem.get_long_no_log(self.address_next);
        self.address_next = self.address_next.wrapping_add(4);
        word
    }

    pub fn get_step_next_pc(&self) -> ProgramCounter {
        match self.address_jump {
            None => ProgramCounter {
                address: self.address_next,
                address_next: self.address_next,
                address_jump: None,
            },
            Some(x) => ProgramCounter {
                address: x,
                address_next: x,
                address_jump: None,
            },
        }
    }

    pub fn get_address(&self) -> u32 {
        self.address
    }

    pub fn get_address_next(&self) -> u32 {
        self.address_next
    }

    pub fn get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0<T>(
        &mut self,
        instr_word: u16,
        reg: &Register,
        mem: &Mem,
        step_log: &mut StepLog,
        get_operation_size_func: T,
    ) -> Result<EffectiveAddressingData, InstructionError>
    where
        T: Fn(u16) -> Result<OperationSize, InstructionError>,
    {
        self.get_effective_addressing_data_from_bit_pos(
            instr_word,
            reg,
            mem,
            step_log,
            get_operation_size_func,
            3,
            0,
        )
    }

    pub fn get_effective_addressing_data_from_bit_pos<T>(
        &mut self,
        instr_word: u16,
        reg: &Register,
        mem: &Mem,
        step_log: &mut StepLog,
        get_operation_size_func: T,
        bit_pos: u8,
        reg_bit_pos: u8,
    ) -> Result<EffectiveAddressingData, InstructionError>
    where
        T: Fn(u16) -> Result<OperationSize, InstructionError>,
    {
        let ea_mode = (instr_word >> bit_pos) & 0x0007;
        let ea_register = Cpu::extract_register_index_from_bit_pos(instr_word, reg_bit_pos)?;
        let operation_size = get_operation_size_func(instr_word)?;
        let ea_mode = match ea_mode {
            0b000 => EffectiveAddressingMode::DRegDirect {
                ea_register: (ea_register),
            },
            0b001 => EffectiveAddressingMode::ARegDirect {
                ea_register: (ea_register),
            },
            0b010 => {
                let address = reg.get_a_reg_long(ea_register, step_log);

                EffectiveAddressingMode::ARegIndirect {
                    ea_register,
                    ea_address: address,
                }
            }
            0b011 => EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                operation_size,
                ea_register,
            },
            0b100 => {
                // (-An)
                EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                    operation_size,
                    ea_register,
                }
            }
            0b101 => {
                let displacement = self.fetch_next_word(mem);
                let (address, _) = reg
                    .get_a_reg_long(ea_register, step_log)
                    .overflowing_add(Cpu::sign_extend_word(displacement));

                // println!("EffectiveAddressingMode::ARegIndirectWithDisplacement: A{}=${:08X}, displacement=${:08X}", ea_register,reg
                // .get_a_reg_long(ea_register), Cpu::sign_extend_word(displacement));
                EffectiveAddressingMode::ARegIndirectWithDisplacement {
                    ea_register,
                    ea_address: address,
                    ea_displacement: displacement,
                }
            }
            0b110 => {
                let extension_word = self.fetch_next_word(mem);
                let extension_word_format = match extension_word & 0x0100 {
                    0x0100 => 'F', // full
                    _ => 'B',      // brief
                };
                if extension_word_format == 'F' {
                    todo!("Full extension word format not implemented")
                }

                // TODO: Fix step_log here
                let register = Cpu::extract_register_index_from_bit_pos(extension_word, 12)?;
                let (register_value, register_type) = match extension_word & 0x8000 {
                    0x8000 => (
                        reg.get_a_reg_long(register, step_log),
                        RegisterType::Address,
                    ),
                    _ => (reg.get_d_reg_long(register, step_log), RegisterType::Data),
                };
                let (register_value, index_size) = match extension_word & 0x0800 {
                    0x0800 => (register_value, OperationSize::Long),
                    _ => (
                        Cpu::sign_extend_word(Cpu::get_word_from_long(register_value)),
                        OperationSize::Word,
                    ),
                };
                let scale_factor = Cpu::extract_scale_factor_from_bit_pos(extension_word, 9);

                let register_value = match scale_factor {
                    ScaleFactor::One => register_value,
                    ScaleFactor::Two => register_value * 2,
                    ScaleFactor::Four => register_value * 4,
                    ScaleFactor::Eight => register_value * 8,
                };

                let displacement = Cpu::get_byte_from_word(extension_word);
                let displacement_long = Cpu::sign_extend_byte(displacement);

                let areg_address = reg.get_a_reg_long(ea_register, step_log);

                let (ea_address, _) = areg_address.overflowing_add(displacement_long);
                let (ea_address, _) = ea_address.overflowing_add(register_value);

                EffectiveAddressingMode::ARegIndirectWithIndexOrMemoryIndirect {
                    ea_register,
                    ea_address,
                    extension_word,
                    displacement,
                    register_type,
                    register,
                    index_size,
                    scale_factor,
                }
            }
            0b111 => match ea_register {
                0b010 => {
                    let displacement = self.fetch_next_word(mem);
                    let ea_address = Cpu::get_address_with_word_displacement_sign_extended(
                        reg.reg_pc.get_address() + 2,
                        displacement,
                    );
                    EffectiveAddressingMode::PcIndirectWithDisplacement {
                        ea_address,
                        displacement,
                    }
                }
                0b011 => {
                    let extension_word = self.fetch_next_word(mem);
                    let extension_word_format = match extension_word & 0x0100 {
                        0x0100 => 'F', // full
                        _ => 'B',      // brief
                    };
                    if extension_word_format == 'F' {
                        todo!("Full extension word format not implemented")
                    }

                    let register = Cpu::extract_register_index_from_bit_pos(extension_word, 12)?;
                    let (register_value, register_type) = match extension_word & 0x8000 {
                        0x8000 => (
                            reg.get_a_reg_long(register, step_log),
                            RegisterType::Address,
                        ),
                        _ => (reg.get_d_reg_long(register, step_log), RegisterType::Data),
                    };
                    let (register_value, index_size) = match extension_word & 0x0800 {
                        0x0800 => (register_value, OperationSize::Long),
                        _ => (
                            Cpu::sign_extend_word(Cpu::get_word_from_long(register_value)),
                            OperationSize::Word,
                        ),
                    };
                    let scale_factor = Cpu::extract_scale_factor_from_bit_pos(extension_word, 9);

                    let register_value = match scale_factor {
                        ScaleFactor::One => register_value,
                        ScaleFactor::Two => register_value * 2,
                        ScaleFactor::Four => register_value * 4,
                        ScaleFactor::Eight => register_value * 8,
                    };

                    let displacement = Cpu::get_byte_from_word(extension_word);
                    let displacement_long = Cpu::sign_extend_byte(displacement);

                    let (pc_address, _) = reg.reg_pc.get_address().overflowing_add(2);

                    let (ea_address, _) = pc_address.overflowing_add(displacement_long);
                    let (ea_address, _) = ea_address.overflowing_add(register_value);

                    EffectiveAddressingMode::PcIndirectWithIndexOrPcMemoryIndirect {
                        ea_register,
                        ea_address,
                        extension_word,
                        displacement,
                        register_type,
                        register,
                        index_size,
                        scale_factor,
                    }
                }
                0b000 => {
                    let displacement = self.fetch_next_word(mem);
                    let ea_address = Cpu::sign_extend_word(displacement);
                    EffectiveAddressingMode::AbsoluteShortAddressing {
                        ea_address,
                        displacement,
                    }
                }
                0b001 => {
                    let ea_address = self.fetch_next_long(mem);
                    EffectiveAddressingMode::AbsolutLongAddressing { ea_address }
                }
                0b100 => match operation_size {
                    OperationSize::Byte => {
                        self.skip_byte();
                        let data = self.fetch_next_byte(mem);
                        EffectiveAddressingMode::ImmediateDataByte { data }
                    }
                    OperationSize::Word => {
                        let data = self.fetch_next_word(mem);
                        EffectiveAddressingMode::ImmediateDataWord { data }
                    }
                    OperationSize::Long => {
                        let data = self.fetch_next_long(mem);
                        EffectiveAddressingMode::ImmediateDataLong { data }
                    }
                },
                _ => panic!("Unable to extract EffectiveAddressingMode"),
            },
            _ => panic!("Unable to extract EffectiveAddressingMode"),
        };
        Ok(EffectiveAddressingData::create(
            instr_word,
            operation_size,
            ea_mode,
        ))
    }
}

pub struct Register {
    reg_d: [u32; 8],
    reg_a: [u32; 7],
    reg_usp: u32,
    reg_ssp: u32,
    pub reg_sr: StatusRegister,
    pub reg_pc: ProgramCounter,
}

impl Register {
    pub fn new() -> Register {
        let register = Register {
            reg_d: [0x00000000; 8],
            reg_a: [0x00000000; 7],
            reg_usp: 0x00000000,
            reg_ssp: 0x00000000,
            reg_sr: StatusRegister::from_word(STATUS_REGISTER_MASK_SUPERVISOR_STATE),
            reg_pc: ProgramCounter::from_address(0x00000000),
        };
        register
    }

    pub fn get_d_reg_long(&self, reg_index: usize, step_log: &mut StepLog) -> u32 {
        let value = self.reg_d[reg_index];
        step_log.add_log_entry(StepLogEntry::ReadRegister {
            reg_type: RegisterType::Data,
            reg_index,
            value,
        });
        value
    }

    pub fn get_d_reg_long_no_log(&self, reg_index: usize) -> u32 {
        let value = self.reg_d[reg_index];
        value
    }

    pub fn get_d_reg_word(&self, reg_index: usize, step_log: &mut StepLog) -> u16 {
        let value = self.reg_d[reg_index];
        step_log.add_log_entry(StepLogEntry::ReadRegister {
            reg_type: RegisterType::Data,
            reg_index,
            value,
        });
        Cpu::get_word_from_long(value)
    }

    pub fn get_d_reg_word_no_log(&self, reg_index: usize) -> u16 {
        let value = self.reg_d[reg_index];
        Cpu::get_word_from_long(value)
    }

    pub fn get_d_reg_byte(&self, reg_index: usize, step_log: &mut StepLog) -> u8 {
        let value = self.reg_d[reg_index];
        step_log.add_log_entry(StepLogEntry::ReadRegister {
            reg_type: RegisterType::Data,
            reg_index,
            value,
        });
        Cpu::get_byte_from_long(value)
    }

    pub fn get_d_reg_byte_no_log(&self, reg_index: usize) -> u8 {
        let value = self.reg_d[reg_index];
        Cpu::get_byte_from_long(value)
    }

    pub fn get_a_reg_long(&self, reg_index: usize, step_log: &mut StepLog) -> u32 {
        let value = match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => self.get_ssp_reg(),
                false => self.get_usp_reg(),
            },
            _ => self.reg_a[reg_index],
        };
        step_log.add_log_entry(StepLogEntry::ReadRegister {
            reg_type: RegisterType::Address,
            reg_index,
            value,
        });
        value
    }

    pub fn get_a_reg_long_no_log(&self, reg_index: usize) -> u32 {
        let value = match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => self.get_ssp_reg(),
                false => self.get_usp_reg(),
            },
            _ => self.reg_a[reg_index],
        };
        value
    }

    pub fn get_a_reg_word(&self, reg_index: usize, step_log: &mut StepLog) -> u16 {
        Cpu::get_word_from_long(self.get_a_reg_long(reg_index, step_log))
    }

    pub fn get_a_reg_word_no_log(&self, reg_index: usize) -> u16 {
        Cpu::get_word_from_long(self.get_a_reg_long_no_log(reg_index))
    }

    pub fn get_a_reg_byte(&self, reg_index: usize, step_log: &mut StepLog) -> u8 {
        Cpu::get_byte_from_long(self.get_a_reg_long(reg_index, step_log))
    }

    pub fn get_a_reg_byte_no_log(&self, reg_index: usize) -> u8 {
        Cpu::get_byte_from_long(self.get_a_reg_long_no_log(reg_index))
    }

    pub fn increment_a_reg(
        &mut self,
        reg_index: usize,
        step_log: &mut StepLog,
        operation_size: OperationSize,
    ) {
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => {
                    let value = self.reg_ssp + operation_size.size_in_bytes();
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_ssp = value;
                }
                false => {
                    let value = self.reg_usp + operation_size.size_in_bytes();
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_usp = value;
                }
            },
            _ => {
                let value = self.reg_a[reg_index] + operation_size.size_in_bytes();
                step_log.add_log_entry(StepLogEntry::WriteRegister {
                    reg_type: RegisterType::Address,
                    reg_index,
                    value,
                });
                self.reg_a[reg_index] = value;
            }
        }
    }

    pub fn decrement_a_reg(
        &mut self,
        reg_index: usize,
        step_log: &mut StepLog,
        operation_size: OperationSize,
    ) {
        // TODO: Maybe have a StepLogEntry::DecrementARegister that logs before and after values
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => {
                    let value = self.reg_ssp - operation_size.size_in_bytes();
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_ssp = value;
                }
                false => {
                    let value = self.reg_usp - operation_size.size_in_bytes();
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_usp = value;
                }
            },
            _ => {
                let value = self.reg_a[reg_index] - operation_size.size_in_bytes();
                step_log.add_log_entry(StepLogEntry::WriteRegister {
                    reg_type: RegisterType::Address,
                    reg_index,
                    value,
                });
                self.reg_a[reg_index] = value;
            }
        }
    }

    pub fn set_d_reg_long(&mut self, step_log: &mut StepLog, reg_index: usize, value: u32) {
        step_log.add_log_entry(StepLogEntry::WriteRegister {
            reg_type: RegisterType::Data,
            reg_index,
            value,
        });
        self.reg_d[reg_index] = value;
    }

    pub fn set_d_reg_long_no_log(&mut self, reg_index: usize, value: u32) {
        self.reg_d[reg_index] = value;
    }

    pub fn set_d_reg_word(&mut self, step_log: &mut StepLog, reg_index: usize, value: u16) {
        let value = Cpu::set_word_in_long(value, self.reg_d[reg_index]);
        step_log.add_log_entry(StepLogEntry::WriteRegister {
            reg_type: RegisterType::Data,
            reg_index,
            value,
        });
        self.reg_d[reg_index] = value;
    }

    pub fn set_d_reg_word_no_log(&mut self, reg_index: usize, value: u16) {
        self.reg_d[reg_index] = Cpu::set_word_in_long(value, self.reg_d[reg_index]);
    }

    pub fn set_d_reg_byte(&mut self, step_log: &mut StepLog, reg_index: usize, value: u8) {
        let value = Cpu::set_byte_in_long(value, self.reg_d[reg_index]);
        step_log.add_log_entry(StepLogEntry::WriteRegister {
            reg_type: RegisterType::Data,
            reg_index,
            value,
        });
        self.reg_d[reg_index] = value;
    }

    pub fn set_d_reg_byte_no_log(&mut self, reg_index: usize, value: u8) {
        self.reg_d[reg_index] = Cpu::set_byte_in_long(value, self.reg_d[reg_index]);
    }

    pub fn set_a_reg_long(&mut self, step_log: &mut StepLog, reg_index: usize, value: u32) {
        step_log.add_log_entry(StepLogEntry::WriteRegister {
            reg_type: RegisterType::Address,
            reg_index,
            value,
        });
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => self.set_ssp_reg(value),
                false => self.set_usp_reg(value),
            },
            _ => self.reg_a[reg_index] = value,
        };
    }

    pub fn set_a_reg_long_no_log(&mut self, reg_index: usize, value: u32) {
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => self.set_ssp_reg(value),
                false => self.set_usp_reg(value),
            },
            _ => self.reg_a[reg_index] = value,
        };
    }

    pub fn set_a_reg_word(&mut self, step_log: &mut StepLog, reg_index: usize, value: u16) {
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => {
                    let value = Cpu::set_word_in_long(value, self.reg_ssp);
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_ssp = value;
                }
                false => {
                    let value = Cpu::set_word_in_long(value, self.reg_usp);
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_usp = value;
                }
            },
            _ => {
                let value = Cpu::set_word_in_long(value, self.reg_a[reg_index]);
                step_log.add_log_entry(StepLogEntry::WriteRegister {
                    reg_type: RegisterType::Address,
                    reg_index,
                    value,
                });
                self.reg_a[reg_index] = value;
            }
        };
    }

    pub fn set_a_reg_word_no_log(&mut self, reg_index: usize, value: u16) {
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => self.reg_ssp = Cpu::set_word_in_long(value, self.reg_ssp),
                false => self.reg_usp = Cpu::set_word_in_long(value, self.reg_usp),
            },
            _ => self.reg_a[reg_index] = Cpu::set_word_in_long(value, self.reg_a[reg_index]),
        };
    }

    pub fn set_a_reg_byte(&mut self, step_log: &mut StepLog, reg_index: usize, value: u8) {
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => {
                    let value = Cpu::set_byte_in_long(value, self.reg_ssp);
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_ssp = value;
                }
                false => {
                    let value = Cpu::set_byte_in_long(value, self.reg_usp);
                    step_log.add_log_entry(StepLogEntry::WriteRegister {
                        reg_type: RegisterType::Address,
                        reg_index,
                        value,
                    });
                    self.reg_usp = value;
                }
            },
            _ => {
                let value = Cpu::set_byte_in_long(value, self.reg_a[reg_index]);
                step_log.add_log_entry(StepLogEntry::WriteRegister {
                    reg_type: RegisterType::Address,
                    reg_index,
                    value,
                });
                self.reg_a[reg_index] = value;
            }
        };
    }

    pub fn set_a_reg_byte_no_log(&mut self, reg_index: usize, value: u8) {
        match reg_index {
            7 => match self.reg_sr.is_sr_supervisor_set_no_log() {
                true => self.reg_ssp = Cpu::set_byte_in_long(value, self.reg_ssp),
                false => self.reg_usp = Cpu::set_byte_in_long(value, self.reg_usp),
            },
            _ => self.reg_a[reg_index] = Cpu::set_byte_in_long(value, self.reg_a[reg_index]),
        };
    }

    pub fn get_ssp_reg(&self) -> u32 {
        // println!(
        //     "get_ssp_reg: [SSP_Access] [Read ____]  ${:08X}",
        //     self.reg_ssp
        // );
        self.reg_ssp
    }

    pub fn set_ssp_reg(&mut self, value: u32) {
        // println!(
        //     "set_ssp_reg: [SSP_Access] [Write ____] ${:08X} [from: ${:08X}]",
        //     value, self.reg_ssp
        // );
        self.reg_ssp = value;
    }

    pub fn get_usp_reg(&self) -> u32 {
        // println!(
        //     "get_usp_reg: [USP_Access] [Write ____] ${:08X}",
        //     self.reg_usp
        // );
        self.reg_usp
    }

    pub fn set_usp_reg(&mut self, value: u32) {
        // println!(
        //     "set_usp_reg: [USP_Access] [Write ____] ${:08X} [from: ${:08X}]",
        //     value, self.reg_usp
        // );
        self.reg_usp = value;
    }

    pub fn stack_push_pc(&mut self, mem: &mut Mem, step_log: &mut StepLog) {
        let pc = self.reg_pc.address;
        self.decrement_a_reg(7, step_log, OperationSize::Long);
        let sp = self.get_a_reg_long(7, step_log);
        mem.set_long(step_log, sp, pc);
    }

    pub fn stack_pop_pc(&mut self, mem: &mut Mem, pc: &mut ProgramCounter, step_log: &mut StepLog) {
        let sp = self.get_a_reg_long(7, step_log);
        let pc_address = mem.get_long(step_log, sp);
        self.increment_a_reg(7, step_log, OperationSize::Long);
        pc.address_jump = Some(pc_address);
    }

    pub fn stack_push_word(&mut self, mem: &mut Mem, step_log: &mut StepLog, value: u16) {
        self.decrement_a_reg(7, step_log, OperationSize::Word);
        let sp = self.get_a_reg_long(7, step_log);
        mem.set_word(step_log, sp, value);
    }

    pub fn stack_pop_word(&mut self, mem: &mut Mem, step_log: &mut StepLog) -> u16 {
        let sp = self.get_a_reg_long(7, step_log);
        let result = mem.get_word(step_log, sp);
        self.increment_a_reg(7, step_log, OperationSize::Word);
        result
    }

    pub fn stack_push_long(&mut self, mem: &mut Mem, step_log: &mut StepLog, value: u32) {
        self.decrement_a_reg(7, step_log, OperationSize::Long);
        let sp = self.get_a_reg_long(7, step_log);
        mem.set_long(step_log, sp, value);
    }

    pub fn stack_pop_long(&mut self, mem: &mut Mem, step_log: &mut StepLog) -> u32 {
        let sp = self.get_a_reg_long(7, step_log);
        let result = mem.get_long(step_log, sp);
        self.increment_a_reg(7, step_log, OperationSize::Long);
        result
    }

    pub fn print_registers(&self) {
        for n in 0..8 {
            print!(" D{} ${:08X}", n, self.reg_d[n]);
        }
        println!();
        for n in 0..7 {
            print!(" A{} ${:08X}", n, self.reg_a[n]);
        }
        match self.reg_sr.is_sr_supervisor_set_no_log() {
            true => print!(" A7 ${:08X}", self.reg_ssp),
            false => print!(" A7 ${:08X}", self.reg_usp),
        }
        println!();
        print!(" USP ${:08X} ", self.reg_usp);
        print!(" SSP ${:08X} ", self.reg_ssp);
        println!();
        print!(" SR ${:04X} {}", self.reg_sr.get_value(), self.reg_sr);

        println!();
        print!(" PC ${:08X}", self.reg_pc.get_address());
        println!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StatusRegister {
    reg_sr: u16,
}

impl StatusRegister {
    pub fn from_empty() -> StatusRegister {
        StatusRegister { reg_sr: 0x0000 }
    }

    pub fn from_word(reg_sr: u16) -> StatusRegister {
        StatusRegister { reg_sr }
    }

    pub fn get_value(&self) -> u16 {
        self.reg_sr
    }

    pub fn set_value(&mut self, reg_sr: u16) {
        self.reg_sr = reg_sr;
    }

    pub fn get_sr_reg_flags_abcde(&self) -> u16 {
        self.reg_sr & 0b0000000000011111
    }

    pub fn set_sr_reg_flags_abcde(&mut self, value: u16) {
        self.reg_sr = (self.reg_sr & 0b1111111111100000) | value;
    }

    pub fn merge_status_register(
        &mut self,
        step_log: &mut StepLog,
        status_register_result: StatusRegisterResult,
    ) {
        let new_sr = (self.reg_sr & !status_register_result.status_register_mask)
            | (status_register_result.status_register
                & status_register_result.status_register_mask);
        if new_sr != self.reg_sr {
            step_log.add_log_entry(StepLogEntry::SrChanged {
                value: new_sr,
                value_old: self.reg_sr,
            });
        } else {
            step_log.add_log_entry(StepLogEntry::SrNotChanged { value: new_sr });
        }
        self.reg_sr = new_sr;
    }

    // pub fn clear_carry(&mut self) {
    //     self.reg_sr &= !STATUS_REGISTER_MASK_CARRY;
    // }

    // pub fn set_carry(&mut self) {
    //     self.reg_sr |= STATUS_REGISTER_MASK_CARRY;
    // }

    // pub fn clear_overflow(&mut self) {
    //     self.reg_sr &= !STATUS_REGISTER_MASK_OVERFLOW;
    // }

    // pub fn set_overflow(&mut self) {
    //     self.reg_sr |= STATUS_REGISTER_MASK_OVERFLOW;
    // }

    // pub fn clear_zero(&mut self) {
    //     self.reg_sr &= !STATUS_REGISTER_MASK_ZERO;
    // }

    // pub fn set_zero(&mut self) {
    //     self.reg_sr |= STATUS_REGISTER_MASK_ZERO;
    // }

    // pub fn clear_negative(&mut self) {
    //     self.reg_sr &= !STATUS_REGISTER_MASK_NEGATIVE;
    // }

    // pub fn set_negative(&mut self) {
    //     self.reg_sr |= STATUS_REGISTER_MASK_NEGATIVE;
    // }

    // pub fn clear_extend(&mut self) {
    //     self.reg_sr &= !STATUS_REGISTER_MASK_EXTEND;
    // }

    // pub fn set_extend(&mut self) {
    //     self.reg_sr |= STATUS_REGISTER_MASK_EXTEND;
    // }

    pub fn clear_supervisor(&mut self) {
        self.reg_sr &= !STATUS_REGISTER_MASK_SUPERVISOR_STATE;
    }

    pub fn set_supervisor(&mut self) {
        self.reg_sr |= STATUS_REGISTER_MASK_SUPERVISOR_STATE;
    }

    pub fn is_sr_carry_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_CARRY) == STATUS_REGISTER_MASK_CARRY;
    }

    pub fn is_sr_overflow_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_OVERFLOW) == STATUS_REGISTER_MASK_OVERFLOW;
    }

    pub fn is_sr_zero_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_ZERO) == STATUS_REGISTER_MASK_ZERO;
    }

    pub fn is_sr_negative_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_NEGATIVE) == STATUS_REGISTER_MASK_NEGATIVE;
    }

    pub fn is_sr_extend_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_EXTEND) == STATUS_REGISTER_MASK_EXTEND;
    }

    pub fn is_sr_supervisor_set(&self, step_log: &mut StepLog) -> bool {
        let value = (self.reg_sr & STATUS_REGISTER_MASK_SUPERVISOR_STATE)
            == STATUS_REGISTER_MASK_SUPERVISOR_STATE;
        step_log.add_log_entry(StepLogEntry::SrReadSupervisorBit { value });
        value
    }

    pub fn is_sr_supervisor_set_no_log(&self) -> bool {
        let value = (self.reg_sr & STATUS_REGISTER_MASK_SUPERVISOR_STATE)
            == STATUS_REGISTER_MASK_SUPERVISOR_STATE;
        value
    }

    pub fn evaluate_condition(&self, conditional_test: &ConditionalTest) -> bool {
        match conditional_test {
            ConditionalTest::T => true,

            ConditionalTest::CC => self.reg_sr & STATUS_REGISTER_MASK_CARRY == 0x0000,
            ConditionalTest::CS => self.reg_sr & STATUS_REGISTER_MASK_CARRY != 0x0000,
            ConditionalTest::EQ => self.reg_sr & STATUS_REGISTER_MASK_ZERO != 0x0000,
            ConditionalTest::F => false,
            ConditionalTest::GE => {
                let ge_mask = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW;
                let sr = self.reg_sr & ge_mask;
                sr == ge_mask || sr == 0x0000
                // (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                //     && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000)
            }
            ConditionalTest::GT => {
                let gt_mask = STATUS_REGISTER_MASK_NEGATIVE
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO;
                let sr = self.reg_sr & gt_mask;
                sr == STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW || sr == 0x0000
                // (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                //     && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000
                //     && reg.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000)
            }
            ConditionalTest::HI => {
                self.reg_sr & STATUS_REGISTER_MASK_CARRY == 0x0000
                    && self.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000
            }
            ConditionalTest::LE => {
                let le_mask = STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE
                    | STATUS_REGISTER_MASK_OVERFLOW;
                let sr = self.reg_sr & le_mask;
                sr == STATUS_REGISTER_MASK_ZERO
                    || sr == STATUS_REGISTER_MASK_NEGATIVE
                    || sr == STATUS_REGISTER_MASK_OVERFLOW
                // (reg.reg_sr & STATUS_REGISTER_MASK_ZERO != 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000)
            }
            ConditionalTest::LS => {
                (self.reg_sr & STATUS_REGISTER_MASK_CARRY != 0x0000)
                    || (self.reg_sr & STATUS_REGISTER_MASK_ZERO != 0x0000)
            }
            ConditionalTest::LT => {
                (self.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                    && self.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000)
                    || (self.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                        && self.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000)
            }
            ConditionalTest::MI => self.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000,
            ConditionalTest::NE => self.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000,
            ConditionalTest::PL => self.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000,
            ConditionalTest::VC => self.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000,
            ConditionalTest::VS => self.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000,
        }
    }
}

impl fmt::Display for StatusRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_sr_supervisor_set_no_log() {
            write!(f, "S");
        } else {
            write!(f, "-");
        }
        write!(f, " ");
        if self.is_sr_extend_set() {
            write!(f, "X");
        } else {
            write!(f, "-");
        }
        if self.is_sr_negative_set() {
            write!(f, "N");
        } else {
            write!(f, "-");
        }
        if self.is_sr_zero_set() {
            write!(f, "Z");
        } else {
            write!(f, "-");
        }
        if self.is_sr_overflow_set() {
            write!(f, "V");
        } else {
            write!(f, "-");
        }
        if self.is_sr_carry_set() {
            write!(f, "C");
        } else {
            write!(f, "-");
        }
        Ok(())
    }
}
