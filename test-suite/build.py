import os
import re
import sys
import traceback
from enum import Enum


class Keyword(Enum):
    ARRANGE_MEM = 100
    ARRANGE_REG = 101
    ARRANGE_CODE = 102
    ASSERT_MEM = 200
    ASSERT_REG = 201
    ASSERT_CODE = 202
    ARRANGE_ASSERT_MEM = 300
    # ARRANGE_ASSERT_REG = 301


class ParsedLineType(Enum):
    BLANK = 1
    TEST_NAME = 2
    KEYWORD = 3
    ADDRESS_NULL_TERMINATION = 4
    ADDRESS_WITH_BYTES = 5
    DATA_REGISTERS = 6
    ADDRESS_REGISTERS = 7
    STATUS_REGISTER = 8
    PROGRAM_COUNTER_REGISTER = 9
    SOURCE_CODE = 10


class ParsedLine(object):
    line_type: ParsedLineType
    line_number: int
    line_raw: str

    test_name: str
    keyword: Keyword
    address: int
    bytes: list[int]
    data_registers: list[int]
    address_registers: list[int]
    status_flags: list[str]
    status_register: int
    program_counter: int
    source_code_instruction: str
    source_code_operands: str

    def __init__(self,
                 line_number: int,
                 line_raw: str,
                 is_blank_line: bool = False,
                 test_name: str = None,
                 keyword: Keyword = None,
                 address: int = None,
                 bytes: list[int] = None,
                 data_registers: list[int] = None,
                 address_registers: list[int] = None,
                 status_flags: list[str] = None,
                 status_register: int = None,
                 program_counter: int = None,
                 source_code_instruction: str = None,
                 source_code_operands: str = None):
        self.line_type = ParsedLineType.BLANK
        self.line_number = line_number
        self.line_raw = line_raw

        if is_blank_line:
            self.line_type = ParsedLineType.BLANK
        elif test_name:
            self.line_type = ParsedLineType.TEST_NAME
            self.test_name = test_name
        elif keyword:
            self.line_type = ParsedLineType.KEYWORD
            self.keyword = keyword
        elif address is not None and bytes is None:
            self.line_type = ParsedLineType.ADDRESS_NULL_TERMINATION
            self.address = address
        elif address is not None and bytes is not None:
            self.line_type = ParsedLineType.ADDRESS_WITH_BYTES
            self.address = address
            self.bytes = bytes
        elif data_registers:
            self.line_type = ParsedLineType.DATA_REGISTERS
            self.data_registers = data_registers
        elif address_registers:
            self.line_type = ParsedLineType.ADDRESS_REGISTERS
            self.address_registers = address_registers
        elif status_flags is not None:
            self.line_type = ParsedLineType.STATUS_REGISTER
            self.status_flags = status_flags
            self.status_register = self.status_register_from_flags(status_flags)
        elif status_register:
            self.line_type = ParsedLineType.STATUS_REGISTER
            self.status_register = status_register
            self.status_flags = self.status_flags_from_register(status_register)
        elif program_counter:
            self.line_type = ParsedLineType.PROGRAM_COUNTER_REGISTER
            self.program_counter = program_counter
        elif source_code_instruction or source_code_operands:
            self.line_type = ParsedLineType.SOURCE_CODE
            self.source_code_instruction = source_code_instruction
            self.source_code_operands = source_code_operands
        else:
            print("Bug in the code!")
            traceback.print_stack()
            sys.exit()

    @staticmethod
    def status_register_from_flags(flags: list[str]):
        result = 0x0000
        if "STATUS_REGISTER_MASK_CARRY" in flags:
            result = result | 0x0001
        if "STATUS_REGISTER_MASK_OVERFLOW" in flags:
            result = result | 0x0002
        if "STATUS_REGISTER_MASK_ZERO" in flags:
            result = result | 0x0004
        if "STATUS_REGISTER_MASK_NEGATIVE" in flags:
            result = result | 0x0008
        if "STATUS_REGISTER_MASK_EXTEND" in flags:
            result = result | 0x0010
        return result

    @staticmethod
    def status_flags_from_register(register: int):
        result = []
        if register & 0x01:
            result.append("STATUS_REGISTER_MASK_CARRY")
        if register & 0x02:
            result.append("STATUS_REGISTER_MASK_OVERFLOW")
        if register & 0x04:
            result.append("STATUS_REGISTER_MASK_ZERO")
        if register & 0x08:
            result.append("STATUS_REGISTER_MASK_NEGATIVE")
        if register & 0x10:
            result.append("STATUS_REGISTER_MASK_EXTEND")
        return result


class TestCase(object):
    test_name: str
    arrange_reg_data: ParsedLine
    arrange_reg_address: ParsedLine
    arrange_reg_sr: ParsedLine
    arrange_code: ParsedLine
    arrange_mem: [ParsedLine]
    assert_reg_data: ParsedLine
    assert_reg_address: ParsedLine
    assert_reg_sr_flags: ParsedLine
    assert_reg_sr: ParsedLine
    assert_reg_pc: ParsedLine
    assert_code: ParsedLine
    assert_mem: [ParsedLine]

    def __init__(self, test_name: str):
        self.test_name = test_name
        self.arrange_reg_data = None
        self.arrange_reg_address = None
        self.arrange_reg_sr = None
        self.arrange_code = None
        self.arrange_mem = []
        self.assert_reg_data = None
        self.assert_reg_address = None
        self.assert_reg_sr_flags = None
        self.assert_reg_sr = None
        self.assert_reg_pc = None
        self.assert_code = None
        self.assert_mem = []

    @staticmethod
    def get_d_reg_string(registers: list[int]):
        result = ""
        for idx, value in enumerate(registers):
            if idx == 0:
                result += f"0x{value:08x}"
            else:
                result += f", 0x{value:08x}"
        return result

    @staticmethod
    def get_a_reg_string(registers: list[int]):
        result = ""
        for idx, value in enumerate(registers):
            if idx == 0:
                result += f"0x{value:08x}"
            else:
                result += f", 0x{value:08x}"
        return result

    @staticmethod
    def get_d_reg_string_amiga(registers: list[int]):
        result = ""
        for idx, value in enumerate(registers):
            if idx == 0:
                result += f"${value:08x}"
            else:
                result += f",${value:08x}"
        return result

    @staticmethod
    def get_a_reg_string_amiga(registers: list[int]):
        result = ""
        for idx, value in enumerate(registers):
            if idx == 0:
                result += f"${value:08x}"
            else:
                result += f",${value:08x}"
        return result

    @staticmethod
    def get_status_reg_string(register: int):
        result = ""
        if register & 0x0010:
            result += 'E'
        else:
            result += '-'
        if register & 0x0008:
            result += 'N'
        else:
            result += '-'
        if register & 0x0004:
            result += 'Z'
        else:
            result += '-'
        if register & 0x0002:
            result += 'O'
        else:
            result += '-'
        if register & 0x0001:
            result += 'C'
        else:
            result += '-'

        return result

    def write_rust_test(self, file):
        file.write(f"\n")
        file.write(f"#[test]\n")
        file.write(f"fn {self.test_name.lower()}() {{\n")

        # Arrange - code
        file.write(f"    // arrange - code\n")
        arrange_code_bytes_hex = format_bytes_as_hex(self.arrange_code.bytes)
        assert_code_comment = get_assert_code_comment(self.assert_code)
        file.write(f"    // {assert_code_comment}\n")
        file.write(f"    let code = [{arrange_code_bytes_hex}].to_vec();\n")
        file.write(f"    let code_memory = RamMemory::from_bytes(0x{self.arrange_code.address:08x}, code);\n")
        file.write(f"\n")

        # Arrange - mem
        file.write(f"    // arrange - mem\n")
        if len(self.arrange_mem) == 0:
            file.write(f"    // -nothing-\n")
        for arr_mem in self.arrange_mem:
            arr_mem_bytes_hex = format_bytes_as_hex(arr_mem.bytes)
            file.write(f"    let arrange_mem_bytes_{arr_mem.address:08x} = [{arr_mem_bytes_hex}].to_vec();\n")
            file.write(f"    let arrange_mem_{arr_mem.address:08x} = RamMemory::from_bytes(0x{arr_mem.address:08x}, arrange_mem_bytes_{arr_mem.address:08x});\n")
        file.write(f"\n")

        # Arrange - common
        file.write(f"    // arrange - common\n")
        file.write(f"    let mut mem = Mem::new();\n")
        file.write(f"    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);\n")
        file.write(f"    let cia_memory = CiaMemory::new();\n")
        file.write(f"    mem.add_range(Rc::new(RefCell::new(code_memory)));\n")
        file.write(f"    mem.add_range(Rc::new(RefCell::new(vectors)));\n")
        file.write(f"    mem.add_range(Rc::new(RefCell::new(cia_memory)));\n")
        for arr_mem in self.arrange_mem:
            file.write(f"    mem.add_range(Rc::new(RefCell::new(arrange_mem_{arr_mem.address:08x})));\n")
        file.write(f"    let cpu = Cpu::new(&mem);\n")
        file.write(f"    let mut modermodem = Modermodem::new(None, cpu, mem, None);\n")
        file.write(f"\n")

        # Arrange - regs
        file.write(f"    // arrange - regs\n")
        file.write(
            f"    modermodem.cpu.register.set_all_d_reg_long_no_log({self.get_d_reg_string(self.arrange_reg_data.data_registers)});\n")
        file.write(
            f"    modermodem.cpu.register.set_all_a_reg_long_no_log({self.get_a_reg_string(self.arrange_reg_address.address_registers)});\n")
        file.write(f"    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x{self.arrange_code.address:08x});\n")
        # file.write(f"    modermodem.cpu.register.set_ssp_reg(0x01000400);\n")
        # if self.arrange_reg_sr is not None:
        file.write(f"    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(\n")
        if len(self.arrange_reg_sr.status_flags) > 0:
            for idx, stats_flag in enumerate(self.arrange_reg_sr.status_flags):
                if idx == 0:
                    file.write(f"       {stats_flag}\n")
                else:
                    file.write(f"       | {stats_flag}\n")
        else:
            file.write("       0x0000\n")
        file.write(f"    );\n")
        file.write(f"\n")

        # Act/Assert - Disassembly

        if self.assert_reg_pc is not None:
            assert_pc = self.assert_reg_pc.program_counter;

            # file.write(f" dc.l ${self.assert_reg_pc.program_counter:08x} ; PC\n")
        else:
            assert_pc = self.arrange_code.address + len(self.arrange_code.bytes);
            # file.write(f" dc.l ${self.arrange_code.address + len(self.arrange_code.bytes):08x} ; PC\n")

        file.write(f"    // act/assert - disassembly\n")
        file.write(f"    let get_disassembly_result = modermodem.get_next_disassembly_no_log();\n")
        file.write(f"""    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x{self.arrange_code.address:08x},
            0x{assert_pc:08x},
            String::from("{self.assert_code.source_code_instruction}"),
            String::from("{self.assert_code.source_code_operands}"),
            ),
            get_disassembly_result
        );\n""")
        file.write(f"\n")

        # Act
        file.write(f"    // act\n")
        file.write(f"    modermodem.step();\n")
        file.write(f"\n")

        # Assert - regs
        file.write(f"    // assert - regs\n")
        file.write(
            f"    modermodem.cpu.register.assert_all_d_reg_long_no_log({self.get_d_reg_string(self.assert_reg_data.data_registers)});\n")
        file.write(
            f"    modermodem.cpu.register.assert_all_a_reg_long_no_log({self.get_a_reg_string(self.assert_reg_address.address_registers)});\n")
        # if self.assert_reg_sr is not None:
        file.write(f"    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(\n")
        if len(self.assert_reg_sr.status_flags) > 0:
            for idx, stats_flag in enumerate(self.assert_reg_sr.status_flags):
                if idx == 0:
                    file.write(f"       {stats_flag}\n")
                else:
                    file.write(f"       | {stats_flag}\n")
        else:
            file.write("       0x0000\n")
        file.write(f"    );\n")
        file.write(f"\n")

        # Assert - mem
        file.write(f"    // assert - mem\n")
        if len(self.assert_mem) == 0:
            file.write(f"    // -nothing-\n")
        for ass_mem in self.assert_mem:
             for index, b in enumerate(ass_mem.bytes):
                 file.write(f"    assert_eq!(0x{b:02x}, modermodem.mem.get_byte_no_log(0x{(ass_mem.address + index):08x}));\n")
        file.write(f"}}\n")

    def write_amiga_test(self, file):
        file.write(";===========================================\n")
        file.write("\n")

        file.write(f"{self.test_name.lower()}\n")
        file.write(" dc.l .name\t; $00\n")
        file.write(" dc.l .arrange_mem\t; $04\n")
        file.write(" dc.l .arrange_regs\t; $08\n")
        file.write(" dc.l .arrange_code\t; $0c\n")
        file.write(" dc.l .assert_mem\t; $10\n")
        file.write(" dc.l .assert_regs\t; $14\n")
        file.write(" dc.l .assert_code\t; $18\n")
        file.write("\n")

        file.write(".name\n")
        file.write(f" dc.b \"{self.test_name.lower()}\",0\n")
        file.write(" even\n")
        file.write("\n")

        file.write(".arrange_mem\n")
        file.write(" ;length,address,ptr\n")
        for arr_mem in self.arrange_mem:
            file.write(f" dc.l ${len(arr_mem.bytes):08x},${arr_mem.address:08x},.arrange_mem_{arr_mem.address:08x}\n")

        file.write(" dc.l $00000000\n")
        file.write("\n")

        for arr_mem in self.arrange_mem:
            file.write(f".arrange_mem_{arr_mem.address:08x}\n")
            file.write(f" dc.b {format_bytes_as_hex_amiga(arr_mem.bytes)}\n")
            if len(arr_mem.bytes) % 2 == 1:
                file.write(f" even\n")
            file.write("\n")

        file.write(".arrange_regs\n")
        file.write(" ;    D0/A0     D1/A1     D2/A2     D3/A3     D4/A4     D5/A5     D6/A6     D7/A7\n")
        file.write(
            f" dc.l {self.get_d_reg_string_amiga(self.arrange_reg_data.data_registers)}\n")
        file.write(
            f" dc.l {self.get_a_reg_string_amiga(self.arrange_reg_address.address_registers)}\n")
        # if self.arrange_reg_sr is not None:
        file.write(f" dc.w ${self.arrange_reg_sr.status_register:04x} ; {self.get_status_reg_string(self.arrange_reg_sr.status_register)}\n")
        file.write("\n")

        file.write(".arrange_code\n")
        file.write(" ;length,address\n")
        file.write(f" dc.l ${(len(self.arrange_code.bytes) // 2):08x},${self.arrange_code.address:08x}\n")
        file.write(f" dc.b {format_bytes_as_hex_amiga(self.arrange_code.bytes)}\n")
        file.write("\n")

        file.write(".assert_mem\n")
        file.write(" ;length,address,ptr\n")
        for ass_mem in self.assert_mem:
            file.write(f" dc.l ${len(ass_mem.bytes):08x},${ass_mem.address:08x},.assert_mem_{ass_mem.address:08x}\n")
        file.write(" dc.l $00000000\n")
        file.write("\n")

        for ass_mem in self.assert_mem:
            file.write(f".assert_mem_{ass_mem.address:08x}\n")
            file.write(f" dc.b {format_bytes_as_hex_amiga(ass_mem.bytes)}\n")
            if len(ass_mem.bytes) % 2 == 1:
                file.write(f" even\n")
            file.write("\n")

        file.write(".assert_regs\n")
        file.write(" ;    D0/A0     D1/A1     D2/A2     D3/A3     D4/A4     D5/A5     D6/A6     D7/A7\n")
        file.write(
            f" dc.l {self.get_d_reg_string_amiga(self.assert_reg_data.data_registers)}\n")
        file.write(
            f" dc.l {self.get_a_reg_string_amiga(self.assert_reg_address.address_registers)}\n")
        # if self.assert_reg_sr is not None:
        if self.assert_reg_pc is not None:
            file.write(f" dc.l ${self.assert_reg_pc.program_counter:08x} ; PC\n")
        else:
            file.write(f" dc.l ${self.arrange_code.address + len(self.arrange_code.bytes):08x} ; PC\n")

        file.write(f" dc.w ${self.assert_reg_sr.status_register:04x} ; SR={self.get_status_reg_string(self.assert_reg_sr.status_register)}\n")
        # file.write(f" dc.l ${self.assert_reg_pc:08x}\n")
        file.write("\n")

        file.write(".assert_code\n")
        # file.write(" even\n")
        if self.assert_code.source_code_operands is not None:
            file.write(f" {self.assert_code.source_code_instruction} {self.assert_code.source_code_operands}\n")
        else:
            file.write(f" {self.assert_code.source_code_instruction}\n")
        file.write("\n")

class TestSet:
    test_spec_file_path: str
    test_cases: list[TestCase]

    def __init__(self, test_spec_file_path: str, test_cases: list[TestCase]):
        self.test_spec_file_path = test_spec_file_path
        self.test_cases = test_cases

    def write_rust_tests(self, path_to_rust_tests: str):
        rust_test_file_name = os.path.splitext(os.path.basename(self.test_spec_file_path))[0] + '.rs'
        rust_test_file_path = os.path.join(path_to_rust_tests, rust_test_file_name)

        # Open rust test file for writing
        file = open(rust_test_file_path, 'w')

        # Write rust test file header
        file.write("// Path: " + rust_test_file_path + "\n")
        file.write("// This file is autogenerated from " + self.test_spec_file_path + "\n")
        file.write("\n")
        file.write("#![allow(unused_imports)]\n")
        file.write("\n")
        file.write("use std::cell::RefCell;\n")
        file.write("use std::rc::Rc;\n")
        file.write("use crate::register::ProgramCounter;\n")
        file.write("use crate::mem::rammemory::RamMemory;\n")
        file.write("use crate::cpu::instruction::GetDisassemblyResult;\n")
        file.write("use crate::mem::memory::Memory;\n")
        file.write("use crate::mem::ciamemory::CiaMemory;\n")
        file.write("use crate::cpu::Cpu;\n")
        file.write("use crate::mem::Mem;\n")
        file.write("use crate::modermodem::Modermodem;\n")
        file.write("use crate::register::STATUS_REGISTER_MASK_CARRY;\n")
        file.write("use crate::register::STATUS_REGISTER_MASK_EXTEND;\n")
        file.write("use crate::register::STATUS_REGISTER_MASK_NEGATIVE;\n")
        file.write("use crate::register::STATUS_REGISTER_MASK_OVERFLOW;\n")
        file.write("use crate::register::STATUS_REGISTER_MASK_ZERO;\n")
        file.write("\n")

        # Write rust test file body
        for test_case in self.test_cases:
            test_case.write_rust_test(file)

        # Close rust test file
        file.close()

    def write_amiga_tests(self, path_to_amiga_tests: str):
        amiga_test_file_name = os.path.splitext(os.path.basename(self.test_spec_file_path))[0] + '.s'
        amiga_test_file_path = os.path.join(path_to_amiga_tests, amiga_test_file_name)

        # Open Amiga test file for writing
        file = open(amiga_test_file_path, 'w')

        # Write Amiga test file header
        file.write("; ----------------------T----------------------------------\n")
        file.write("\n")
        file.write("; Path: " + amiga_test_file_path + "\n")
        file.write("; This file is autogenerated\n")
        file.write("\n")
        file.write(" ;rts in case this source is run by mistake\n")
        file.write(" rts\n")
        file.write("\n")

        # Write Amiga test file body
        for test_case in self.test_cases:
            test_case.write_amiga_test(file)

        # Close Amiga test file
        file.close()


def format_bytes_as_hex(bytes: list[int]):
    bytes_hex = []
    for byte in bytes:
        byte_hex = f"0x{format(byte, '02X')}"
        bytes_hex.append(byte_hex)
    bytes_hex_str = ', '.join(bytes_hex)
    return bytes_hex_str

def format_bytes_as_hex_amiga(bytes: list[int]):
    bytes_hex = []
    for byte in bytes:
        byte_hex = f"${format(byte, '02X')}"
        bytes_hex.append(byte_hex)
    bytes_hex_str = ','.join(bytes_hex)
    return bytes_hex_str


def get_assert_code_comment(parsed_line: ParsedLine):
    if parsed_line.source_code_operands is not None:
        return f"{parsed_line.source_code_instruction} {parsed_line.source_code_operands}"
    else:
        return parsed_line.source_code_instruction


def ensure_folder_exists(folder_path):
    if not os.path.exists(folder_path):
        os.makedirs(folder_path)
        print(f"Folder '{folder_path}' created.")


# Parse command line arguments
if len(sys.argv) != 5:
    print("Usage:")
    print(
        f" {sys.argv[0]} [path_to_test_specs] [output_path_to_rust_tests] [output_path_to_rust_test_mod_file] [output_path_to_amiga_test_suit_file]")
    print("Example:")
    print(
        f" {sys.argv[0]} tests ..\\src\\cpu\\instruction\\gen_tests ..\\src\\cpu\\instruction\\gen_tests.rs D:\\Amiga\\KrustWB3\\Output\\Dev\\github\\rust-amiga-emul-ami-test-runner")
    sys.exit()

g_path_to_test_specs = sys.argv[1]
g_output_path_to_rust_tests = sys.argv[2]
g_output_path_to_rust_test_mod_file = sys.argv[3]
g_output_path_to_amiga_tests = sys.argv[4]

ensure_folder_exists(g_output_path_to_rust_tests)
ensure_folder_exists(g_output_path_to_amiga_tests)


# Get path to all test spec files
def get_test_spec_file_paths():
    test_spec_file_paths = [os.path.join(g_path_to_test_specs, f) for f in os.listdir(g_path_to_test_specs) if
                            os.path.isfile(os.path.join(g_path_to_test_specs, f))]
    return test_spec_file_paths


# Iterate all test spec files
def iterate_test_spec_file_paths(test_spec_file_paths):
    test_sets = []
    for test_spec_file_path in test_spec_file_paths:
        test_spec_file = open(test_spec_file_path, 'r')
        lines = test_spec_file.readlines()
        parsed_lines = parse_lines(lines)
        test_cases = get_test_cases(parsed_lines)
        test_set = TestSet(test_spec_file_path=test_spec_file_path, test_cases=test_cases)
        test_sets.append(test_set)
    return test_sets


def parse_lines(lines) -> list[ParsedLine]:
    parsed_lines = []
    line_number = 1
    for line in lines:
        line_raw = line.rstrip('\r\n')
        parsed_line = parse_line(line_number, line_raw)
        parsed_lines.append(parsed_line)
        line_number += 1
    return parsed_lines


def is_blank_line(line_stripped):
    return len(line_stripped) == 0


def is_test_name_line(line_stripped):
    return line_stripped.startswith(':')


keyword_regex = "^([a-zA-Z_]+)$"


def get_keyword_line(line_stripped):
    return re.search(keyword_regex, line_stripped)


def get_keyword(keyword_str):
    match keyword_str:
        case 'arrange_code':
            return Keyword.ARRANGE_CODE
        case 'arrange_mem':
            return Keyword.ARRANGE_MEM
        case 'arrange_reg':
            return Keyword.ARRANGE_REG
        case 'assert_code':
            return Keyword.ASSERT_CODE
        case 'assert_mem':
            return Keyword.ASSERT_MEM
        case 'assert_reg':
            return Keyword.ASSERT_REG
        case 'arrange_assert_mem':
            return Keyword.ARRANGE_ASSERT_MEM

    return None


address_line_regex = r"^\$([0-9a-fA-F]{8})(\s+.*)*$"


def get_address_line(line_stripped):
    return re.search(address_line_regex, line_stripped)


data_register_line_regex = r"^D0(\s+.*)$"
address_register_line_regex = r"^A0(\s+.*)$"
# TODO: Can we use SR for both of these?
status_register_flags_line_regex = r"^SR_FLAGS(\s+.*)$"
status_register_line_regex = r"^SR(\s+.*)$"
pc_register_line_regex = r"^PC(\s+.*)$"


def get_data_register_line(line_stripped):
    return re.search(data_register_line_regex, line_stripped)


def get_address_register_line(line_stripped):
    return re.search(address_register_line_regex, line_stripped)


def get_status_register_flags_line(line_stripped):
    return re.search(status_register_flags_line_regex, line_stripped)


def get_status_register_line(line_stripped):
    return re.search(status_register_line_regex, line_stripped)


def get_pc_register_line(line_stripped):
    return re.search(pc_register_line_regex, line_stripped)


bytes_regex = "([0-9a-fA-F]{2})"
words_regex = "([0-9a-fA-F]{4})"
longs_regex = "([0-9a-fA-F]{8})"
status_flags_regex = "\\s+[-E][-N][-Z][-O][-C]$"


def get_bytes(line_stripped):
    bytes = re.finditer(bytes_regex, line_stripped)
    if bytes is None:
        return None
    real_bytes = []
    for matchNum, match in enumerate(bytes, start=1):
        for group_num in range(0, len(match.groups())):
            group_num = group_num + 1
            byte = match.group(group_num)
            byte = int(byte, 16)
            real_bytes.append(byte)
    return real_bytes


def get_words(line_stripped):
    words = re.finditer(words_regex, line_stripped)
    if words is None:
        return None
    real_words = []
    for matchNum, match in enumerate(words, start=1):
        for group_num in range(0, len(match.groups())):
            group_num = group_num + 1
            word = match.group(group_num)
            word = int(word, 16)
            real_words.append(word)
    return real_words


def get_longs(line_stripped):
    longs = re.finditer(longs_regex, line_stripped)
    if longs is None:
        return None
    real_longs = []
    for matchNum, match in enumerate(longs, start=1):
        for groupNum in range(0, len(match.groups())):
            groupNum = groupNum + 1
            long = match.group(groupNum)
            long = int(long, 16)
            real_longs.append(long)
    return real_longs


def get_status_flags(line_stripped):
    status_flags = re.search(status_flags_regex, line_stripped)
    if status_flags is None:
        return None
    real_status_flags = []
    for status_flag in status_flags.group(0):
        match status_flag:
            case 'E':
                real_status_flags.append("STATUS_REGISTER_MASK_EXTEND")
            case 'N':
                real_status_flags.append("STATUS_REGISTER_MASK_NEGATIVE")
            case 'Z':
                real_status_flags.append("STATUS_REGISTER_MASK_ZERO")
            case 'O':
                real_status_flags.append("STATUS_REGISTER_MASK_OVERFLOW")
            case 'C':
                real_status_flags.append("STATUS_REGISTER_MASK_CARRY")
    return real_status_flags


def parse_line(line_number, line_raw) -> ParsedLine:
    semicolon_pos = line_raw.find(';')
    line_stripped = line_raw if semicolon_pos < 0 else line_raw[0:semicolon_pos]
    line_stripped = line_stripped.strip()

    if is_blank_line(line_stripped):
        line = ParsedLine(line_number=line_number, line_raw=line_raw, is_blank_line=True)
    elif is_test_name_line(line_stripped):
        # TODO: Make sure valid test_name ?
        line = ParsedLine(line_number=line_number, line_raw=line_raw, test_name=line_stripped[1:])
    elif get_keyword_line(line_stripped):
        keyword_match = get_keyword_line(line_stripped)
        keyword_str = keyword_match.group(1)
        keyword = get_keyword(keyword_str)
        if keyword is None:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unknown keyword '{keyword_str}' found at line {line_number}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, keyword=keyword)
    elif get_address_line(line_stripped):
        address_line = get_address_line(line_stripped)
        address = int(address_line.group(1), 16)
        address_content = address_line.group(2)
        if address_content is None:
            if address != 0x00000000:
                print(f"{line_number:5d}: {line_raw}")
                print(f"Unable to parse content of address 0x{address:08x}.")
                print(f" Missing content!")
                print(f" note: Only null termination address $00000000 can be used without content.")
                sys.exit()
            line = ParsedLine(line_number=line_number, line_raw=line_raw, address=address)
        elif get_bytes(address_content):
            bytes = get_bytes(address_content)
            line = ParsedLine(line_number=line_number, line_raw=line_raw, address=address, bytes=bytes)
        else:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of address 0x{address:08x}.")
            print(f" Content was: {address_content}")
            sys.exit()
    elif get_data_register_line(line_stripped):
        longs = get_longs(line_stripped)
        if len(longs) != 8:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of data registers.")
            print(
                f" Expected 8 32-bit integer hexadecimal values (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, data_registers=longs)
    elif get_address_register_line(line_stripped):
        longs = get_longs(line_stripped)
        if len(longs) != 8:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of address registers.")
            print(
                f" Expected 8 32-bit integer hexadecimal values (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, address_registers=longs)
    elif get_status_register_flags_line(line_stripped):
        status_flags = get_status_flags(line_stripped)
        if status_flags is None:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of status flags.")
            print(
                f" Expected 5 chars signalling SR flags in correct order, or dash for zero SR flag. Order is ENZOC.")
        line = ParsedLine(line_number=line_number, line_raw=line_raw, status_flags=status_flags)
    elif get_status_register_line(line_stripped):
        words = get_words(line_stripped)
        if len(words) != 1:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of status register.")
            print(
                f" Expected 1 16-bit integer hexadecimal value (without $ or 0x prefixes). Found {len(words)} integers: {words}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, status_register=words[0])
    elif get_pc_register_line(line_stripped):
        longs = get_longs(line_stripped)
        if len(longs) != 1:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of program counter register.")
            print(
                f" Expected 1 32-bit integer hexadecimal value (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, program_counter=longs[0])
    elif line_stripped.startswith('>'):
        code_parts = line_stripped[1:].strip().split()
        if len(code_parts) == 2:
            source_code_instruction = code_parts[0]
            source_code_operands = code_parts[1]
            line = ParsedLine(line_number=line_number, line_raw=line_raw,
                              source_code_instruction=source_code_instruction,
                              source_code_operands=source_code_operands)
        elif len(code_parts) == 1:
            source_code_instruction = code_parts[0]
            source_code_operands = code_parts[1]
            line = ParsedLine(line_number=line_number, line_raw=line_raw,
                              source_code_instruction=source_code_instruction,
                              source_code_operands=source_code_operands)
        else:
            print(f"{line_number:5d}: {line_raw}")
            print("Unable to parse content of code.")
            print(
                f" Expected 1 or 2 strings for instruction and possibly operands. E.g. NOP or MOVE.B #0,D0")
            sys.exit()
    else:
        print(f"{line_number:5d}: {line_raw}")
        print(f"Syntax Error parsing line {line_number}")
        sys.exit()

    return line


class ParseState(Enum):
    GLOBAL = 1
    ARRANGE_MEM = 2
    ARRANGE_REG = 3
    ARRANGE_CODE = 4
    ASSERT_MEM = 5
    ASSERT_REG = 6
    ASSERT_CODE = 7
    ARRANGE_AND_ASSERT_MEM = 10
    DONE = 666


def get_test_cases(parsed_lines: list[ParsedLine]):
    test_cases: list[TestCase] = []
    line_count = len(parsed_lines)
    current_line = 0
    while current_line < line_count:
        parsed_line = parsed_lines[current_line]
        current_line += 1
        match parsed_line.line_type:
            case ParsedLineType.BLANK:
                continue
            case ParsedLineType.TEST_NAME:
                # print(f"Found test name: {parsed_line.test_name}")
                test_case, current_line = get_test_case(parsed_line.test_name, parsed_lines, current_line, line_count)
                test_cases.append(test_case)
            case _:
                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                print(f"Syntax Error: Expected test name line. Found: {parsed_line.line_raw}")
                sys.exit()

    return test_cases


def get_test_case(test_name: str, parsed_lines: list[ParsedLine], current_line: int, line_count: int):
    test_case = TestCase(test_name=test_name)
    parse_state = ParseState.GLOBAL
    while parse_state != ParseState.DONE:
        if current_line >= line_count:
            parse_state = ParseState.DONE
        else:
            parsed_line = parsed_lines[current_line]

            match (parsed_line.line_type, parse_state):
                case (ParsedLineType.BLANK, _):
                    pass
                case (ParsedLineType.TEST_NAME, _):
                    parse_state = ParseState.DONE
                case (ParsedLineType.KEYWORD, _):
                    match parsed_line.keyword:
                        case Keyword.ARRANGE_MEM:
                            parse_state = ParseState.ARRANGE_MEM
                        case Keyword.ARRANGE_REG:
                            parse_state = ParseState.ARRANGE_REG
                        case Keyword.ARRANGE_CODE:
                            parse_state = ParseState.ARRANGE_CODE
                        case Keyword.ASSERT_MEM:
                            parse_state = ParseState.ASSERT_MEM
                        case Keyword.ASSERT_REG:
                            parse_state = ParseState.ASSERT_REG
                        case Keyword.ASSERT_CODE:
                            parse_state = ParseState.ASSERT_CODE
                        case Keyword.ARRANGE_ASSERT_MEM:
                            parse_state = ParseState.ARRANGE_AND_ASSERT_MEM
                case (ParsedLineType.ADDRESS_WITH_BYTES, _):
                    match parse_state:
                        case ParseState.ARRANGE_MEM:
                            test_case.arrange_mem.append(parsed_line)
                            pass
                        case ParseState.ARRANGE_CODE:
                            test_case.arrange_code = parsed_line
                            parse_state = ParseState.GLOBAL
                            pass
                        case ParseState.ASSERT_MEM:
                            test_case.assert_mem.append(parsed_line)
                            pass
                        case ParseState.ARRANGE_AND_ASSERT_MEM:
                            test_case.arrange_mem.append(parsed_line)
                            test_case.assert_mem.append(parsed_line)
                            pass
                        case _:
                            print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                            print(
                                f"Syntax Error parsing. Found address with bytes, but was in incorrect state {parse_state}.")
                            sys.exit()
                case (ParsedLineType.ADDRESS_NULL_TERMINATION, _):
                    match parse_state:
                        case ParseState.ARRANGE_MEM:
                            parse_state = ParseState.GLOBAL
                        case ParseState.ASSERT_MEM:
                            parse_state = ParseState.GLOBAL
                        case ParseState.ARRANGE_AND_ASSERT_MEM:
                            parse_state = ParseState.GLOBAL
                        case _:
                            print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                            print(
                                f"Syntax Error parsing. Found address null-termination, but was in incorrect state {parse_state}.")
                            sys.exit()
                case (ParsedLineType.DATA_REGISTERS, ParseState.ARRANGE_REG):
                    if test_case.arrange_reg_data is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of data registers for 'arrange_reg'.")
                        sys.exit()
                    else:
                        test_case.arrange_reg_data = parsed_line
                    # if test_case.is_arrange_reg_done():
                    #     parse_state = ParseState.GLOBAL
                case (ParsedLineType.DATA_REGISTERS, ParseState.ASSERT_REG):
                    if test_case.assert_reg_data is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of data registers for 'assert_reg'.")
                        sys.exit()
                    else:
                        test_case.assert_reg_data = parsed_line
                    # if test_case.is_assert_reg_done():
                    #     parse_state = ParseState.GLOBAL
                case (ParsedLineType.DATA_REGISTERS, _):
                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                    print(
                        f"Syntax Error parsing. Unexpected data registers found outside of 'arrange_reg' or 'assert_reg'.")
                    sys.exit()
                case (ParsedLineType.ADDRESS_REGISTERS, ParseState.ARRANGE_REG):
                    if test_case.arrange_reg_address is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of address registers for 'arrange_reg'.")
                        sys.exit()
                    else:
                        test_case.arrange_reg_address = parsed_line
                    # if test_case.is_arrange_reg_done():
                    #     parse_state = ParseState.GLOBAL
                case (ParsedLineType.ADDRESS_REGISTERS, ParseState.ASSERT_REG):
                    if test_case.assert_reg_address is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of address registers for 'assert_reg'.")
                        sys.exit()
                    else:
                        test_case.assert_reg_address = parsed_line
                    # if test_case.is_assert_reg_done():
                    #     parse_state = ParseState.GLOBAL
                case (ParsedLineType.ADDRESS_REGISTERS, _):
                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                    print(
                        f"Syntax Error parsing. Unexpected address registers found outside of 'arrange_reg' or 'assert_reg'.")
                    sys.exit()
                case (ParsedLineType.STATUS_REGISTER, ParseState.ARRANGE_REG):
                    if test_case.arrange_reg_sr is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of SR or SR_FLAGS for 'arrange_reg'.")
                        sys.exit()
                    else:
                        test_case.arrange_reg_sr = parsed_line
                case (ParsedLineType.STATUS_REGISTER, ParseState.ASSERT_REG):
                    if test_case.assert_reg_sr is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of SR or SR_FLAGS for 'assert_reg'.")
                        sys.exit()
                    else:
                        test_case.assert_reg_sr = parsed_line
                case (ParsedLineType.STATUS_REGISTER, _):
                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                    print(
                        f"Syntax Error parsing. Unexpected status register found outside of 'arrange_reg' or 'assert_reg'.")
                    sys.exit()
                case (ParsedLineType.PROGRAM_COUNTER_REGISTER, ParseState.ASSERT_REG):
                    if test_case.assert_reg_pc is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of program counter register for 'assert_reg'.")
                        sys.exit()
                    else:
                        test_case.assert_reg_pc = parsed_line
                case (ParsedLineType.PROGRAM_COUNTER_REGISTER, _):
                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                    print(
                        f"Syntax Error parsing. Unexpected program counter register found outside of 'assert_reg'.")
                    sys.exit()
                case (ParsedLineType.SOURCE_CODE, ParseState.ASSERT_CODE):
                    if test_case.assert_code is not None:
                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                        print(
                            f"Syntax Error parsing. Found multiple rows of source code for 'assert_code'.")
                        sys.exit()
                    else:
                        test_case.assert_code = parsed_line
                case (ParsedLineType.SOURCE_CODE, _):
                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                    print(f"Syntax Error parsing. Unexpected source code found outside of 'assert_code'.")
                    sys.exit()
                case (_, _):
                    print(f"State={parse_state} : {parsed_line.line_type}")
                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                    print(
                        f"Syntax Error parsing. Unexpected LineType and ParseState combo {(parsed_line.line_type, parse_state)}.")
                    sys.exit()

        # if parse_state == ParseState.DONE:
        #     print(f"Test case is done: {test_case.test_name}")
        #     print(f"arrange_reg_data: {test_case.arrange_reg_data.data_registers}")
        #     print(f"arrange_reg_address: {test_case.arrange_reg_address.address_registers}")
        #     if test_case.arrange_reg_sr is not None:
        #         print(f"arrenge_reg_sr: {test_case.arrange_reg_sr.status_register}")
        #     elif test_case.arrange_reg_sr_flags is not None:
        #         print(f"arrenge_reg_sr: {test_case.arrange_reg_sr_flags.status_flags}")
        #
        #     print(f"arrange_code: {test_case.arrange_code.address} => {test_case.arrange_code.bytes}")
        #     print(f"assert_reg_data: {test_case.assert_reg_data.data_registers}")
        #     print(f"assert_reg_address: {test_case.assert_reg_address.address_registers}")
        #
        #     if test_case.assert_reg_sr is not None:
        #         print(f"assert_reg_sr: {test_case.assert_reg_sr.status_register}")
        #     elif test_case.assert_reg_sr_flags is not None:
        #         print(f"assert_reg_sr: {test_case.assert_reg_sr_flags.status_flags}")
        #
        #     print(f"assert_reg_pc: {test_case.assert_reg_pc.program_counter}")
        #     print(f"assert_code: {test_case.assert_code}")
        #     # continue
        # else:
        #     current_line += 1
        if parse_state != ParseState.DONE:
            current_line += 1

    return test_case, current_line


def write_rust_mod_file(output_path_to_rust_test_mod_file: str, test_sets: list[TestSet]):
    # Open rust test file for writing
    file = open(output_path_to_rust_test_mod_file, 'w')

    # Write rust test file header
    file.write("// Path: " + output_path_to_rust_test_mod_file + "\n")
    file.write("// This file is autogenerated\n")
    file.write("\n")

    # Write rust test file body
    for test_set in test_sets:
        rust_test_file_name = os.path.splitext(os.path.basename(test_set.test_spec_file_path))[0]
        file.write("pub mod " + rust_test_file_name + ";\n")
        file.write("\n")

    # Close rust test file
    file.close()


def write_amiga_test_suite_file(output_path_to_amiga_tests: str, test_sets: list[TestSet]):
    amiga_test_suites_file_path = os.path.join(output_path_to_amiga_tests, "test_suite.s")

    # Open Amiga test file for writing
    file = open(amiga_test_suites_file_path, 'w')

    # Write Amiga test file header
    file.write(";---------------T---------T---------------------T----------\\x10")
    file.write("\n")
    file.write("; Path: " + amiga_test_suites_file_path + "\n")
    file.write("; This file is autogenerated\n")
    file.write("\n")
    file.write("\t; rts in case this source is run by mistake\n")
    file.write("\trts\n")
    file.write("\n")

    # Write Amiga test list
    file.write("test_suite\n")
    for test_set in test_sets:
        if len(test_set.test_cases) > 0:
            for test_case in test_set.test_cases:
                file.write(f"\tdc.l\t{test_case.test_name.lower()}\n")
    file.write("\n")
    file.write("\tdc.l\t$0\n")
    file.write("\n")

    # Write Amiga test includes
    for test_set in test_sets:
        if len(test_set.test_cases) > 0:
            rust_test_file_name = os.path.splitext(os.path.basename(test_set.test_spec_file_path))[0]
            file.write(f"\tinclude\t\"{rust_test_file_name.lower()}.s\"\n")

    # Close Amiga test file
    file.close()


g_test_spec_file_paths = get_test_spec_file_paths()
g_test_sets = iterate_test_spec_file_paths(g_test_spec_file_paths)
write_rust_mod_file(g_output_path_to_rust_test_mod_file, g_test_sets)
write_amiga_test_suite_file(g_output_path_to_amiga_tests, g_test_sets)
for g_test_set in g_test_sets:
    if len(g_test_set.test_cases) > 0:
        g_test_set.write_rust_tests(g_output_path_to_rust_tests)
        g_test_set.write_amiga_tests(g_output_path_to_amiga_tests)
