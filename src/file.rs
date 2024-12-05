use core::fmt;
use std::mem::transmute;

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum File {
    FA = 0,
    FB = 1,
    FC = 2,
    FD = 3,
    FE = 4,
    FF = 5,
    FG = 6,
    FH = 7,
}

impl From<u8> for File {
    #[inline(always)]
    fn from(number: u8) -> Self {
        unsafe { transmute::<u8, Self>(number) }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f_char = (b'a' + *self as u8) as char;
        f.pad(&f_char.to_string())
    }
}
