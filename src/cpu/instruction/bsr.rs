use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::{step_log::StepLog, Cpu},
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
    step_log: &mut StepLog,
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
    reg.stack_push_long(mem, step_log, pc.get_address_next());
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let displacement = Cpu::get_byte_from_word(instr_word);

    match displacement {
        0x00 => {
            let displacement = pc.fetch_next_word(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                mem,
                String::from("BSR.W"),
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
                mem,
                String::from("BSR.L"),
                format!(
                    "${:08X} [${:08X}]",
                    displacement,
                    pc.get_branch_long_address(displacement)
                ),
            ))
        }
        _ => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("BSR.B"),
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
    use crate::cpu::instruction::GetDisassemblyResult;

    // byte

    #[test]
    fn step_bsr_byte() {
        // arrange
        let code = [0x61, 0x02].to_vec(); // BSR.B $02
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BSR.B"),
                String::from("$02 [$00C00004]"),
                vec![0x6102]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC00004, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00002, mm.mem.get_long_no_log(0x10003fc));
    }

    #[test]
    fn step_bsr_byte_negative() {
        // arrange
        let code = [0x61, 0xfc].to_vec(); // BSR.B $FC
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        println!("sp: ${:08X}", mm.cpu.register.get_a_reg_long_no_log(7));

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BSR.B"),
                String::from("$FC [$00BFFFFE]"),
                vec![0x61fc]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xBFFFFE, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00002, mm.mem.get_long_no_log(0x10003fc));
    }

    // word

    #[test]
    fn step_bsr_word() {
        // arrange
        let code = [0x61, 0x00, 0x00, 0x04].to_vec(); // BSR.W $0004
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        println!("sp: ${:08X}", mm.cpu.register.get_a_reg_long_no_log(7));

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BSR.W"),
                String::from("$0004 [$00C00006]"),
                vec![0x6100, 0x0004]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC00006, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00004, mm.mem.get_long_no_log(0x10003fc));
    }

    #[test]
    fn step_bsr_word_negative() {
        // arrange
        let code = [0x61, 0x00, 0xff, 0xfc].to_vec(); // BSR.W $FFFC
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        println!("sp: ${:08X}", mm.cpu.register.get_a_reg_long_no_log(7));

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BSR.W"),
                String::from("$FFFC [$00BFFFFE]"),
                vec![0x6100, 0xfffc]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xBFFFFE, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00004, mm.mem.get_long_no_log(0x10003fc));
    }

    // long

    #[test]
    fn step_bsr_long() {
        // arrange
        let code = [0x61, 0xff, 0x00, 0x00, 0x00, 0x06].to_vec(); // BSR.L $00000006
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        println!("sp: ${:08X}", mm.cpu.register.get_a_reg_long_no_log(7));

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BSR.L"),
                String::from("$00000006 [$00C00008]"),
                vec![0x61ff, 0x0000, 0x0006]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC00008, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00006, mm.mem.get_long_no_log(0x10003fc));
    }

    #[test]
    fn step_bsr_long_negative() {
        // arrange
        let code = [0x61, 0xff, 0xff, 0xff, 0xff, 0xfc].to_vec(); // BSR.L $FFFFFFFC
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        println!("sp: ${:08X}", mm.cpu.register.get_a_reg_long_no_log(7));

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BSR.L"),
                String::from("$FFFFFFFC [$00BFFFFE]"),
                vec![0x61ff, 0xffff, 0xfffc]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xBFFFFE, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00006, mm.mem.get_long_no_log(0x10003fc));
    }
}
