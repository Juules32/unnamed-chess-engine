use core::fmt;
use std::ops::{Index, IndexMut};

#[derive(PartialEq)]
pub struct Color {
    data: u8
}

impl Color {
    pub const NULL: Color = Color{ data: 0b0 };
    pub const WHITE: Color = Color{ data: 0b01 };
    pub const BLACK: Color = Color{ data: 0b11 };

    #[inline(always)]
    pub fn switch(&mut self) {
        self.data ^= 0b10;

        debug_assert!(*self == Color::WHITE || *self == Color::BLACK);
    }

    #[inline(always)]
    pub fn opposite(&self) -> Color {
        debug_assert!(*self == Color::WHITE || *self == Color::BLACK);

        match *self {
            Color::WHITE => Color::BLACK,
            Color::BLACK => Color::WHITE,
            _ => Color::NULL
        }
    }
}

impl<T, const N: usize> Index<Color> for [T; N] {
    type Output = T;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index.data as usize]
    }
}

impl<T, const N: usize> IndexMut<Color> for [T; N] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index.data as usize]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(match *self {
            Color::NULL => "NULL",
            Color::WHITE => "White",
            Color::BLACK => "Black",
            _ => panic!("Unknown side value!")
        })
    }
}
