// #[test]
//     fn immediate_data_to_data_register_direct_word() {
//         // arrange
//         let code = [0x06, 0x47, 0x12, 0x34].to_vec(); // ADD.W #$1234,D7
//         let mut cpu = crate::instr_test_setup(code, None);
//         cpu.register.reg_d[7] = 0x00004321;
//         cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
//             | STATUS_REGISTER_MASK_OVERFLOW
//             | STATUS_REGISTER_MASK_ZERO
//             | STATUS_REGISTER_MASK_NEGATIVE
//             | STATUS_REGISTER_MASK_EXTEND;
//         // act assert - debug
//         let debug_result = cpu.get_next_disassembly();
//         assert_eq!(
//             DisassemblyResult::from_address_and_address_next(
//                 0xC00000,
//                 0xC00002,
//                 String::from("ADD.W"),
//                 String::from("#$1234,D7")
//             ),
//             debug_result
//         );
//         // act
//         cpu.execute_next_instruction();
//         // assert
//         assert_eq!(0x5555, cpu.register.reg_d[7]);
//         assert_eq!(false, cpu.register.is_sr_carry_set());
//         assert_eq!(false, cpu.register.is_sr_coverflow_set());
//         assert_eq!(false, cpu.register.is_sr_zero_set());
//         assert_eq!(false, cpu.register.is_sr_negative_set());
//         assert_eq!(true, cpu.register.is_sr_extend_set());
//     }
