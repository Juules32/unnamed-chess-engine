use crate::{piece::PieceType, square::Square};
use core::fmt;
use std::mem::transmute;

const SOURCE_MASK: u16 =  0b0000_0000_0011_1111;
const TARGET_MASK: u16 =  0b0000_1111_1100_0000;
const FLAG_MASK: u16 =    0b1111_0000_0000_0000;

#[derive(Clone, Copy)]
pub struct BitMove(u16);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum MoveFlag {
    None,
    WEnPassant,
    BEnPassant,
    WDoublePawn,
    BDoublePawn,
    WKCastle,
    WQCastle,
    BKCastle,
    BQCastle,
    PromoN,
    PromoB,
    PromoR,
    PromoQ,
}

impl From<u8> for MoveFlag {
    #[inline(always)]
    fn from(number: u8) -> Self {
        unsafe { transmute::<u8, Self>(number) }
    }
}

impl fmt::Display for MoveFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            MoveFlag::None => "None",
            MoveFlag::WDoublePawn => "Double Pawn Push",
            MoveFlag::BDoublePawn => "Double Pawn Push",
            MoveFlag::WEnPassant => "En-passant",
            MoveFlag::BEnPassant => "En-passant",
            MoveFlag::WKCastle => "Kingside Castle",
            MoveFlag::WQCastle => "Queenside Castle",
            MoveFlag::BKCastle => "Kingside Castle",
            MoveFlag::BQCastle => "Queenside Castle",
            MoveFlag::PromoN => "Knight Promotion",
            MoveFlag::PromoB => "Bishop Promotion",
            MoveFlag::PromoR => "Rook Promotion",
            MoveFlag::PromoQ => "Queen Promotion",
        };
        f.pad(name)
    }
}

impl BitMove {
    pub const EMPTY: BitMove = BitMove(0);

    #[inline(always)]
    pub fn source(&self) -> Square {
        Square::from((self.0 & SOURCE_MASK) as u8)
    }

    #[inline(always)]
    pub fn target(&self) -> Square {
        Square::from(((self.0 & TARGET_MASK) >> 6) as u8)
    }

    #[inline(always)]
    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from(((self.0 & FLAG_MASK) >> 12) as u8)
    }

    #[inline(always)]
    pub fn encode(source: Square, target: Square, flag: MoveFlag) -> BitMove {
        BitMove(source as u16 | (target as u16) << 6 | (flag as u16) << 12)
    }

    #[inline(always)]
    pub fn decode(&self) -> (Square, Square, MoveFlag) {
        (self.source(), self.target(), self.flag())
    }

    pub fn to_row_string(self) -> String {
        format!(
            "  | {:<8} | {:<8} | {:<19} |\n",
            self.source(),
            self.target(),
            self.flag()
        )
    }

    pub fn to_uci_string(self) -> String {
        format!(
            "{}{}{}",
            self.source(),
            self.target(),
            match self.flag() {
                MoveFlag::PromoN => "n",
                MoveFlag::PromoB => "b",
                MoveFlag::PromoR => "r",
                MoveFlag::PromoQ => "q",
                _ => "",
            }
        )
    }
}

impl fmt::Display for BitMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!(
            "
  Raw move data: {:b}
  Source Square: {}
  Target Square: {}
  Move Flag:     {}\n",
            self.0,
            self.source(),
            self.target(),
            self.flag()
        ))
    }
}
