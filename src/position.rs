use core::fmt;

use crate::{
    bit_move::{BitMove, MoveFlag}, bitboard::Bitboard, castling_rights::CastlingRights, color::Color, move_masks, move_list::MoveList, piece::PieceType, rank::Rank, square::Square
};

#[derive(Clone)]
pub struct Position {
    pub bbs: [Bitboard; 12],
    pub wo: Bitboard,
    pub bo: Bitboard,
    pub ao: Bitboard,
    pub side: Color,
    pub en_passant_sq: Square,
    pub castling_rights: CastlingRights,
}

impl Position {
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

    pub fn starting_position() -> Position {
        Position {
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
            wo: Bitboard::WHITE_STARTING_PIECES,
            bo: Bitboard::BLACK_STARTING_PIECES,
            ao: Bitboard::ALL_STARTING_PIECES,
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
    pub fn make_move(&mut self, bit_move: BitMove) -> bool {
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
        if (move_masks::get_pawn_capture_mask(defending_side, square) & self.bbs[*enemy_pawn]).is_not_empty() {
            return true;
        }
        if (move_masks::get_knight_mask(square) & self.bbs[*enemy_knight]).is_not_empty() {
            return true;
        }
        if (move_masks::get_bishop_mask(square, self.ao) & self.bbs[*enemy_bishop]).is_not_empty() {
            return true;
        }
        if (move_masks::get_rook_mask(square, self.ao) & self.bbs[*enemy_rook]).is_not_empty() {
            return true;
        }
        if (move_masks::get_queen_mask(square, self.ao) & self.bbs[*enemy_queen]).is_not_empty() {
            return true;
        }
        if (move_masks::get_king_mask(square) & self.bbs[*enemy_king]).is_not_empty() {
            return true;
        }
        false
    }
    
    // Based on side, relevant pieces and occupancies can be selected
    #[inline]
    pub fn generate_moves(self: &Position) -> MoveList {
        let mut move_list = MoveList::default();
        
        let side = self.side;
        let en_passant_sq = self.en_passant_sq;
        let inv_all_occupancies = !self.ao;
        
        let ([pawn, knight, bishop, rook, queen, king], enemy_pieces) = match side {
            Color::White => (PieceType::WHITE_PIECES, PieceType::BLACK_PIECES),
            Color::Black => (PieceType::BLACK_PIECES, PieceType::WHITE_PIECES)
        };

        let (inv_own_occupancies, enemy_occupancies) = match side {
            Color::White => (!self.wo, self.bo),
            Color::Black => (!self.bo, self.wo)
        };
        
        let (pawn_promotion_rank, pawn_starting_rank, en_passant_rank, pawn_double_push_rank) = match side {
            Color::White => (Rank::R7, Rank::R2, Rank::R5, Rank::R4),
            Color::Black => (Rank::R2, Rank::R7, Rank::R4, Rank::R5)
        };
        
        let (double_pawn_flag, en_passant_flag, king_side_castling_flag, queen_side_castling_flag) = match side {
            Color::White => (MoveFlag::WDoublePawn, MoveFlag::WEnPassant, MoveFlag::WKCastle, MoveFlag::WQCastle),
            Color::Black => (MoveFlag::BDoublePawn, MoveFlag::BEnPassant, MoveFlag::BKCastle, MoveFlag::BQCastle)
        };

        let (king_side_castling_mask, queen_side_castling_mask) = match side {
            Color::White => (Bitboard::W_KING_SIDE_MASK, Bitboard::W_QUEEN_SIDE_MASK),
            Color::Black => (Bitboard::B_KING_SIDE_MASK, Bitboard::B_QUEEN_SIDE_MASK)
        };

        let (king_side_castling_right, queen_side_castling_right) = match side {
            Color::White => (self.castling_rights.wk(), self.castling_rights.wq()),
            Color::Black => (self.castling_rights.bk(), self.castling_rights.bq())
        };

        let (castling_square_c, castling_square_d, castling_square_e, castling_square_f, castling_square_g) = match side {
            Color::White => (Square::C1, Square::D1, Square::E1, Square::F1, Square::G1),
            Color::Black => (Square::C8, Square::D8, Square::E8, Square::F8, Square::G8)
        };

        {
            /*------------------------------*\ 
                        Pawn moves
            \*------------------------------*/
            let mut pawn_bb = self.bbs[pawn];
            while pawn_bb.is_not_empty() {
                let source = pawn_bb.pop_lsb();
                let source_rank = source.rank();

                // Captures
                let mut capture_mask = move_masks::get_pawn_capture_mask(side, source) & enemy_occupancies;
                while capture_mask.is_not_empty() {
                    let target = capture_mask.pop_lsb();
                    let target_piece = self.get_target_piece(enemy_pieces, target);

                    if source_rank == pawn_promotion_rank {
                        move_list.add(BitMove::encode(source, target, pawn, target_piece, MoveFlag::PromoN));
                        move_list.add(BitMove::encode(source, target, pawn, target_piece, MoveFlag::PromoB));
                        move_list.add(BitMove::encode(source, target, pawn, target_piece, MoveFlag::PromoR));
                        move_list.add(BitMove::encode(source, target, pawn, target_piece, MoveFlag::PromoQ));
                    } else {
                        move_list.add(BitMove::encode(source, target, pawn, target_piece, MoveFlag::None));
                    }
                }

                // Quiet moves
                let mut quiet_mask = move_masks::get_pawn_quiet_mask(side, source) & inv_all_occupancies;
                while quiet_mask.is_not_empty() {
                    let target = quiet_mask.pop_lsb();
                    
                    if source_rank == pawn_starting_rank && target.rank() == pawn_double_push_rank {
                        // Making sure both squares in front of the pawn are empty
                        if (move_masks::get_pawn_quiet_mask(side, source) & self.ao).is_empty() {
                            move_list.add(BitMove::encode(source, target, pawn, PieceType::None, double_pawn_flag));
                        } 
                    } else if source_rank == pawn_promotion_rank {
                        move_list.add(BitMove::encode(source, target, pawn, PieceType::None, MoveFlag::PromoN));
                        move_list.add(BitMove::encode(source, target, pawn, PieceType::None, MoveFlag::PromoB));
                        move_list.add(BitMove::encode(source, target, pawn, PieceType::None, MoveFlag::PromoR));
                        move_list.add(BitMove::encode(source, target, pawn, PieceType::None, MoveFlag::PromoQ));
                    } else {
                        move_list.add(BitMove::encode(source, target, pawn, PieceType::None, MoveFlag::None));
                    }
                }
                
                // En-passant (could maybe be combined with captures?)
                if en_passant_sq != Square::None && source_rank == en_passant_rank {
                    let mut en_passant_mask = move_masks::get_pawn_capture_mask(side, source);
                    while en_passant_mask.is_not_empty() {
                        let target = en_passant_mask.pop_lsb();
                        if target == en_passant_sq {
                            move_list.add(BitMove::encode(source, target, pawn, PieceType::None, en_passant_flag));
                        }
                    }
                }
            }
        }

        {
            /*------------------------------*\ 
                    Knight moves
            \*------------------------------*/
            let mut knight_bb = self.bbs[knight];
            while knight_bb.is_not_empty() {
                let source = knight_bb.pop_lsb();
                
                let mut move_mask = move_masks::get_knight_mask(source) & inv_own_occupancies;
                while move_mask.is_not_empty() {
                    let target = move_mask.pop_lsb();
                    let target_piece = self.get_target_piece_if_any(enemy_pieces, enemy_occupancies, target);
                    move_list.add(BitMove::encode(source, target, knight, target_piece, MoveFlag::None));
                }
            }
        }

        {
            /*------------------------------*\ 
                        King moves
            \*------------------------------*/
            let mut king_bb = self.bbs[king];
            let source = king_bb.pop_lsb();
            let mut move_mask = move_masks::get_king_mask(source) & inv_own_occupancies;
            while move_mask.is_not_empty() {
                let target = move_mask.pop_lsb();
                let target_piece = self.get_target_piece_if_any(enemy_pieces, enemy_occupancies, target);
                move_list.add(BitMove::encode(source, target, king, target_piece, MoveFlag::None));
            }

            // Kingside Castling
            #[allow(clippy::collapsible_if)]
            if king_side_castling_right && (self.ao & king_side_castling_mask).is_empty() {
                if !self.is_square_attacked(castling_square_e, self.side, &enemy_pieces) &&
                !self.is_square_attacked(castling_square_f, self.side, &enemy_pieces) &&
                !self.is_square_attacked(castling_square_g, self.side, &enemy_pieces)
                {
                    move_list.add(BitMove::encode(source, castling_square_g, king, PieceType::None, king_side_castling_flag));
                }
            }

            // Queenside Castling
            #[allow(clippy::collapsible_if)]
            if queen_side_castling_right && (self.ao & queen_side_castling_mask).is_empty() {
                if !self.is_square_attacked(castling_square_e, self.side, &enemy_pieces) &&
                !self.is_square_attacked(castling_square_d, self.side, &enemy_pieces) &&
                !self.is_square_attacked(castling_square_c, self.side, &enemy_pieces)
                {
                    move_list.add(BitMove::encode(source, castling_square_c, king, PieceType::None, queen_side_castling_flag));
                }
            }
        }

        {
            /*------------------------------*\ 
                    Bishop moves
            \*------------------------------*/
            let mut bishop_bb = self.bbs[bishop];
            while bishop_bb.is_not_empty() {
                let source = bishop_bb.pop_lsb();
                let mut move_mask = move_masks::get_bishop_mask(source, self.ao) & inv_own_occupancies;
                while move_mask.is_not_empty() {
                    let target = move_mask.pop_lsb();
                    let target_piece = self.get_target_piece_if_any(enemy_pieces, enemy_occupancies, target);
                    move_list.add(BitMove::encode(source, target, bishop, target_piece, MoveFlag::None));
                }
            }
        }

        {
            /*------------------------------*\ 
                        Rook moves
            \*------------------------------*/
            let mut rook_bb = self.bbs[rook];
            while rook_bb.is_not_empty() {
                let source = rook_bb.pop_lsb();
                let mut move_mask = move_masks::get_rook_mask(source, self.ao) & inv_own_occupancies;
                while move_mask.is_not_empty() {
                    let target = move_mask.pop_lsb();
                    let target_piece = self.get_target_piece_if_any(enemy_pieces, enemy_occupancies, target);
                    move_list.add(BitMove::encode(source, target, rook, target_piece, MoveFlag::None));
                }
            }
        }

        {
            /*------------------------------*\ 
                    Queen moves
            \*------------------------------*/
            let mut queen_bb = self.bbs[queen];
            while queen_bb.is_not_empty() {
                let source = queen_bb.pop_lsb();
                let mut move_mask = move_masks::get_queen_mask(source, self.ao) & inv_own_occupancies;
                while move_mask.is_not_empty() {
                    let target = move_mask.pop_lsb();
                    let target_piece = self.get_target_piece_if_any(enemy_pieces, enemy_occupancies, target);
                    move_list.add(BitMove::encode(source, target, queen, target_piece, MoveFlag::None));
                }
            }
        }

        // debug: all moves are different

        move_list
    }

    #[inline(always)]
    pub fn get_target_piece(&self, enemy_piece_types: [PieceType; 6], target: Square) -> PieceType {
        for piece_type in enemy_piece_types {
            if self.bbs[piece_type].is_set_sq(target) {
                return piece_type;
            }
        }

        panic!("There seems to be something wrong with the occupancy bitboards!")
    }


    #[inline(always)]
    pub fn get_target_piece_if_any(&self, enemy_piece_types: [PieceType; 6], enemy_occupancies: Bitboard, target: Square) -> PieceType {
        if (enemy_occupancies & target.to_bb()).is_empty() {
            return PieceType::None;
        }
        
        self.get_target_piece(enemy_piece_types, target)
    }

}

impl Default for Position {
    fn default() -> Position {
        Position {
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

impl fmt::Display for Position {
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
