import sys
import os
from enum import Enum
import re
import traceback

class Keyword(Enum):
    ARRANGE_MEM = 100
    ARRANGE_REG = 101
    ARRANGE_CODE = 102
    ASSERT_MEM = 200
    ASSERT_REG = 201
    ASSERT_CODE = 202

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
    keyword: str
    address: int
    bytes: list[int]
    data_registers: list[int]
    address_registers: list[int]
    status_register: int
    program_counter: int
    source_code: str

    def __init__(self, line_number: int, line_raw: str, is_blank_line: bool = False, test_name: str = None, keyword: str = None, address: int = None, bytes: list[int] = None,  data_registers: list[int] = None, address_registers: list[int] = None, status_register: int = None, program_counter: int = None, source_code: str = None):
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
        elif address != None and bytes == None:
            self.line_type = ParsedLineType.ADDRESS_NULL_TERMINATION
            self.address = address
        elif address != None and bytes != None:
            self.line_type = ParsedLineType.ADDRESS_WITH_BYTES
            self.address = address
            self.bytes = bytes
        elif data_registers:
            self.line_type = ParsedLineType.DATA_REGISTERS
            self.data_registers = data_registers
        elif address_registers:
            self.line_type = ParsedLineType.ADDRESS_REGISTERS
            self.address_registers = address_registers
        elif status_register:
            self.line_type = ParsedLineType.STATUS_REGISTER
            self.status_register = status_register
        elif program_counter:
            self.line_type = ParsedLineType.PROGRAM_COUNTER_REGISTER
            self.program_counter = program_counter
        elif source_code:
            self.line_type = ParsedLineType.SOURCE_CODE
            self.source_code = source_code
        else:
            print("Bug in the code!")
            traceback.print_stack()
            sys.exit()

class TestCase(object):
    test_name: str
    arrange_reg_data: ParsedLine
    arrange_reg_address: ParsedLine
    arrange_reg_sr: ParsedLine
    arrange_code: ParsedLine
    assert_reg_data: ParsedLine
    assert_reg_address: ParsedLine
    assert_reg_sr: ParsedLine
    assert_reg_pc: ParsedLine
    assert_code: list[str]

    def __init__(self, test_name: str):
        self.test_name = test_name
        self.arrange_reg_data = None
        self.arrange_reg_address = None
        self.arrange_reg_sr = None

        self.arrange_code = None
        self.assert_reg_data = None
        self.assert_reg_address = None
        self.assert_reg_sr = None
        self.assert_reg_pc = None
        self.assert_code = None
        self.assert_code = []
    
    def is_arrange_reg_done(self):
        return self.arrange_reg_data != None and self.arrange_reg_address != None and self.arrange_reg_sr != None
    
    def is_assert_reg_done(self):
        return self.assert_reg_data != None and self.assert_reg_address != None and self.assert_reg_sr != None and self.assert_reg_pc != None

class TestSet:
    test_spec_file_path: str

    def __init__(self, test_spec_file_path: str):
        self.test_spec_file_path = test_spec_file_path

# Parse command line arguments
if len(sys.argv) != 4:
    print("Usage:")
    print(f" {sys.argv[0]} [path_to_test_specs] [output_path_to_rust_tests] [output_path_to_amiga_tests]")
    sys.exit()

path_to_test_specs = sys.argv[1]
output_path_to_rust_tests = sys.argv[2]
output_path_to_amiga_tests = sys.argv[3]

# Get path to all test spec files
def get_test_spec_file_paths():
    test_spec_file_paths = [os.path.join(path_to_test_specs, f) for f in os.listdir(path_to_test_specs) if os.path.isfile(os.path.join(path_to_test_specs, f))]
    return test_spec_file_paths

# Iterate all test spec files
def iterate_test_spec_file_paths(test_spec_file_paths):
    for test_spec_file_path in test_spec_file_paths:
        # print(f"file: {test_spec_file_path}")
        # file_name = os.path.basename(test_spec_file_path)
        # print(f"file: {file_name}")
        # deff = os.path.splitext(file_name)[0]+'.rs'
        # print(f"deff: {deff}")
        test_sets = []
        test_spec_file = open(test_spec_file_path, 'r')
        lines = test_spec_file.readlines()
        parsed_lines = parse_lines(lines)
        # for parsed_line in parsed_lines:
        #     print(parsed_line)
        get_test_spec(parsed_lines)
        test_set = TestSet(test_spec_file_path=test_spec_file_path)
        test_sets.append(test_set)
        return test_sets

def parse_lines(lines) -> list[ParsedLine]:
    print("Parsing file")
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
        
    return None

address_line_regex = r"^\$([0-9a-fA-F]{8})(\s+.*)*$"
def get_address_line(line_stripped):
    return re.search(address_line_regex, line_stripped)

data_register_line_regex    = r"^D0(\s+.*)$"
address_register_line_regex = r"^A0(\s+.*)$"
status_register_line_regex  = r"^SR(\s+.*)$"
pc_register_line_regex      = r"^PC(\s+.*)$"

def get_data_register_line(line_stripped):
    return re.search(data_register_line_regex, line_stripped)

def get_address_register_line(line_stripped):
    return re.search(address_register_line_regex, line_stripped)

def get_status_register_line(line_stripped):
    return re.search(status_register_line_regex, line_stripped)

def get_pc_register_line(line_stripped):
    return re.search(pc_register_line_regex, line_stripped)

bytes_regex = "([0-9a-fA-F]{2})"
words_regex = "([0-9a-fA-F]{4})"
longs_regex = "([0-9a-fA-F]{8})"

def get_bytes(line_stripped):
    bytes = re.finditer(bytes_regex, line_stripped)
    if bytes == None:
        return None
    real_bytes = []
    for matchNum, match in enumerate(bytes, start=1):
        for groupNum in range(0, len(match.groups())):
            groupNum = groupNum + 1
            byte = match.group(groupNum)
            # print(f"byte: {byte}")
            byte = int(byte, 16)
            real_bytes.append(byte)
    return real_bytes

def get_words(line_stripped):
    words = re.finditer(words_regex, line_stripped)
    if words == None:
        return None
    real_words = []
    for matchNum, match in enumerate(words, start=1):
        for groupNum in range(0, len(match.groups())):
            groupNum = groupNum + 1
            word = match.group(groupNum)
            # print(f"byte: {byte}")
            word = int(word, 16)
            real_words.append(word)
    return real_words


def get_longs(line_stripped):
    longs = re.finditer(longs_regex, line_stripped)
    if longs == None:
        return None
    real_longs = []
    for matchNum, match in enumerate(longs, start=1):
        for groupNum in range(0, len(match.groups())):
            groupNum = groupNum + 1
            long = match.group(groupNum)
            # print(f"byte: {byte}")
            long = int(long, 16)
            real_longs.append(long)
    return real_longs

def parse_line(line_number, line_raw) -> ParsedLine:
    semicolon_pos = line_raw.find(';')
    line_stripped = line_raw if semicolon_pos < 0 else line_raw[0:semicolon_pos]
    line_stripped = line_stripped.strip()

    line = None
    if is_blank_line(line_stripped):
        line = ParsedLine(line_number=line_number, line_raw=line_raw, is_blank_line=True)
    elif is_test_name_line(line_stripped):
        line = ParsedLine(line_number=line_number, line_raw=line_raw, test_name=line_stripped[1:])
    elif get_keyword_line(line_stripped):
        keyword_match = get_keyword_line(line_stripped)
        keyword_str = keyword_match.group(1)
        keyword = get_keyword(keyword_str)
        if keyword == None:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unknown keyword '{keyword_str}' found at line {line_number}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, keyword=keyword)
    elif get_address_line(line_stripped):
        address_line = get_address_line(line_stripped)
        address = int(address_line.group(1), 16)
        address_content = address_line.group(2)
        if address_content == None:
            if address != 0x00000000:
                print(f"{line_number:5d}: {line_raw}")
                print(f"Unable to parse content of address 0x{address:08x}.")
                print(f" Missing content!")
                print(f" note: Only null termination address $00000000 can be used without content.")
                sys.exit()
            print(f"address: {address}")
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
            print(f" Expected 8 32-bit integer hexadecimal values (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, data_registers=longs)
    elif get_address_register_line(line_stripped):
        longs = get_longs(line_stripped)
        if len(longs) != 8:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of address registers.")
            print(f" Expected 8 32-bit integer hexadecimal values (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, address_registers=longs)
    elif get_status_register_line(line_stripped):
        words = get_words(line_stripped)
        if len(words) != 1:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of status register.")
            print(f" Expected 1 16-bit integer hexadecimal value (without $ or 0x prefixes). Found {len(words)} integers: {words}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, status_register=words[0])
    elif get_pc_register_line(line_stripped):
        longs = get_longs(line_stripped)
        if len(longs) != 1:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of program counter register.")
            print(f" Expected 1 32-bit integer hexadecimal value (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = ParsedLine(line_number=line_number, line_raw=line_raw, program_counter=longs[0])
    elif line_stripped.startswith('>'):
        code = line_stripped[1:]
        line = ParsedLine(line_number=line_number, line_raw=line_raw, source_code=code)
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

def get_test_spec(parsed_lines: list[ParsedLine]):
    
    line_count = len(parsed_lines)
    current_line = 0
    while current_line < line_count:
        parsed_line = parsed_lines[current_line]
        current_line += 1
        match parsed_line.line_type:
            case ParsedLineType.BLANK:
                continue
            case ParsedLineType.TEST_NAME:
                print(f"Found test name: {parsed_line.test_name}")
                test_case = TestCase(test_name=parsed_line.test_name)
                parse_state = ParseState.GLOBAL
                while parse_state != ParseState.DONE:
                    if current_line >= line_count:
                        parse_state = ParseState.DONE
                    else:
                        parsed_line = parsed_lines[current_line]
                        
                        match (parsed_line.line_type, parse_state):
                            case (ParsedLineType.BLANK, _):
                                # print(f"State={parse_state} : skipping blank line {parsed_line.line_number}")
                                pass
                            case (ParsedLineType.TEST_NAME, _):
                                parse_state = ParseState.DONE
                            case (ParsedLineType.KEYWORD, ParseState.GLOBAL):
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
                            case (ParsedLineType.KEYWORD, ParseState.ARRANGE_MEM):
                                if parsed_line.keyword == Keyword.ASSERT_MEM:
                                    parse_state = ParseState.ARRANGE_AND_ASSERT_MEM
                                else:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing 'arrange_mem' segment. Unexpected Keyword {parsed_line.keyword}.")
                                    sys.exit()
                            case (ParsedLineType.KEYWORD, ParseState.ASSERT_MEM):
                                if parsed_line.keyword == Keyword.ARRANGE_MEM:
                                    parse_state = ParseState.ARRANGE_AND_ASSERT_MEM
                                else:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing 'assert_mem' segment. Unexpected Keyword {parsed_line.keyword}.")
                                    sys.exit()
                            case (ParsedLineType.ADDRESS_WITH_BYTES, _):
                                match parse_state:
                                    case ParseState.ARRANGE_MEM:
                                        # TODO: Do it
                                        print("Adding address with bytes as ARRANGE_MEM")
                                        pass
                                    case ParseState.ARRANGE_CODE:
                                        test_case.arrange_code = parsed_line
                                        parse_state = ParseState.GLOBAL
                                        pass
                                    case ParseState.ASSERT_MEM:
                                        # TODO: Do it
                                        print("Adding address with bytes as ASSERT_MEM")
                                        pass
                                    case ParseState.ARRANGE_AND_ASSERT_MEM:
                                        # TODO: Do it
                                        print("Adding address with bytes as ARRANGE_AND_ASSERT_MEM")
                                        pass
                                    case _:
                                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                        print(f"Syntax Error parsing. Found address with bytes, but was in incorrect state {parse_state}.")
                                        sys.exit()
                            case (ParsedLineType.ADDRESS_NULL_TERMINATION, _):
                                match parse_state:
                                    case ParseState.ARRANGE_MEM:
                                        # TODO: Do it
                                        print("Adding address null-termination as ARRANGE_MEM")
                                        parse_state = ParseState.GLOBAL
                                    case ParseState.ASSERT_MEM:
                                        # TODO: Do it
                                        print("Adding address null-termination as ASSERT_MEM")
                                        parse_state = ParseState.GLOBAL
                                    case ParseState.ARRANGE_AND_ASSERT_MEM:
                                        # TODO: Do it
                                        print("Adding address null-termination as ARRANGE_AND_ASSERT_MEM")
                                        parse_state = ParseState.GLOBAL
                                    case _:
                                        print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                        print(f"Syntax Error parsing. Found address null-termination, but was in incorrect state {parse_state}.")
                                        sys.exit()
                            case (ParsedLineType.DATA_REGISTERS, ParseState.ARRANGE_REG):
                                if test_case.arrange_reg_data != None:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing. Found multiple rows of data registers for 'arrange_reg'.")
                                    sys.exit()
                                else:
                                    test_case.arrange_reg_data = parsed_line
                                if test_case.is_arrange_reg_done():
                                    parse_state = ParseState.GLOBAL   
                            case (ParsedLineType.DATA_REGISTERS, ParseState.ASSERT_REG):
                                if test_case.assert_reg_data != None:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing. Found multiple rows of data registers for 'assert_reg'.")
                                    sys.exit()
                                else:
                                    test_case.assert_reg_data = parsed_line
                                if test_case.is_assert_reg_done():
                                    parse_state = ParseState.GLOBAL
                            case (ParsedLineType.DATA_REGISTERS, _):
                                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                print(f"Syntax Error parsing. Unexpected data registers found outside of 'arrange_reg' or 'assert_reg'.")
                                sys.exit()
                            case (ParsedLineType.ADDRESS_REGISTERS, ParseState.ARRANGE_REG):
                                if test_case.arrange_reg_address != None:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing. Found multiple rows of address registers for 'arrange_reg'.")
                                    sys.exit()
                                else:
                                    test_case.arrange_reg_address = parsed_line
                                if test_case.is_arrange_reg_done():
                                    parse_state = ParseState.GLOBAL
                            case (ParsedLineType.ADDRESS_REGISTERS, ParseState.ASSERT_REG):
                                if test_case.assert_reg_address != None:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing. Found multiple rows of address registers for 'assert_reg'.")
                                    sys.exit()
                                else:
                                    test_case.assert_reg_address = parsed_line
                                if test_case.is_assert_reg_done():
                                    parse_state = ParseState.GLOBAL
                            case (ParsedLineType.ADDRESS_REGISTERS, _):
                                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                print(f"Syntax Error parsing. Unexpected address registers found outside of 'arrange_reg' or 'assert_reg'.")
                                sys.exit()
                            case (ParsedLineType.STATUS_REGISTER, ParseState.ARRANGE_REG):
                                if test_case.arrange_reg_sr != None:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing. Found multiple rows of status register for 'arrange_reg'.")
                                    sys.exit()
                                else:
                                    test_case.arrange_reg_sr = parsed_line
                                if test_case.is_arrange_reg_done():
                                    parse_state = ParseState.GLOBAL        
                            case (ParsedLineType.STATUS_REGISTER, ParseState.ASSERT_REG):
                                if test_case.assert_reg_sr != None:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing. Found multiple rows of status register for 'assert_reg'.")
                                    sys.exit()
                                else:
                                    test_case.assert_reg_sr = parsed_line
                                if test_case.is_assert_reg_done():
                                    parse_state = ParseState.GLOBAL
                            case (ParsedLineType.STATUS_REGISTER, _):
                                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                print(f"Syntax Error parsing. Unexpected status register found outside of 'arrange_reg' or 'assert_reg'.")
                                sys.exit()
                            case (ParsedLineType.PROGRAM_COUNTER_REGISTER, ParseState.ASSERT_REG):
                                if test_case.assert_reg_pc != None:
                                    print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                    print(f"Syntax Error parsing. Found multiple rows of program counter register for 'assert_reg'.")
                                    sys.exit()
                                else:
                                    test_case.assert_reg_pc = parsed_line
                                if test_case.is_assert_reg_done():
                                    parse_state = ParseState.GLOBAL
                            case (ParsedLineType.PROGRAM_COUNTER_REGISTER, _):
                                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                print(f"Syntax Error parsing. Unexpected program counter register found outside of 'assert_reg'.")
                                sys.exit()
                            case (ParsedLineType.SOURCE_CODE, ParseState.ASSERT_CODE):
                                test_case.assert_code.append(parsed_line.source_code)
                                print("Adding pc-reg as ASSERT_REG")
                            case (ParsedLineType.SOURCE_CODE, _):
                                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                print(f"Syntax Error parsing. Unexpected source code found outside of 'assert_code'.")
                                sys.exit()
                            case (_, _):
                                print(f"State={parse_state} : {parsed_line.line_type}")
                                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                                print(f"Syntax Error parsing. Unexpected LineType and ParseState combo {(parsed_line.line_type, parse_state)}.")
                                sys.exit()          
                    if parse_state == ParseState.DONE:
                        print(f"Test case is done: {test_case.test_name}")
                        print(f"arrange_reg_data: {test_case.arrange_reg_data.data_registers}")
                        print(f"arrange_reg_address: {test_case.arrange_reg_address.address_registers}")
                        print(f"arrange_reg_sr: {test_case.arrange_reg_sr.status_register}")

                        print(f"arrange_code: {test_case.arrange_code.address} => {test_case.arrange_code.bytes}")
                        print(f"assert_reg_data: {test_case.assert_reg_data.data_registers}")
                        print(f"assert_reg_address: {test_case.assert_reg_address.address_registers}")

                        print(f"assert_reg_sr: {test_case.assert_reg_sr.status_register}")
                        print(f"assert_reg_pc: {test_case.assert_reg_pc.program_counter}")
                        print(f"assert_code: {test_case.assert_code}")
                        continue
                    else:
                        current_line += 1
                # continue
            case _:
                print(f"{parsed_line.line_number:5d}: {parsed_line.line_raw}")
                print(f"Syntax Error: Expected test name line. Found: {parsed_line.line_raw}")
                sys.exit()
        
    print(f"number of lines: {line_count}")

test_spec_file_paths = get_test_spec_file_paths()
test_sets = iterate_test_spec_file_paths(test_spec_file_paths)
for test_set in test_sets:
    print(f"test_spec_file_path: {test_set.test_spec_file_path}")
