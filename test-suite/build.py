import sys
import os
from enum import Enum
from typing import NamedTuple
import re


class Keyword(Enum):
    ARRANGE_MEM = 100
    ARRANGE_REG = 101
    ARRANGE_CODE = 102
    ASSERT_MEM = 200
    ASSERT_REG = 201
    ASSERT_CODE = 202

class BlankLine(NamedTuple):
    line_number: int
    line_raw: str

class TestNameLine(NamedTuple):
    line_number: int
    line_raw: str
    name: str

class KeywordLine(NamedTuple):
    line_number: int
    line_raw: str
    keyword: str

class AddressNullTermination(NamedTuple):
    line_number: int
    line_raw: str

class AddressWithBytesLine(NamedTuple):
    line_number: int
    line_raw: str
    address: int
    bytes: list[int]

class DataRegistersLine(NamedTuple):
    line_number: int
    line_raw: str
    longs: list[int]

class AddressRegistersLine(NamedTuple):
    line_number: int
    line_raw: str
    longs: list[int]

class StatusRegisterLine(NamedTuple):
    line_number: int
    line_raw: str
    word: int

class PcRegisterLine(NamedTuple):
    line_number: int
    line_raw: str
    long: int

class CodeLine(NamedTuple):
    line_number: int
    line_raw: str
    code: str

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
        test_spec_file = open(test_spec_file_path, 'r')
        lines = test_spec_file.readlines()
        # print(str(lines))
        # parse_arrange_mem(lines)
        parsed_lines = parse_lines(lines)
        for parsed_line in parsed_lines:
            print(parsed_line)
        # print(f"{str(parsed_lines)}")

def parse_lines(lines):
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

def parse_line(line_number, line_raw):
    semicolon_pos = line_raw.find(';')
    line_stripped = line_raw if semicolon_pos < 0 else line_raw[0:semicolon_pos]
    line_stripped = line_stripped.strip()

    if is_blank_line(line_stripped):
        line = BlankLine(line_number=line_number, line_raw=line_raw)
    elif is_test_name_line(line_stripped):
        line = TestNameLine(line_number=line_number, line_raw=line_raw, name=line_stripped[1])
    elif get_keyword_line(line_stripped):
        keyword_match = get_keyword_line(line_stripped)
        keyword_str = keyword_match.group(1)
        keyword = get_keyword(keyword_str)
        if keyword == None:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unknown keyword '{keyword_str}' found at line {line_number}")
            sys.exit()
        line = KeywordLine(line_number=line_number, line_raw=line_raw, keyword=keyword)
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
            line = AddressNullTermination(line_number=line_number, line_raw=line_raw)
        elif get_bytes(address_content):
            bytes = get_bytes(address_content)
            line = AddressWithBytesLine(line_number=line_number, line_raw=line_raw, address=address, bytes=bytes)
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
        line = DataRegistersLine(line_number=line_number, line_raw=line_raw, longs=longs)
    elif get_address_register_line(line_stripped):
        longs = get_longs(line_stripped)
        if len(longs) != 8:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of address registers.")
            print(f" Expected 8 32-bit integer hexadecimal values (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = AddressRegistersLine(line_number=line_number, line_raw=line_raw, longs=longs)
    elif get_status_register_line(line_stripped):
        words = get_words(line_stripped)
        if len(words) != 1:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of status register.")
            print(f" Expected 1 16-bit integer hexadecimal value (without $ or 0x prefixes). Found {len(words)} integers: {words}")
            sys.exit()
        line = StatusRegisterLine(line_number=line_number, line_raw=line_raw, word=words[0])
    elif get_pc_register_line(line_stripped):
        longs = get_longs(line_stripped)
        if len(longs) != 1:
            print(f"{line_number:5d}: {line_raw}")
            print(f"Unable to parse content of program counter register.")
            print(f" Expected 1 32-bit integer hexadecimal value (without $ or 0x prefixes). Found {len(longs)} integers: {longs}")
            sys.exit()
        line = PcRegisterLine(line_number=line_number, line_raw=line_raw, long=longs[0])
    elif line_stripped.startswith('>'):
        code = line_stripped[1:]
        line = CodeLine(line_number=line_number, line_raw=line_raw, code=code)
    else:
        print(f"{line_number:5d}: {line_raw}")
        print(f"Syntax Error parsing line {line_number}")
        sys.exit()
    
    return line

test_spec_file_paths = get_test_spec_file_paths()
iterate_test_spec_file_paths(test_spec_file_paths)
