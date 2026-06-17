use super::*;

/// The 8 possible king moves as bit shifts.
const KING_MOVES: [i16; 8] = [-9, -8, -7, -1, 1, 7, 8, 9];

/// Squares the king passes through for castling, relative to king.
/// [0..2] = kingside (f, g), [2..4] = queenside (d, c), [4] = b (empty only, no attack check).
const CASTLING_CHECKS: [i16; 5] = [1, 2, -1, -2, -3];

impl MoveGenerator {
    /// Generates all pseudo-legal king moves for one side, including castling.
    #[inline(always)]
    pub(super) fn get_all_king_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, color);
        let index: u16 = pieces.trailing_zeros() as u16;
        let king: u64 = 1u64 << index;

        // Try every adjacent king offset and reject moves that wrap the file
        // or land on our own pieces.
        for m in KING_MOVES.iter() {
            let (moved_king, target_index) = if m.is_negative() {
                (
                    king >> m.unsigned_abs(),
                    index.wrapping_sub(m.unsigned_abs()),
                )
            } else {
                (
                    king << m.unsigned_abs(),
                    index.wrapping_add(m.unsigned_abs()),
                )
            };

            // A zero bit means the shift ran off the board.
            if moved_king == 0
                || ((target_index % 8) as i16 - (index % 8) as i16).abs() > 1
                || (moved_king & own_pieces) != 0
            {
                continue;
            }

            // The enemy occupancy tells us whether this is a capture move.
            let flag = if moved_king & enemy_pieces != 0 {
                0b0001
            } else {
                0b0000
            };
            available_moves.push(match color {
                Color::White => Move::new(index, target_index, flag),
                Color::Black => Move::new(index ^ 56, target_index ^ 56, flag),
            });
        }

        // Castling — use the real (unswapped) king square for position checks.
        let home_rank = match color {
            Color::White => 0,
            Color::Black => 7,
        };

        let king_sq = Piece::new(color, PieceType::King);
        let king_bb = board.pieces[king_sq as usize];
        let king_real_sq = king_bb.trailing_zeros() as u8;

        if king_real_sq % 8 == 4 && king_real_sq / 8 == home_rank && board.castling_rights != 0 {
            // King must not be in check to castle
            let mut moves = Vec::new();
            if !Self::is_check(board, color, king_bb, &mut moves) {
                let (ks_bit, qs_bit) = match color {
                    Color::White => (Board::WHITE_KINGSIDE, Board::WHITE_QUEENSIDE),
                    Color::Black => (Board::BLACK_KINGSIDE, Board::BLACK_QUEENSIDE),
                };

                // Kingside castling
                if board.castling_rights & ks_bit != 0 {
                    let rook_bb = board.pieces[Piece::new(color, PieceType::Rook) as usize];
                    if rook_bb & (1u64 << (home_rank * 8 + 7)) != 0 {
                        let mut safe = true;
                        for shift in &CASTLING_CHECKS[0..2] {
                            let sq_bb = if shift.is_positive() {
                                king.wrapping_shl(*shift as u32)
                            } else {
                                king.wrapping_shr(shift.unsigned_abs() as u32)
                            };
                            let sq_bb_real = if shift.is_positive() {
                                king_bb.wrapping_shl(*shift as u32)
                            } else {
                                king_bb.wrapping_shr(shift.unsigned_abs() as u32)
                            };
                            if sq_bb & (own_pieces | enemy_pieces) != 0
                                || Self::is_check(board, color, sq_bb_real, &mut moves)
                            {
                                safe = false;
                                break;
                            }
                        }
                        if safe {
                            available_moves.push(match color {
                                Color::White => Move::new(index, index + 2, 0b0100),
                                Color::Black => Move::new(index ^ 56, (index + 2) ^ 56, 0b0100),
                            });
                        }
                    }
                }

                // Queenside castling
                if board.castling_rights & qs_bit != 0 {
                    let rook_bb = board.pieces[Piece::new(color, PieceType::Rook) as usize];
                    if rook_bb & (1u64 << (home_rank * 8)) != 0 {
                        let mut safe = true;
                        for shift in &CASTLING_CHECKS[2..4] {
                            let sq_bb = if shift.is_positive() {
                                king.wrapping_shl(*shift as u32)
                            } else {
                                king.wrapping_shr(shift.unsigned_abs() as u32)
                            };
                            let sq_bb_real = if shift.is_positive() {
                                king_bb.wrapping_shl(*shift as u32)
                            } else {
                                king_bb.wrapping_shr(shift.unsigned_abs() as u32)
                            };
                            if sq_bb & (own_pieces | enemy_pieces) != 0
                                || Self::is_check(board, color, sq_bb_real, &mut moves)
                            {
                                safe = false;
                                break;
                            }
                        }
                        // b8/b1 must be empty but does not need to be unattacked
                        if safe {
                            let b_bb = king.wrapping_shr(3);
                            if b_bb & (own_pieces | enemy_pieces) != 0 {
                                safe = false;
                            }
                        }
                        if safe {
                            available_moves.push(match color {
                                Color::White => Move::new(index, index - 2, 0b0110),
                                Color::Black => Move::new(index ^ 56, (index - 2) ^ 56, 0b0110),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Piece};

    fn get_moves(fen: &str, piece: Piece) -> Vec<String> {
        let board = Board::initialize_from_fen(fen);
        let mut moves = Vec::new();
        MoveGenerator::get_all_king_moves(&board, piece, &mut moves);
        let mut uci: Vec<String> = moves.iter().map(|m| m.to_uci()).collect();
        uci.sort();
        uci
    }

    fn has_move(moves: &[String], m: &str) -> bool {
        moves.contains(&m.to_string())
    }

    #[test]
    fn test_white_king_center_moves() {
        let moves = get_moves("8/8/8/8/3K4/8/8/8 w - - 0 1", Piece::WhiteKing);
        assert!(has_move(&moves, "d4c3"));
        assert!(has_move(&moves, "d4c4"));
        assert!(has_move(&moves, "d4c5"));
        assert!(has_move(&moves, "d4d3"));
        assert!(has_move(&moves, "d4d5"));
        assert!(has_move(&moves, "d4e3"));
        assert!(has_move(&moves, "d4e4"));
        assert!(has_move(&moves, "d4e5"));
        assert_eq!(moves.iter().filter(|m| m.starts_with("d4")).count(), 8);
    }

    #[test]
    fn test_black_king_edge_no_wrap() {
        let moves = get_moves("8/8/8/8/8/8/8/7k b - - 0 1", Piece::BlackKing);
        assert!(has_move(&moves, "h1g1"));
        assert!(has_move(&moves, "h1g2"));
        assert!(has_move(&moves, "h1h2"));
        assert!(!has_move(&moves, "h1a2"));
    }
}
