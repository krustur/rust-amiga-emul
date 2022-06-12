use crate::{
    cpu::{
        instruction::{DisassemblyResult, PcResult},
        Cpu,
    },
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::InstructionExecutionResult;

// Instruction State
// =================
// step-logic: TODO
// step cc: TODO (none)
// step tests: TODO
// get_disassembly: TODO
// get_disassembly tests: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    let instr_word = pc.fetch_next_unsigned_word(mem);
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition_result = Cpu::evaluate_condition(reg, &conditional_test);
    let displacement_16bit = pc.fetch_next_signed_word(mem);

    let result = match condition_result {
        false => {
            let register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
            let reg_word = reg.reg_d[register].wrapping_sub(1) & 0xffff;
            reg.reg_d[register] = (reg.reg_d[register] & 0xffff0000) | reg_word;

            match reg.reg_d[register] & 0xffff {
                0x0000ffff => {
                    InstructionExecutionResult::Done {
                        // -1
                        pc_result: PcResult::Increment,
                    }
                }
                _ => {
                    let branch_to = Cpu::get_address_with_i16_displacement(
                        pc.get_address() + 2,
                        displacement_16bit,
                    );
                    InstructionExecutionResult::Done {
                        pc_result: PcResult::Set(branch_to),
                    }
                }
            }
        }
        true => InstructionExecutionResult::Done {
            pc_result: PcResult::Increment,
        },
    };

    result
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    let instr_word = pc.fetch_next_unsigned_word(mem);
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word);

    let displacement_16bit = pc.fetch_next_signed_word(mem);

    let branch_to =
        Cpu::get_address_with_i16_displacement(pc.get_address() + 2, displacement_16bit);

    let result = DisassemblyResult::from_pc(
        pc,
        format!("DB{:?}", conditional_test),
        format!(
            "D{},${:04X} [${:08X}]",
            register, displacement_16bit, branch_to
        ),
    );

    result
}

#[cfg(test)]
mod tests {
    use crate::{cpu::instruction::DisassemblyResult, register::STATUS_REGISTER_MASK_CARRY};

    // DBCC C set/reg gt 0 => decrease reg and branch (not -1)
    // DBCC C set/reg eq 0 => decrease reg and no branch (-1)
    // DBCC C clear => do nothing

    #[test]
    fn step_dbcc_when_carry_set_and_reg_greater_than_zero() {
        // arrange
        let code = [0x54, 0xc8, 0x00, 0x04].to_vec(); // DBCC D0,0x0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0xffff0001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D0,$0004 [$00C00006]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff0000, cpu.register.reg_d[0]);
        assert_eq!(0xC00006, cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_dbcc_when_carry_set_and_reg_equal_to_zero() {
        // arrange
        let code = [0x54, 0xc9, 0x00, 0x04].to_vec(); // DBCC D1,0x0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[1] = 0xffff0000;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D1,$0004 [$00C00006]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffffffff, cpu.register.reg_d[1]);
        assert_eq!(0xc00004, cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_dbcc_when_carry_clear() {
        // arrange
        let code = [0x54, 0xca, 0x00, 0x04].to_vec(); // DBCC D2,0x0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0xffff0001;
        cpu.register.reg_sr = 0x0000; //STATUS_REGISTER_MASK_CARRY;
                                      // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D2,$0004 [$00C00006]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff0001, cpu.register.reg_d[2]);
        assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
    }
}
