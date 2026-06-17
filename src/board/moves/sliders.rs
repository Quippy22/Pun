use super::*;

/// The 4 possible bishop directions.
const BISHOP_DIRECTIONS: [i16; 4] = [-9, -7, 7, 9];
/// The 4 possible rook directions.
const ROOK_DIRECTIONS: [i16; 4] = [-1, 1, -8, 8];

impl MoveGenerator {
    /// Shared raycaster for bishop, rook, and queen moves.
    ///
    /// The helper walks each direction one step at a time, stopping when it
    /// hits the edge of the board, one of our own pieces, or a capture.
    #[inline(always)]
    fn raycast_moves(
        index: u16,
        directions: &[i16],
        own_pieces: u64,
        enemy_pieces: u64,
        color: Color,
        is_diagonal: bool,
        available_moves: &mut Vec<Move>,
    ) {
        // Each bit tracks whether a ray is still alive.
        let mut active_directions = 0b1111u8;
        // Each direction gets its own current square index.
        let mut current_indices = [index as i16; 4];

        for _ in 1..=7 {
            if active_directions == 0 {
                break;
            }

            for i in 0..directions.len() {
                if (active_directions & (1 << i)) == 0 {
                    continue;
                }

                let prev_file = current_indices[i] % 8;
                current_indices[i] += directions[i];
                let current_idx = current_indices[i];

                if !(0..=63).contains(&current_idx) {
                    active_directions &= !(1 << i);
                    continue;
                }

                let current_file = current_idx % 8;
                if is_diagonal {
                    // Diagonal rays must move exactly one file per step.
                    if (current_file - prev_file).abs() != 1 {
                        active_directions &= !(1 << i);
                        continue;
                    }
                } else if directions[i].abs() >= 8 {
                    // Vertical rays must stay on the same file.
                    if current_file != prev_file {
                        active_directions &= !(1 << i);
                        continue;
                    }
                } else if (current_file - prev_file).abs() != 1 {
                    // Horizontal rays must move exactly one file per step.
                    active_directions &= !(1 << i);
                    continue;
                }

                let target_index = current_idx as u16;
                let target_bit = 1u64 << target_index;

                // Own pieces block the ray without generating a move.
                if (target_bit & own_pieces) != 0 {
                    active_directions &= !(1 << i);
                    continue;
                }

                // Enemy pieces are capturable, but also terminate the ray.
                let is_capture = (target_bit & enemy_pieces) != 0;
                let flag = if is_capture { 0b0001 } else { 0b0000 };

                available_moves.push(match color {
                    Color::White => Move::new(index, target_index, flag),
                    Color::Black => Move::new(index ^ 56, target_index ^ 56, flag),
                });

                if is_capture {
                    active_directions &= !(1 << i);
                }
            }
        }
    }

    pub(super) fn get_all_bishop_moves(
        board: &Board,
        piece: Piece,
        available_moves: &mut Vec<Move>,
    ) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, color);
        let mut index: u16;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            Self::raycast_moves(
                index,
                &BISHOP_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                color,
                true,
                available_moves,
            );

            pieces &= pieces - 1;
        }
    }

    pub(super) fn get_all_rook_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, color);
        let mut index: u16;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            Self::raycast_moves(
                index,
                &ROOK_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                color,
                false,
                available_moves,
            );

            pieces &= pieces - 1;
        }
    }

    pub(super) fn get_all_queen_moves(
        board: &Board,
        piece: Piece,
        available_moves: &mut Vec<Move>,
    ) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, color);
        let mut index: u16;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            Self::raycast_moves(
                index,
                &BISHOP_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                color,
                true,
                available_moves,
            );
            Self::raycast_moves(
                index,
                &ROOK_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                color,
                false,
                available_moves,
            );

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
        MoveGenerator::get_possible_moves(&board, piece, &mut moves);
        let mut uci: Vec<String> = moves.iter().map(|m| m.to_uci()).collect();
        uci.sort();
        uci
    }

    fn has_move(moves: &[String], m: &str) -> bool {
        moves.contains(&m.to_string())
    }

    mod bishop {
        use super::*;

        mod center_moves {
            use super::*;

            #[test]
            fn test_white_bishop_center_all_moves() {
                let moves = get_moves("8/8/8/8/3B4/8/8/8 w - - 0 1", Piece::WhiteBishop);
                assert!(has_move(&moves, "d4a1"));
                assert!(has_move(&moves, "d4b2"));
                assert!(has_move(&moves, "d4c3"));
                assert!(has_move(&moves, "d4e5"));
                assert!(has_move(&moves, "d4f6"));
                assert!(has_move(&moves, "d4g7"));
                assert!(has_move(&moves, "d4h8"));
                assert!(has_move(&moves, "d4a7"));
                assert!(has_move(&moves, "d4b6"));
                assert!(has_move(&moves, "d4c5"));
                assert!(has_move(&moves, "d4e3"));
                assert!(has_move(&moves, "d4f2"));
                assert!(has_move(&moves, "d4g1"));
                assert_eq!(moves.iter().filter(|m| m.starts_with("d4")).count(), 13);
            }
        }

        mod captures {
            use super::*;

            #[test]
            fn test_white_bishop_capture_single_target() {
                let moves = get_moves("8/8/8/4p3/3B4/8/8/8 w - - 0 1", Piece::WhiteBishop);
                assert!(has_move(&moves, "d4e5"));
                assert!(!has_move(&moves, "d4f6"));
            }

            #[test]
            fn test_black_bishop_capture_single_target() {
                let moves = get_moves("8/8/8/4P3/3b4/8/8/8 b - - 0 1", Piece::BlackBishop);
                assert!(has_move(&moves, "d4e5"));
                assert!(!has_move(&moves, "d4f6"));
            }

            #[test]
            fn test_white_bishop_capture_both_diagonals() {
                let moves = get_moves("8/8/8/4p3/3B4/1p6/8/8 w - - 0 1", Piece::WhiteBishop);
                assert!(has_move(&moves, "d4e5"));
                assert!(has_move(&moves, "d4b2"));
            }
        }

        mod blocked {
            use super::*;

            #[test]
            fn test_white_bishop_blocked_by_own_piece() {
                let moves = get_moves("8/8/8/4p3/3B4/2P5/8/8 w - - 0 1", Piece::WhiteBishop);
                assert!(has_move(&moves, "d4e5"));
                assert!(!has_move(&moves, "d4f6"));
                assert!(!has_move(&moves, "d4c3"));
            }
        }

        mod edge_files {
            use super::*;

            #[test]
            fn test_white_bishop_a1_no_wrap() {
                let moves = get_moves("8/8/8/8/8/8/8/B7 w - - 0 1", Piece::WhiteBishop);
                assert!(has_move(&moves, "a1b2"));
                assert!(has_move(&moves, "a1c3"));
                assert!(has_move(&moves, "a1d4"));
                assert!(!has_move(&moves, "a1g2"));
            }

            #[test]
            fn test_black_bishop_h8_no_wrap() {
                let moves = get_moves("7b/8/8/8/8/8/8/8 b - - 0 1", Piece::BlackBishop);
                assert!(has_move(&moves, "h8g7"));
                assert!(has_move(&moves, "h8f6"));
                assert!(has_move(&moves, "h8e5"));
                assert!(!has_move(&moves, "h8a7"));
            }
        }
    }

    mod rook {
        use super::*;

        mod center_moves {
            use super::*;

            #[test]
            fn test_white_rook_center_all_moves() {
                let moves = get_moves("8/8/8/8/3R4/8/8/8 w - - 0 1", Piece::WhiteRook);
                assert!(has_move(&moves, "d4d1"));
                assert!(has_move(&moves, "d4d2"));
                assert!(has_move(&moves, "d4d3"));
                assert!(has_move(&moves, "d4d5"));
                assert!(has_move(&moves, "d4d6"));
                assert!(has_move(&moves, "d4d7"));
                assert!(has_move(&moves, "d4d8"));
                assert!(has_move(&moves, "d4a4"));
                assert!(has_move(&moves, "d4b4"));
                assert!(has_move(&moves, "d4c4"));
                assert!(has_move(&moves, "d4e4"));
                assert!(has_move(&moves, "d4f4"));
                assert!(has_move(&moves, "d4g4"));
                assert!(has_move(&moves, "d4h4"));
                assert_eq!(moves.iter().filter(|m| m.starts_with("d4")).count(), 14);
            }
        }

        mod captures {
            use super::*;

            #[test]
            fn test_white_rook_capture_single_target() {
                let moves = get_moves("8/8/8/8/3R4/8/3p4/8 w - - 0 1", Piece::WhiteRook);
                assert!(has_move(&moves, "d4d2"));
                assert!(!has_move(&moves, "d4d1"));
            }

            #[test]
            fn test_black_rook_capture_single_target() {
                let moves = get_moves("8/8/3P4/8/3r4/8/8/8 b - - 0 1", Piece::BlackRook);
                assert!(has_move(&moves, "d4d6"));
                assert!(!has_move(&moves, "d4d7"));
            }
        }

        mod blocked {
            use super::*;

            #[test]
            fn test_white_rook_blocked_by_own_piece() {
                let moves = get_moves("8/8/3P4/8/3R4/8/8/8 w - - 0 1", Piece::WhiteRook);
                assert!(has_move(&moves, "d4d5"));
                assert!(!has_move(&moves, "d4d6"));
            }

            #[test]
            fn test_black_rook_blocked_by_own_piece() {
                let moves = get_moves("8/8/8/8/3r4/8/3p4/8 b - - 0 1", Piece::BlackRook);
                assert!(has_move(&moves, "d4d3"));
                assert!(!has_move(&moves, "d4d2"));
            }
        }

        mod edge_files {
            use super::*;

            #[test]
            fn test_white_rook_a1_no_wrap() {
                let moves = get_moves("8/8/8/8/8/8/8/R7 w - - 0 1", Piece::WhiteRook);
                assert!(has_move(&moves, "a1a8"));
                assert!(has_move(&moves, "a1h1"));
                assert!(!has_move(&moves, "a1b2"));
            }

            #[test]
            fn test_black_rook_h8_no_wrap() {
                let moves = get_moves("7r/8/8/8/8/8/8/8 b - - 0 1", Piece::BlackRook);
                assert!(has_move(&moves, "h8h1"));
                assert!(has_move(&moves, "h8a8"));
                assert!(!has_move(&moves, "h8g7"));
            }
        }
    }

    mod queen {
        use super::*;

        mod center_moves {
            use super::*;

            #[test]
            fn test_white_queen_center_all_moves() {
                let moves = get_moves("8/8/8/8/3Q4/8/8/8 w - - 0 1", Piece::WhiteQueen);
                assert!(has_move(&moves, "d4d1"));
                assert!(has_move(&moves, "d4a4"));
                assert!(has_move(&moves, "d4a1"));
                assert!(has_move(&moves, "d4h4"));
                assert!(has_move(&moves, "d4h8"));
                assert_eq!(moves.iter().filter(|m| m.starts_with("d4")).count(), 27);
            }
        }

        mod captures {
            use super::*;

            #[test]
            fn test_white_queen_capture_bishop_line() {
                let moves = get_moves("8/8/8/4p3/3Q4/8/8/8 w - - 0 1", Piece::WhiteQueen);
                assert!(has_move(&moves, "d4e5"));
                assert!(!has_move(&moves, "d4f6"));
            }

            #[test]
            fn test_black_queen_capture_rook_line() {
                let moves = get_moves("8/8/3P4/8/3q4/8/8/8 b - - 0 1", Piece::BlackQueen);
                assert!(has_move(&moves, "d4d6"));
                assert!(!has_move(&moves, "d4d7"));
            }
        }

        mod blocked {
            use super::*;

            #[test]
            fn test_white_queen_blocked_by_own_piece() {
                let moves = get_moves("8/8/8/4p3/3Q4/3P4/8/8 w - - 0 1", Piece::WhiteQueen);
                assert!(has_move(&moves, "d4e5"));
                assert!(!has_move(&moves, "d4f6"));
                assert!(!has_move(&moves, "d4d3"));
            }
        }

        mod edge_files {
            use super::*;

            #[test]
            fn test_white_queen_a1_no_wrap() {
                let moves = get_moves("8/8/8/8/8/8/8/Q7 w - - 0 1", Piece::WhiteQueen);
                assert!(has_move(&moves, "a1a8"));
                assert!(has_move(&moves, "a1h1"));
                assert!(has_move(&moves, "a1h8"));
                assert!(!has_move(&moves, "a1h2"));
            }
        }
    }
}
