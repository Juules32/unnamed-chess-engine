use core::fmt;

use crate::{
    bit_move::{BitMove, MoveFlag},
    bitboard::Bitboard,
    castling_rights::CastlingRights,
    color::Color,
    move_gen,
    piece::PieceType,
    square::Square,
};

pub struct BoardState {
    pub bbs: [Bitboard; 12],
    pub wo: Bitboard,
    pub bo: Bitboard,
    pub ao: Bitboard,
    pub side: Color,
    pub en_passant_sq: Square,
    pub castling_rights: CastlingRights,
}

impl BoardState {
    #[inline(always)]
    pub fn merge_occupancies(&mut self) {
        self.ao = self.wo | self.bo;
    }

    #[inline(always)]
    pub fn populate_occupancies(&mut self) {
        self.wo = self.bbs[PieceType::WP]
                | self.bbs[PieceType::WN]
                | self.bbs[PieceType::WB]
                | self.bbs[PieceType::WR]
                | self.bbs[PieceType::WQ]
                | self.bbs[PieceType::WK];
        self.bo = self.bbs[PieceType::BP]
                | self.bbs[PieceType::BN]
                | self.bbs[PieceType::BB]
                | self.bbs[PieceType::BR]
                | self.bbs[PieceType::BQ]
                | self.bbs[PieceType::BK];

        self.merge_occupancies();
    }

    pub fn starting_position() -> BoardState {
        BoardState {
            bbs: [
                Bitboard::WP,
                Bitboard::WN,
                Bitboard::WB,
                Bitboard::WR,
                Bitboard::WQ,
                Bitboard::WK,
                Bitboard::BP,
                Bitboard::BN,
                Bitboard::BB,
                Bitboard::BR,
                Bitboard::BQ,
                Bitboard::BK,
            ],
            wo: Bitboard::WHITE_PIECES,
            bo: Bitboard::BLACK_PIECES,
            ao: Bitboard::ALL_PIECES,
            side: Color::White,
            en_passant_sq: Square::None,
            castling_rights: CastlingRights::DEFAULT,
        }
    }

    #[inline(always)]
    pub fn set_piece(&mut self, piece: PieceType, sq: Square) {
        self.bbs[piece].set_sq(sq);
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, piece: PieceType, sq: Square) {
        self.bbs[piece].pop_sq(sq);
    }

    #[inline]
    pub fn make_move(&mut self, bit_move: BitMove, castling_rights: CastlingRights) -> bool {
        let (source, target, piece, capture, flag) = bit_move.decode();

        debug_assert_eq!(piece.color(), self.side);
        debug_assert!(capture == PieceType::None || capture.color() == self.side.opposite());
        debug_assert!(self.bbs[piece].is_set_sq(source));
        debug_assert!(capture == PieceType::None || self.bbs[capture].is_set_sq(target));

        // Moves piece
        self.remove_piece(piece, source);
        self.set_piece(piece, target);

        // Removes captured piece
        if capture != PieceType::None {
            self.remove_piece(capture, target);
        }

        // Resets en-passant square
        self.en_passant_sq = Square::None;

        match flag {
            MoveFlag::None => (),
            MoveFlag::WDoublePawn => self.en_passant_sq = target.below(),
            MoveFlag::BDoublePawn => self.en_passant_sq = target.above(),
            MoveFlag::WEnPassant => self.remove_piece(PieceType::BP, target.below()),
            MoveFlag::BEnPassant => self.remove_piece(PieceType::WP, target.above()),
            MoveFlag::WKCastle => {
                self.remove_piece(PieceType::WR, Square::H1);
                self.set_piece(PieceType::WR, Square::F1);
            }
            MoveFlag::WQCastle => {
                self.remove_piece(PieceType::WR, Square::A1);
                self.set_piece(PieceType::WR, Square::D1);
            }
            MoveFlag::BKCastle => {
                self.remove_piece(PieceType::BR, Square::H8);
                self.set_piece(PieceType::BR, Square::F8);
            }
            MoveFlag::BQCastle => {
                self.remove_piece(PieceType::BR, Square::A8);
                self.set_piece(PieceType::BR, Square::D8);
            }
            MoveFlag::PromoQ => {
                self.remove_piece(piece, target);
                self.set_piece(
                    match self.side {
                        Color::White => PieceType::WQ,
                        Color::Black => PieceType::BQ,
                    },
                    target,
                );
            }
            MoveFlag::PromoR => {
                self.remove_piece(piece, target);
                self.set_piece(
                    match self.side {
                        Color::White => PieceType::WR,
                        Color::Black => PieceType::BR,
                    },
                    target,
                );
            }
            MoveFlag::PromoN => {
                self.remove_piece(piece, target);
                self.set_piece(
                    match self.side {
                        Color::White => PieceType::WN,
                        Color::Black => PieceType::BN,
                    },
                    target,
                );
            }
            MoveFlag::PromoB => {
                self.remove_piece(piece, target);
                self.set_piece(
                    match self.side {
                        Color::White => PieceType::WB,
                        Color::Black => PieceType::BB,
                    },
                    target,
                );
            }
        };

        self.castling_rights.update(source, target);
        self.side.switch();
        self.populate_occupancies();

        if self.is_square_attacked(
            if self.side == Color::White {
                self.bbs[PieceType::BK].to_sq()
            } else {
                self.bbs[PieceType::WK].to_sq()
            },
            self.side.opposite(),
            if self.side == Color::White {
                &PieceType::WHITE_PIECES
            } else {
                &PieceType::BLACK_PIECES
            },
        ) {
            self.undo_move(bit_move, castling_rights);
            return false;
        }

        true
    }

    #[inline]
    pub fn undo_move(&mut self, bit_move: BitMove, old_castling_rights: CastlingRights) {
        let (source, target, piece, capture, flag) = bit_move.decode();

        // Switches side first to make it easier to conceptualize
        self.side.switch();

        debug_assert_eq!(piece.color(), self.side);
        debug_assert!(capture == PieceType::None || capture.color() == self.side.opposite());

        self.set_piece(piece, source);
        self.remove_piece(piece, target);

        if capture != PieceType::None {
            self.set_piece(capture, target);
        }

        self.en_passant_sq = Square::None;

        match flag {
            MoveFlag::None | MoveFlag::WDoublePawn | MoveFlag::BDoublePawn => (),
            MoveFlag::WEnPassant => {
                self.en_passant_sq = target;
                self.set_piece(PieceType::BP, target.below())
            }
            MoveFlag::BEnPassant => {
                self.en_passant_sq = target;
                self.set_piece(PieceType::WP, target.above())
            }
            MoveFlag::WKCastle => {
                self.set_piece(PieceType::WR, Square::H1);
                self.remove_piece(PieceType::WR, Square::F1);
            }
            MoveFlag::WQCastle => {
                self.set_piece(PieceType::WR, Square::A1);
                self.remove_piece(PieceType::WR, Square::D1);
            }
            MoveFlag::BKCastle => {
                self.set_piece(PieceType::BR, Square::H8);
                self.remove_piece(PieceType::BR, Square::F8);
            }
            MoveFlag::BQCastle => {
                self.set_piece(PieceType::BR, Square::A8);
                self.remove_piece(PieceType::BR, Square::D8);
            }
            MoveFlag::PromoQ => {
                self.remove_piece(
                    match self.side {
                        Color::White => PieceType::WQ,
                        Color::Black => PieceType::BQ,
                    },
                    target,
                );
            }
            MoveFlag::PromoR => {
                self.remove_piece(
                    match self.side {
                        Color::White => PieceType::WR,
                        Color::Black => PieceType::BR,
                    },
                    target,
                );
            }
            MoveFlag::PromoN => {
                self.remove_piece(
                    match self.side {
                        Color::White => PieceType::WN,
                        Color::Black => PieceType::BN,
                    },
                    target,
                );
            }
            MoveFlag::PromoB => {
                self.remove_piece(
                    match self.side {
                        Color::White => PieceType::WB,
                        Color::Black => PieceType::BB,
                    },
                    target,
                );
            }
        };

        self.castling_rights = old_castling_rights;
        self.populate_occupancies();
    }

    #[inline(always)]
    pub fn is_square_attacked(
        &self,
        square: Square,
        defending_side: Color,
        [enemy_pawn, enemy_knight, enemy_bishop, enemy_rook, enemy_queen, enemy_king]: &[PieceType; 6]
    ) -> bool {
        if (move_gen::get_pawn_capture_mask(defending_side, square) & self.bbs[*enemy_pawn]).is_not_empty() {
            return true;
        }
        if (move_gen::get_knight_mask(square) & self.bbs[*enemy_knight]).is_not_empty() {
            return true;
        }
        if (move_gen::get_bishop_mask(square, self.ao) & self.bbs[*enemy_bishop]).is_not_empty() {
            return true;
        }
        if (move_gen::get_rook_mask(square, self.ao) & self.bbs[*enemy_rook]).is_not_empty() {
            return true;
        }
        if (move_gen::get_queen_mask(square, self.ao) & self.bbs[*enemy_queen]).is_not_empty() {
            return true;
        }
        if (move_gen::get_king_mask(square) & self.bbs[*enemy_king]).is_not_empty() {
            return true;
        }
        false
    }
}

impl Default for BoardState {
    fn default() -> BoardState {
        BoardState {
            bbs: [Bitboard::EMPTY; 12],
            wo: Bitboard::EMPTY,
            bo: Bitboard::EMPTY,
            ao: Bitboard::EMPTY,
            side: Color::White,
            en_passant_sq: Square::None,
            castling_rights: CastlingRights::NONE,
        }
    }
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::from("\n");
        for rank in 0..8_u8 {
            s += &format!("  {}  ", 8 - rank);
            for file in 0..8_u8 {
                let mut is_occupied = false;
                let sq = Square::from(rank * 8 + file);
                for piece_type in PieceType::ALL_PIECES {
                    if Bitboard::is_set_sq(&self.bbs[piece_type], sq) {
                        s += &format!("{} ", piece_type);
                        is_occupied = true;
                    }
                }
                if !is_occupied {
                    s += ". ";
                }
            }
            s += "\n";
        }
        s += &format!(
            "
     a b c d e f g h

     FEN:        {}
     Side        {}
     En-passant: {}
     Castling:   {}\n",
            "Not Implemented",
            self.side,
            self.en_passant_sq,
            self.castling_rights
        );
        f.pad(&s)
    }
}
