use super::{GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError};
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
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        |instr_word| Ok(OperationSize::Long),
    )?;

    let address = ea_data.get_address(pc, reg, mem);
    // println!("${:08X}", address);

    pc.jump_long(address);
    reg.stack_push_long(mem, pc.get_address_next());
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        |instr_word| Ok(OperationSize::Long),
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(ea_data.operation_size), mem);
    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("JSR"),
        ea_format.format,
    ))
}

#[cfg(test)]
mod tests {
    use crate::cpu::instruction::GetDisassemblyResult;

    #[test]
    fn jsr_address_register_indirect() {
        // arrange
        let code = [0x4e, 0x90].to_vec(); // JSR (A0)
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00c0c0f0);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("JSR"),
                String::from("(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c0c0f0, cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
        assert_eq!(0xC00002, cpu.memory.get_long(0x10003fc));
    }

    #[test]
    fn jsr_address_register_indirect_witrh() {
        // arrange
        let code = [0x4e, 0xae, 0xfd, 0x84].to_vec(); // JSR -636(A6)
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(6, 0x00c0c0f0);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("JSR"),
                String::from("($FD84,A6) [-636]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00C0BE74, cpu.register.reg_pc.get_address());
        assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
        assert_eq!(0xC00004, cpu.memory.get_long(0x10003fc));
    }
}
