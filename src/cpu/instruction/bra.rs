use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: DONE
// step cc: DONE (not affected)
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let displacement = Cpu::get_byte_from_word(instr_word);

    let result = match displacement {
        0x00 => {
            let displacement = pc.fetch_next_word(mem);
            pc.branch_word(displacement);
        }
        0xff => {
            let displacement = pc.fetch_next_long(mem);
            pc.branch_long(displacement);
        }
        _ => {
            pc.branch_byte(displacement);
        }
    };
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let displacement = Cpu::get_byte_from_word(instr_word);

    match displacement {
        0x00 => {
            let displacement = pc.fetch_next_word(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("BRA.W"),
                format!(
                    "${:04X} [${:08X}]",
                    displacement,
                    pc.get_branch_word_address(displacement)
                ),
            ))
        }
        0xff => {
            let displacement = pc.fetch_next_long(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("BRA.L"),
                format!(
                    "${:08X} [${:08X}]",
                    displacement,
                    pc.get_branch_long_address(displacement)
                ),
            ))
        }
        _ => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("BRA.B"),
            format!(
                "${:02X} [${:08X}]",
                displacement,
                pc.get_branch_byte_address(displacement)
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn bra_byte_positive() {
        // arrange
        let code = [0x60, 0x02].to_vec(); // BRA.B $02
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BRA.B"),
                String::from("$02 [$00C00004]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
        assert_eq!(0x0000, cpu.register.reg_sr.get_sr_reg_flags_abcde());
    }

    #[test]
    fn bra_byte_negative() {
        // arrange
        let code = [0x60, 0xfa].to_vec(); // BRA.B $FA
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BRA.B"),
                String::from("$FA [$00BFFFFC]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00BFFFFC, cpu.register.reg_pc.get_address());
        assert_eq!(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_EXTEND,
            cpu.register.reg_sr.get_sr_reg_flags_abcde()
        );
    }

    #[test]
    fn bra_word_positive() {
        // arrange
        let code = [0x60, 0x00, 0x00, 0x08].to_vec(); // BRA.B $08
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BRA.W"),
                String::from("$0008 [$00C0000A]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xC0000A, cpu.register.reg_pc.get_address());
        assert_eq!(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_EXTEND,
            cpu.register.reg_sr.get_sr_reg_flags_abcde()
        );
    }

    #[test]
    fn bra_word_negative() {
        // arrange
        let code = [0x60, 0x00, 0xff, 0xf8].to_vec(); // BRA.B $FFF8
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BRA.W"),
                String::from("$FFF8 [$00BFFFFA]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00BFFFFA, cpu.register.reg_pc.get_address());
        assert_eq!(0x0000, cpu.register.reg_sr.get_sr_reg_flags_abcde());
    }

    #[test]
    fn bra_long_positive() {
        // arrange
        let code = [0x60, 0xff, 0x00, 0x00, 0x00, 0x0C].to_vec(); // BRA.B $0000000C
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BRA.L"),
                String::from("$0000000C [$00C0000E]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xC0000E, cpu.register.reg_pc.get_address());
        assert_eq!(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_EXTEND,
            cpu.register.reg_sr.get_sr_reg_flags_abcde()
        );
    }

    #[test]
    fn bra_long_negative() {
        // arrange
        let code = [0x60, 0xff, 0xff, 0xff, 0xff, 0xf6].to_vec(); // BRA.B $FFFFFFF6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BRA.L"),
                String::from("$FFFFFFF6 [$00BFFFF8]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00BFFFF8, cpu.register.reg_pc.get_address());
        assert_eq!(0x0000, cpu.register.reg_sr.get_sr_reg_flags_abcde());
    }
}
