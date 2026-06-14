use super::*;

/// The 8 possible knight moves as bit shifts
const KNIGHT_MOVES: [i16; 8] = [17, 15, 10, 6, -17, -15, -10, -6];

impl MoveGenerator {
    #[inline(always)]
    pub(super) fn get_all_knight_moves(
        board: &Board,
        piece: Piece,
        available_moves: &mut Vec<Move>,
    ) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let mut index: u16;
        let mut knight: u64;
        let (own_pieces, enemy_pieces) = Self::get_sides(board, color);

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            knight = 1 << index;

            // check all the possible moves
            for m in KNIGHT_MOVES.iter() {
                let (moved_knight, target_index) = if m.is_negative() {
                    (
                        knight >> m.unsigned_abs(),
                        index.wrapping_sub(m.unsigned_abs()),
                    )
                } else {
                    (
                        knight << m.unsigned_abs(),
                        index.wrapping_add(m.unsigned_abs()),
                    )
                };
                // check if the move is valid
                if moved_knight == 0
                    || ((target_index % 8) as i16 - (index % 8) as i16).abs() > 2
                    || (moved_knight & own_pieces) != 0
                {
                    continue;
                }

                // check for capture
                let flag = if moved_knight & enemy_pieces != 0 {
                    0b0001
                } else {
                    0b0000
                };
                available_moves.push(match color {
                    Color::White => Move::new(index, target_index, flag),
                    Color::Black => Move::new(index ^ 56, target_index ^ 56, flag),
                });
            }

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
        MoveGenerator::get_all_knight_moves(&board, piece, &mut moves);
        let mut uci: Vec<String> = moves.iter().map(|m| m.to_uci()).collect();
        uci.sort();
        uci
    }

    fn has_move(moves: &[String], m: &str) -> bool {
        moves.contains(&m.to_string())
    }

    mod center_moves {
        use super::*;

        #[test]
        fn test_white_knight_center_all_moves() {
            let moves = get_moves("8/8/8/8/3N4/8/8/8 w - - 0 1", Piece::WhiteKnight);
            assert!(has_move(&moves, "d4b3"));
            assert!(has_move(&moves, "d4b5"));
            assert!(has_move(&moves, "d4c2"));
            assert!(has_move(&moves, "d4c6"));
            assert!(has_move(&moves, "d4e2"));
            assert!(has_move(&moves, "d4e6"));
            assert!(has_move(&moves, "d4f3"));
            assert!(has_move(&moves, "d4f5"));
            assert_eq!(moves.iter().filter(|m| m.starts_with("d4")).count(), 8);
        }

        #[test]
        fn test_black_knight_center_all_moves() {
            let moves = get_moves("8/8/8/8/3n4/8/8/8 b - - 0 1", Piece::BlackKnight);
            assert!(has_move(&moves, "d4b3"));
            assert!(has_move(&moves, "d4b5"));
            assert!(has_move(&moves, "d4c2"));
            assert!(has_move(&moves, "d4c6"));
            assert!(has_move(&moves, "d4e2"));
            assert!(has_move(&moves, "d4e6"));
            assert!(has_move(&moves, "d4f3"));
            assert!(has_move(&moves, "d4f5"));
            assert_eq!(moves.iter().filter(|m| m.starts_with("d4")).count(), 8);
        }
    }

    mod captures {
        use super::*;

        #[test]
        fn test_white_knight_capture_single_target() {
            let moves = get_moves("8/8/8/8/3N4/5p2/8/8 w - - 0 1", Piece::WhiteKnight);
            assert!(has_move(&moves, "d4f3"));
        }

        #[test]
        fn test_black_knight_capture_single_target() {
            let moves = get_moves("8/8/8/5P2/3n4/8/8/8 b - - 0 1", Piece::BlackKnight);
            assert!(has_move(&moves, "d4f5"));
        }

        #[test]
        fn test_white_knight_capture_both_sides() {
            let moves = get_moves("8/8/8/5p2/3N4/5p2/8/8 w - - 0 1", Piece::WhiteKnight);
            assert!(has_move(&moves, "d4f5"));
            assert!(has_move(&moves, "d4f3"));
        }

        #[test]
        fn test_black_knight_capture_both_sides() {
            let moves = get_moves("8/8/8/5P2/3n4/5P2/8/8 b - - 0 1", Piece::BlackKnight);
            assert!(has_move(&moves, "d4f5"));
            assert!(has_move(&moves, "d4f3"));
        }
    }

    mod blocked {
        use super::*;

        #[test]
        fn test_white_knight_blocked_by_own_piece() {
            let moves = get_moves("8/8/8/8/3N4/5P2/8/8 w - - 0 1", Piece::WhiteKnight);
            assert!(!has_move(&moves, "d4f3"));
            assert!(has_move(&moves, "d4e2"));
        }

        #[test]
        fn test_black_knight_blocked_by_own_piece() {
            let moves = get_moves("8/8/8/5p2/3n4/8/8/8 b - - 0 1", Piece::BlackKnight);
            assert!(!has_move(&moves, "d4f5"));
            assert!(has_move(&moves, "d4e6"));
        }
    }

    mod edge_files {
        use super::*;

        #[test]
        fn test_white_knight_a_file_no_wrap() {
            let moves = get_moves("8/8/8/8/8/8/8/N7 w - - 0 1", Piece::WhiteKnight);
            assert!(has_move(&moves, "a1b3"));
            assert!(has_move(&moves, "a1c2"));
            assert!(!has_move(&moves, "a1g2"));
            assert!(!has_move(&moves, "a1h3"));
        }

        #[test]
        fn test_white_knight_h_file_no_wrap() {
            let moves = get_moves("8/8/8/8/8/8/8/7N w - - 0 1", Piece::WhiteKnight);
            assert!(has_move(&moves, "h1f2"));
            assert!(has_move(&moves, "h1g3"));
            assert!(!has_move(&moves, "h1a2"));
            assert!(!has_move(&moves, "h1b3"));
        }

        #[test]
        fn test_black_knight_a_file_no_wrap() {
            let moves = get_moves("n7/8/8/8/8/8/8/8 b - - 0 1", Piece::BlackKnight);
            assert!(has_move(&moves, "a8b6"));
            assert!(has_move(&moves, "a8c7"));
            assert!(!has_move(&moves, "a8g6"));
            assert!(!has_move(&moves, "a8h7"));
        }

        #[test]
        fn test_black_knight_h_file_no_wrap() {
            let moves = get_moves("7n/8/8/8/8/8/8/8 b - - 0 1", Piece::BlackKnight);
            assert!(has_move(&moves, "h8f7"));
            assert!(has_move(&moves, "h8g6"));
            assert!(!has_move(&moves, "h8a7"));
            assert!(!has_move(&moves, "h8b6"));
        }
    }
}
