use std::{any::Any, fmt::Display};

pub trait Memory: Any + Display {
    fn as_any(&self) -> &dyn Any;

    fn get_start_address(&self) -> u32;
    fn get_end_address(&self) -> u32;
    fn get_length(&self) -> usize;

    fn get_long(&self, address: u32) -> u32;
    fn set_long(&mut self, address: u32, value: u32);
    fn get_word(&self, address: u32) -> u16;
    fn set_word(&mut self, address: u32, value: u16);
    fn get_byte(&self, address: u32) -> u8;
    fn set_byte(&mut self, address: u32, value: u8) -> Option<SetMemoryResult>;
}

pub struct SetMemoryResult {
    pub set_overlay: Option<bool>,
}
