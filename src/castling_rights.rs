use crate::square::Square;
use core::fmt;

// Castling right update constants
const INDEX_2_CASTLING_RIGHTS: [u8; 64] = [
    0b0111, 0b1111, 0b1111, 0b1111, 0b0011, 0b1111, 0b1111, 0b1011,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1101, 0b1111, 0b1111, 0b1111, 0b1100, 0b1111, 0b1111, 0b1110
];

pub struct CastlingRights {
    data: u8
}

impl CastlingRights {
    pub const DEFAULT: CastlingRights = CastlingRights{ data: 0b1111 };
    pub const NONE: CastlingRights = CastlingRights{ data: 0b0000 };

    const WK: CastlingRights = CastlingRights{ data: 0b0001 };
    const WQ: CastlingRights = CastlingRights{ data: 0b0010 };
    const BK: CastlingRights = CastlingRights{ data: 0b0100 };
    const BQ: CastlingRights = CastlingRights{ data: 0b1000 };

    #[inline(always)]
    pub fn update(&mut self, source: Square, target: Square) {
        self.data &= INDEX_2_CASTLING_RIGHTS[source] & INDEX_2_CASTLING_RIGHTS[target];
    }

    #[inline(always)]
    pub fn wk(&self) -> bool {
        self.data & CastlingRights::WK.data != 0
    }

    #[inline(always)]
    pub fn wq(&self) -> bool {
        self.data & CastlingRights::WQ.data != 0
    }

    #[inline(always)]
    pub fn bk(&self) -> bool {
        self.data & CastlingRights::BK.data != 0
    }

    #[inline(always)]
    pub fn bq(&self) -> bool {
        self.data & CastlingRights::BQ.data != 0
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let wk = if self.wk() { "K" } else { "-" };
        let wq = if self.wq() { "Q" } else { "-" };
        let bk = if self.bk() { "k" } else { "-" };
        let bq = if self.bq() { "q" } else { "-" };
        write!(f, "{}{}{}{}", wk, wq, bk, bq)
    }
}
