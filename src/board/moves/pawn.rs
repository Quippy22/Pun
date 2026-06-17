use super::*;

/// Promotion flag combinations used by pawn move generation.
const PROMOTIONS: [u16; 4] = [0b1000, 0b1010, 0b1100, 0b1110];

impl MoveGenerator {
    /// Generates all pseudo-legal pawn moves for one side.
    #[inline]
    pub(super) fn get_all_pawn_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, color);
        let mut index: u16;
        let mut pawn: u64;
        let mut is_promotion: bool;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            pawn = 1u64 << index;
            is_promotion = index / 8 == 6;

            let mut flag = 0b0000;

            // Forward moves are only legal if the square ahead is empty.
            if pawn << 8 & (own_pieces | enemy_pieces) == 0 {
                // Promotion moves are emitted as four separate moves so the
                // caller can pick the promoted piece later.
                if is_promotion {
                    for p in PROMOTIONS.iter() {
                        available_moves.push(match color {
                            Color::White => Move::new(index, index + 8, flag | p),
                            Color::Black => Move::new(index ^ 56, (index ^ 56) - 8, flag | p),
                        });
                    }
                } else {
                    available_moves.push(match color {
                        Color::White => Move::new(index, index + 8, flag),
                        Color::Black => Move::new(index ^ 56, (index ^ 56) - 8, flag),
                    });
                }

                // A double push is only available from the starting rank and
                // only when both squares are empty.
                if index / 8 == 1 && pawn << 16 & (own_pieces | enemy_pieces) == 0 {
                    available_moves.push(match color {
                        Color::White => Move::new(index, index + 16, flag),
                        Color::Black => Move::new(index ^ 56, (index ^ 56) - 16, flag),
                    });
                }
            }

            // Capture moves are marked separately from quiet moves.
            flag |= 0b0001;

            let file = index % 8;

            // Left capture means "toward the A-file" from White's perspective.
            if file != 0 && (pawn << 7) & enemy_pieces != 0 {
                if is_promotion {
                    for p in PROMOTIONS.iter() {
                        available_moves.push(match color {
                            Color::White => Move::new(index, index + 7, flag | p),
                            Color::Black => Move::new(index ^ 56, (index ^ 56) - 9, flag | p),
                        });
                    }
                } else {
                    available_moves.push(match color {
                        Color::White => Move::new(index, index + 7, flag),
                        Color::Black => Move::new(index ^ 56, (index ^ 56) - 9, flag),
                    });
                }
            }

            // Right capture means "toward the H-file" from White's perspective.
            if file != 7 && (pawn << 9) & enemy_pieces != 0 {
                if is_promotion {
                    for p in PROMOTIONS.iter() {
                        available_moves.push(match color {
                            Color::White => Move::new(index, index + 9, flag | p),
                            Color::Black => Move::new(index ^ 56, (index ^ 56) - 7, flag | p),
                        });
                    }
                } else {
                    available_moves.push(match color {
                        Color::White => Move::new(index, index + 9, flag),
                        Color::Black => Move::new(index ^ 56, (index ^ 56) - 7, flag),
                    });
                }
            }

            // En passant: the enemy pawn just double-pushed and landed next to us.
            if let Some(ep_sq) = board.en_passant_sq {
                let ep_sq = ep_sq as u16;
                let ep_target = match color {
                    Color::White => {
                        if index / 8 == 4 && file != 0 && ep_sq == index + 7 {
                            Some((index, index + 7))
                        } else if index / 8 == 4 && file != 7 && ep_sq == index + 9 {
                            Some((index, index + 9))
                        } else {
                            None
                        }
                    }
                    Color::Black => {
                        let raw = index ^ 56;
                        if raw / 8 == 3 && file != 0 && ep_sq == raw - 9 {
                            Some((raw, raw - 9))
                        } else if raw / 8 == 3 && file != 7 && ep_sq == raw - 7 {
                            Some((raw, raw - 7))
                        } else {
                            None
                        }
                    }
                };

                if let Some((from, to)) = ep_target {
                    available_moves.push(Move::new(from, to, 0b0011));
                }
            }

            // clear the bit
            pieces &= pieces - 1;
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
        MoveGenerator::get_all_pawn_moves(&board, piece, &mut moves);
        let mut uci: Vec<String> = moves.iter().map(|m| m.to_uci()).collect();
        uci.sort();
        uci
    }

    fn has_move(moves: &[String], m: &str) -> bool {
        moves.contains(&m.to_string())
    }

    mod forward_moves {
        use super::*;

        #[test]
        fn test_white_pawn_single_push() {
            let moves = get_moves("8/8/8/8/8/8/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e2e3"));
        }

        #[test]
        fn test_white_pawn_double_push_from_rank2() {
            let moves = get_moves("8/8/8/8/8/8/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e2e4"));
        }

        #[test]
        fn test_white_pawn_no_double_push_from_rank3() {
            let moves = get_moves("8/8/8/8/8/4P3/8/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e3e4"));
            assert!(!has_move(&moves, "e3e5"));
        }

        #[test]
        fn test_black_pawn_single_push() {
            let moves = get_moves("8/4p3/8/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e7e6"));
        }

        #[test]
        fn test_black_pawn_double_push_from_rank7() {
            let moves = get_moves("8/4p3/8/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e7e5"));
        }

        #[test]
        fn test_black_pawn_no_double_push_from_rank6() {
            let moves = get_moves("8/8/4p3/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e6e5"));
            assert!(!has_move(&moves, "e6e4"));
        }
    }

    mod blocked {
        use super::*;

        #[test]
        fn test_white_pawn_blocked_single_push() {
            let moves = get_moves("8/8/8/8/8/4p3/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(!has_move(&moves, "e2e3"));
            assert!(!has_move(&moves, "e2e4"));
        }

        #[test]
        fn test_white_pawn_blocked_double_push_by_piece_on_rank4() {
            let moves = get_moves("8/8/8/8/4p3/8/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e2e3"));
            assert!(!has_move(&moves, "e2e4"));
        }

        #[test]
        fn test_black_pawn_blocked_single_push() {
            let moves = get_moves("8/4p3/4P3/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(!has_move(&moves, "e7e6"));
            assert!(!has_move(&moves, "e7e5"));
        }

        #[test]
        fn test_black_pawn_blocked_double_push_by_piece_on_rank5() {
            let moves = get_moves("8/4p3/8/4P3/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e7e6"));
            assert!(!has_move(&moves, "e7e5"));
        }
    }

    mod captures {
        use super::*;

        #[test]
        fn test_white_pawn_capture_left_only() {
            let moves = get_moves("8/8/8/8/8/3p4/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e2d3"));
            assert!(!has_move(&moves, "e2f3"));
        }

        #[test]
        fn test_white_pawn_capture_right_only() {
            let moves = get_moves("8/8/8/8/8/5p2/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e2f3"));
            assert!(!has_move(&moves, "e2d3"));
        }

        #[test]
        fn test_white_pawn_capture_both_sides() {
            let moves = get_moves("8/8/8/8/8/3p1p2/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e2d3"));
            assert!(has_move(&moves, "e2f3"));
        }

        #[test]
        fn test_white_pawn_cannot_capture_own_pieces() {
            let moves = get_moves("8/8/8/8/8/3P1P2/4P3/8 w - - 0 1", Piece::WhitePawn);
            assert!(!has_move(&moves, "e2d3"));
            assert!(!has_move(&moves, "e2f3"));
        }

        #[test]
        fn test_black_pawn_capture_both_sides() {
            let moves = get_moves("8/4p3/3P1P2/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e7d6"));
            assert!(has_move(&moves, "e7f6"));
        }

        #[test]
        fn test_black_pawn_cannot_capture_own_pieces() {
            let moves = get_moves("8/4p3/3p1p2/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(!has_move(&moves, "e7d6"));
            assert!(!has_move(&moves, "e7f6"));
        }
    }

    mod edge_files {
        use super::*;

        #[test]
        fn test_white_pawn_a_file_no_left_wrap() {
            let moves = get_moves("8/8/8/8/8/1p6/P7/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "a2b3"));
            assert!(!has_move(&moves, "a2h3"));
        }

        #[test]
        fn test_white_pawn_h_file_no_right_wrap() {
            let moves = get_moves("8/8/8/8/8/6p1/7P/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "h2g3"));
            assert!(!has_move(&moves, "h2a3"));
        }

        #[test]
        fn test_black_pawn_a_file_no_left_wrap() {
            let moves = get_moves("8/p7/1P6/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "a7b6"));
            assert!(!has_move(&moves, "a7h6"));
        }

        #[test]
        fn test_black_pawn_h_file_no_right_wrap() {
            let moves = get_moves("8/7p/6P1/8/8/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "h7g6"));
            assert!(!has_move(&moves, "h7a6"));
        }
    }

    mod promotions {
        use super::*;

        #[test]
        fn test_white_pawn_promotion_all_four_pieces() {
            let moves = get_moves("8/4P3/8/8/8/8/8/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e7e8q"));
            assert!(has_move(&moves, "e7e8r"));
            assert!(has_move(&moves, "e7e8b"));
            assert!(has_move(&moves, "e7e8n"));
            assert_eq!(moves.iter().filter(|m| m.starts_with("e7e8")).count(), 4);
        }

        #[test]
        fn test_white_pawn_blocked_promotion() {
            let moves = get_moves("4r3/4P3/8/8/8/8/8/8 w - - 0 1", Piece::WhitePawn);
            assert!(!has_move(&moves, "e7e8q"));
            assert!(!has_move(&moves, "e7e8r"));
            assert!(!has_move(&moves, "e7e8b"));
            assert!(!has_move(&moves, "e7e8n"));
        }

        #[test]
        fn test_white_pawn_promotion_capture_left() {
            let moves = get_moves("3r4/4P3/8/8/8/8/8/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e7d8q"));
            assert!(has_move(&moves, "e7d8r"));
            assert!(has_move(&moves, "e7d8b"));
            assert!(has_move(&moves, "e7d8n"));
        }

        #[test]
        fn test_white_pawn_promotion_capture_right() {
            let moves = get_moves("5r2/4P3/8/8/8/8/8/8 w - - 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e7f8q"));
            assert!(has_move(&moves, "e7f8r"));
            assert!(has_move(&moves, "e7f8b"));
            assert!(has_move(&moves, "e7f8n"));
        }

        #[test]
        fn test_black_pawn_promotion_all_four_pieces() {
            let moves = get_moves("8/8/8/8/8/8/4p3/8 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e2e1q"));
            assert!(has_move(&moves, "e2e1r"));
            assert!(has_move(&moves, "e2e1b"));
            assert!(has_move(&moves, "e2e1n"));
            assert_eq!(moves.iter().filter(|m| m.starts_with("e2e1")).count(), 4);
        }

        #[test]
        fn test_black_pawn_promotion_capture_left() {
            let moves = get_moves("8/8/8/8/8/8/4p3/3R4 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e2d1q"));
            assert!(has_move(&moves, "e2d1r"));
            assert!(has_move(&moves, "e2d1b"));
            assert!(has_move(&moves, "e2d1n"));
        }

        #[test]
        fn test_black_pawn_promotion_capture_right() {
            let moves = get_moves("8/8/8/8/8/8/4p3/5R2 b - - 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "e2f1q"));
            assert!(has_move(&moves, "e2f1r"));
            assert!(has_move(&moves, "e2f1b"));
            assert!(has_move(&moves, "e2f1n"));
        }
    }

    mod en_passant {
        use super::*;

        #[test]
        fn test_white_pawn_en_passant_left() {
            // White pawn e5, black pawn just double-pushed to d5, en passant target d6
            let moves = get_moves("8/8/8/3pP3/8/8/8/8 w - d6 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e5d6"));
        }

        #[test]
        fn test_white_pawn_en_passant_right() {
            // White pawn e5, black pawn just double-pushed to f5, en passant target f6
            let moves = get_moves("8/8/8/4Pp2/8/8/8/8 w - f6 0 1", Piece::WhitePawn);
            assert!(has_move(&moves, "e5f6"));
        }

        #[test]
        fn test_white_pawn_no_en_passant_without_target() {
            // No en passant square set
            let moves = get_moves("8/8/8/3pP3/8/8/8/8 w - - 0 1", Piece::WhitePawn);
            assert!(!has_move(&moves, "e5d6"));
        }

        #[test]
        fn test_white_pawn_no_en_passant_wrong_rank() {
            // White pawn on rank 4, en passant target on rank 6 — too far
            let moves = get_moves("8/8/8/8/3Pp3/8/8/8 w - f3 0 1", Piece::WhitePawn);
            assert!(!has_move(&moves, "d4f3"));
        }

        #[test]
        fn test_white_pawn_no_en_passant_own_file() {
            // White pawn e5, en passant on e6 — same file, not diagonal
            let moves = get_moves("8/8/8/4p3/4P3/8/8/8 w - e6 0 1", Piece::WhitePawn);
            assert!(!has_move(&moves, "e5e6"));
        }

        #[test]
        fn test_black_pawn_en_passant_left() {
            // Black pawn d4, white pawn just double-pushed to e4, en passant target e3
            let moves = get_moves("8/8/8/8/3pP3/8/8/8 b - e3 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "d4e3"));
        }

        #[test]
        fn test_black_pawn_en_passant_right() {
            // Black pawn d4, white pawn just double-pushed to c4, en passant target c3
            let moves = get_moves("8/8/8/8/2Pp4/8/8/8 b - c3 0 1", Piece::BlackPawn);
            assert!(has_move(&moves, "d4c3"));
        }

        #[test]
        fn test_black_pawn_no_en_passant_without_target() {
            // No en passant square set
            let moves = get_moves("8/8/8/8/3pP3/8/8/8 b - - 0 1", Piece::BlackPawn);
            assert!(!has_move(&moves, "d4e3"));
        }
    }
}
