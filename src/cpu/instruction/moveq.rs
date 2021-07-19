use crate::{cpu::Cpu, cpu::instruction::{OperationSize, PcResult}};
use crate::mem::Mem;
use crate::register::{Register, STATUS_REGISTER_MASK_ZERO};
use byteorder::ReadBytesExt;

use super::InstructionExecutionResult;


pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem<'a>,
) -> InstructionExecutionResult {
    // TODO: Condition codes
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
    let operand = instr_bytes.read_i8().unwrap();
    let mut status_register_flags = 0x0000;
    match operand {
        0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
        i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
        _ => (),
    }
    let operand = Cpu::sign_extend_i8(operand);
    let operands_format = format!("#{},D{}", operand, register);
    let instr_comment = format!("moving {:#010x} into D{}", operand, register);
    let status_register_mask = 0xfff0;

    reg.reg_d[register] = operand;
    reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;
    InstructionExecutionResult {
        name: String::from("MOVEQ"),
        operands_format: operands_format,
        comment: instr_comment,
        op_size: OperationSize::Long,
        pc_result: PcResult::Increment(2),
    }
}


#[cfg(test)]
mod tests {
    use crate::{cpu::Cpu};
    use crate::memrange::MemRange;
    use crate::mem::{Mem};

    #[test]
    fn step() {
        // TODO: This is way too messy! Too much code required to setup simple test.
        // TODO: Would be nice to not need the rom cheat
        let rom_cheat =
        MemRange::from_file(0x000000, 512 * 1024, "D:\\Amiga\\ROM\\Kickstart 3.1.rom")
            .unwrap();
        let code: [u8; 2] = [0x70, 0x1d];
        let code = code.to_vec();
        let mut mem_ranges = Vec::new();
        let code = MemRange::from_xxx(0x080000, code);
        mem_ranges.push(&rom_cheat);
        mem_ranges.push(&code);
        let mem = Mem::new(mem_ranges);
        let mut cpu = Cpu::new(mem);
        cpu.register.reg_pc = 0x080000;

        cpu.execute_next_instruction();
        assert_eq!(0x1d, cpu.register.reg_d[0]);
    }
}