use super::{
    EffectiveAddressingMode, GetDisassemblyResult, GetDisassemblyResultError, OperationSize,
    StepError, StepResult,
};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register},
};

// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<StepResult, StepError> {
    // let instr_word = pc.peek_next_word(mem);
    // let size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;
    // let ea_data =
    //     pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, Some(size))?;

    // // todo!();
    // let status_register_result = match size {
    //     // OperationSize::Byte => ea_data.set_value_byte(pc, reg, mem, 0x00, true),
    //     // OperationSize::Word => ea_data.set_value_word(pc, reg, mem, 0x0000, true),
    //     OperationSize::Long => {
    //         let value = ea_data.get_value_long(pc, reg, mem, false);

    //         let result = Cpu::not_long(value);
    //         println!("value after not:  ${:08X}", result.result);
    //         ea_data.set_value_long(pc, reg, mem, value, true);
    //         result.status_register_result
    //     }
    //     _ => todo!(),
    // };

    // reg.reg_sr = status_register_result.merge_status_register(reg.reg_sr);

    // Ok(StepResult::Done {})
    todo!()
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            match instr_word & 0x0038 {
                0x0000 => {
                    // DRegDirect
                    Ok(OperationSize::Long)
                }
                _ => {
                    // other
                    Ok(OperationSize::Byte)
                }
            }
        })?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(OperationSize::Byte), reg, mem);

    match ea_data.instr_word & 0x0100 {
        0x0100 => {
            // Bit Number Dynamic, Specified in a Register
            println!("Bit Number Dynamic, Specified in a Register");
            todo!();
        }
        _ => {
            // Bit Number Static, Specified as Immediate Data
            println!("Bit Number Static, Specified as Immediate Data");
            let bit_number = match ea_data.operation_size {
                OperationSize::Long => pc.fetch_next_word(mem) % 32,
                _ => pc.fetch_next_word(mem) % 8,
            };
            println!("bit number: {}", bit_number);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from(format!("BTST.{}", ea_data.operation_size.get_format())),
                format!("#${:02X},{}", bit_number, ea_format.format),
            ))
        }
    }
}
