use std::ops::{Shl, Shr};

pub trait AInt: Copy + PartialEq + Shl<u32, Output = Self> + Shr<u32, Output = Self> {
    fn bit_and(self, other: Self) -> Self;
    fn bit_or(self, other: Self) -> Self;
    fn bit_not(self) -> Self;
    fn is_zero(self) -> bool;
    fn is_msb_set(self) -> bool;
    fn is_lsb_set(self) -> bool;
    fn get_msb_mask(self) -> Self;
    // fn leading_zeros(self) -> u32;
    fn zero() -> Self;

    fn get_hex_string(self) -> String;

    fn get_from_long(value: u32) -> Self;
    fn set_in_long(self, dest: u32) -> u32;

    fn checked_shift_left(self, rhs: u32) -> Option<Self>;
    fn checked_shift_right(self, rhs: u32) -> Option<Self>;
}

impl AInt for u8 {
    fn bit_and(self, other: Self) -> Self {
        self & other
    }
    fn bit_or(self, other: Self) -> Self {
        self | other
    }
    fn bit_not(self) -> Self {
        !self
    }
    fn is_zero(self) -> bool {
        self == 0x00
    }
    fn is_msb_set(self) -> bool {
        (self & 0x80) == 0x80
    }
    fn is_lsb_set(self) -> bool {
        (self & 0x01) == 0x01
    }
    fn get_msb_mask(self) -> Self {
        self & 0x80
    }
    // fn leading_zeros(self) -> u32 {
    //     self.leading_zeros()
    // }
    fn zero() -> Self {
        0x00
    }

    fn get_hex_string(self) -> String {
        format!("{:02x}", self)
    }

    fn get_from_long(value: u32) -> Self {
        (value & 0x000000ff) as Self
    }
    fn set_in_long(self, dest: u32) -> u32{
        dest & 0xffffff00 | self as u32
    }

    fn checked_shift_left(self, rhs: u32) -> Option<Self> {
        self.checked_shl(rhs)
    }
    fn checked_shift_right(self, rhs: u32) -> Option<Self> {
        self.checked_shr(rhs)
    }
}

impl AInt for u16 {
    fn bit_and(self, other: Self) -> Self {
        self & other
    }
    fn bit_or(self, other: Self) -> Self {
        self | other
    }
    fn bit_not(self) -> Self {
        !self
    }
    fn is_zero(self) -> bool {
        self == 0x0000
    }
    fn is_msb_set(self) -> bool {
        (self & 0x8000) == 0x8000
    }
    fn is_lsb_set(self) -> bool {
        (self & 0x0001) == 0x0001
    }
    fn get_msb_mask(self) -> Self {
        self & 0x8000
    }
    // fn leading_zeros(self) -> u32 {
    //     self.leading_zeros()
    // }
    fn zero() -> Self {
        0x0000
    }

    fn get_hex_string(self) -> String {
        format!("{:04x}", self)
    }

    fn get_from_long(value: u32) -> Self {
        (value & 0x0000ffff) as Self
    }
    fn set_in_long(self, dest: u32) -> u32{
        dest & 0xffff0000 | self as u32
    }

    fn checked_shift_left(self, rhs: u32) -> Option<Self> {
        self.checked_shl(rhs)
    }
    fn checked_shift_right(self, rhs: u32) -> Option<Self> {
        self.checked_shr(rhs)
    }
}

impl AInt for u32 {
    fn bit_and(self, other: Self) -> Self {
        self & other
    }
    fn bit_or(self, other: Self) -> Self {
        self | other
    }
    fn bit_not(self) -> Self {
        !self
    }
    fn is_zero(self) -> bool {
        self == 0x00000000
    }
    fn is_msb_set(self) -> bool {
        (self & 0x80000000) == 0x80000000
    }
    fn is_lsb_set(self) -> bool {
        (self & 0x00000001) == 0x00000001
    }
    fn get_msb_mask(self) -> Self {
        self & 0x80000000
    }
    // fn leading_zeros(self) -> u32 {
    //     self.leading_zeros()
    // }
    fn zero() -> Self {
        0x00000000
    }

    fn get_hex_string(self) -> String {
        format!("{:08x}", self)
    }

    fn get_from_long(value: u32) -> Self {
        value
    }
    fn set_in_long(self, dest: u32) -> u32{
        self
    }

    fn checked_shift_left(self, rhs: u32) -> Option<Self> {
        self.checked_shl(rhs)
    }
    fn checked_shift_right(self, rhs: u32) -> Option<Self> {
        self.checked_shr(rhs)
    }
}

