mod attacks;
pub(crate) mod king;
pub(crate) mod knight;
pub(crate) mod openings;
pub(crate) mod pawn;
pub(crate) mod sliders;

use crate::board::{Board, Color, Piece, PieceType};
use crate::utils::string_to_square;

/// Compact move encoding used throughout the engine.
///
/// The value stores:
/// - from-square
/// - to-square
/// - special move flags
///
/// Slots:   15 14 13 12 | 11 10  9  8  7  6 |  5  4  3  2  1  0
///        +-------------+-------------------+-----------------+
/// Roles: |   4 Flags   |    6 To-Square    |  6 From-Square  |
///        +-------------+-------------------+-----------------+
///
/// Flags:
/// 0000 - normal move
/// 0001 - capture
/// 0011 - en passant
/// 0100 - castle
/// 1000 - promotion
///
/// Promotions:
/// 1000 - queen
/// 1010 - rook
/// 1100 - bishop
/// 1110 - knight
///
/// Castles:
/// 0100 - king side
/// 0110 - queen side
#[derive(Clone, Copy, Debug, Default)]
pub struct Move(pub u16);

impl Move {
    /// Creates a new encoded move.
    pub fn new(start: u16, end: u16, flag: u16) -> Self {
        Self(start | end << 6 | flag << 12)
    }
    /// Returns the source square index.
    pub fn start_pos(&self) -> usize {
        (self.0 & 0b111111) as usize
    }
    /// Returns the destination square index.
    pub fn end_pos(&self) -> usize {
        ((self.0 >> 6) & 0b111111) as usize
    }
    /// Returns the special flag nibble.
    pub fn special_flag(&self) -> u16 {
        self.0 >> 12_u16
    }
    /// Returns `true` if this move captures a piece.
    pub fn is_capture(&self) -> bool {
        (self.special_flag() & 0b0001) == 1
    }
    /// Returns `true` if this move is a promotion.
    pub fn is_promotion(&self) -> bool {
        (self.special_flag() & 0b1000) == 0b1000
    }
    /// Returns `true` if this move is an en passant capture.
    pub fn is_en_passant(&self) -> bool {
        self.special_flag() == 0b0011
    }
    /// Returns `true` if this move is a castle.
    pub fn is_castle(&self) -> bool {
        self.special_flag() == 0b0100 || self.special_flag() == 0b0110
    }

    /// Returns the promoted piece letter, ignoring the capture bit.
    fn promotion_piece(&self) -> Option<char> {
        match self.special_flag() & !0b0001 {
            0b1000 => Some('q'),
            0b1010 => Some('r'),
            0b1100 => Some('b'),
            0b1110 => Some('n'),
            _ => None,
        }
    }

    /// Parses a UCI move string (e.g. "e2e4", "e7e8q") into a Move.
    pub fn from_uci(uci: &str) -> Self {
        let from = string_to_square(&uci[0..2]);
        let to = string_to_square(&uci[2..4]);
        let promotion = uci.as_bytes().get(4).copied().map(char::from);

        let mut flag: u16 = 0;

        // Set capture flag if there's a piece on the target square
        // (en passant is handled separately by the board)
        // We can't know here if it's a capture, so we set it later in make_move.
        // For from_uci, we encode promotions into the flag.

        if let Some(p) = promotion {
            let promo_flag = match p {
                'q' => 0b1000,
                'r' => 0b1010,
                'b' => 0b1100,
                'n' => 0b1110,
                _ => panic!("Invalid promotion piece: {}", p),
            };
            flag |= promo_flag;
        }

        Move::new(from as u16, to as u16, flag)
    }

    /// Formats the move as UCI.
    pub fn to_uci(&self) -> String {
        let start = self.start_pos();
        let end = self.end_pos();
        let mut s = format!(
            "{}{}{}{}",
            (b'a' + (start % 8) as u8) as char,
            (b'1' + (start / 8) as u8) as char,
            (b'a' + (end % 8) as u8) as char,
            (b'1' + (end / 8) as u8) as char,
        );

        if let Some(piece) = self.promotion_piece() {
            s.push(piece);
        }

        s
    }
}

/// Stateless move generator entrypoint.
pub struct MoveGenerator;

impl MoveGenerator {
    /// Filters out the illegal moves
    /// Sorts the moves captures -> promotions -> quiets
    pub fn get_all_moves(board: &mut Board, color: Color, available_moves: &mut Vec<Move>) {
        let mut pseudo = Vec::new();
        Self::get_all_pseudo_legal_moves(board, color, &mut pseudo);

        let king = Piece::new(color, PieceType::King);
        let mut capture_moves = Vec::new();
        let mut promotion_moves = Vec::new();
        let mut quiet_moves = Vec::new();

        for mv in &pseudo {
            board.make_move(mv);
            let king_bb = board.pieces[king as usize];
            let legal = !Self::is_check(board, color, king_bb);
            board.unmake_move();
            if legal {
                if mv.is_capture() {
                    capture_moves.push(*mv);
                } else if mv.is_promotion() {
                    promotion_moves.push(*mv);
                } else {
                    quiet_moves.push(*mv);
                }
            }
        }

        available_moves.extend(capture_moves);
        available_moves.extend(promotion_moves);
        available_moves.extend(quiet_moves);
    }

    /// Generates all pseudo-legal moves for a side.
    fn get_all_pseudo_legal_moves(board: &Board, color: Color, available_moves: &mut Vec<Move>) {
        for kind in PieceType::all() {
            let piece = Piece::new(color, kind);
            if board.pieces[piece as usize] != 0 {
                Self::get_possible_moves(board, piece, available_moves);
            }
        }
    }

    /// Generates all pseudo-legal moves for a single piece type.
    fn get_possible_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        match piece.kind() {
            PieceType::Pawn => Self::get_all_pawn_moves(board, piece, available_moves),
            PieceType::Knight => Self::get_all_knight_moves(board, piece, available_moves),
            PieceType::Bishop => Self::get_all_bishop_moves(board, piece, available_moves),
            PieceType::Rook => Self::get_all_rook_moves(board, piece, available_moves),
            PieceType::Queen => Self::get_all_queen_moves(board, piece, available_moves),
            PieceType::King => Self::get_all_king_moves(board, piece, available_moves),
        }
    }

    /// Returns the bitboard and color for a piece type.
    ///
    /// Black bitboards are byte-swapped into the same orientation as white so
    /// the shift logic can stay symmetric.
    fn get_bitboard(board: &Board, piece: Piece) -> (u64, Color) {
        let color = piece.color();
        let bb = board.pieces[piece as usize];
        let bb = if color == Color::White {
            bb
        } else {
            bb.swap_bytes()
        };
        (bb, color)
    }

    /// Returns own and enemy occupancy masks in the working orientation.
    fn get_sides(board: &Board, color: Color) -> (u64, u64) {
        let (own_bitboard, enemy_bitboard) = match color {
            Color::White => (
                board.get_side_bitboard(Color::White),
                board.get_side_bitboard(Color::Black),
            ),
            Color::Black => (
                board.get_side_bitboard(Color::Black).swap_bytes(),
                board.get_side_bitboard(Color::White).swap_bytes(),
            ),
        };

        (own_bitboard, enemy_bitboard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Color};

    fn legal_moves(fen: &str, color: Color) -> Vec<String> {
        let mut board = Board::initialize_from_fen(fen);
        let mut moves = Vec::new();
        MoveGenerator::get_all_moves(&mut board, color, &mut moves);
        let mut uci: Vec<String> = moves.iter().map(|m| m.to_uci()).collect();
        uci.sort();
        uci
    }

    #[test]
    fn get_all_moves_keeps_board_state_intact() {
        let mut board =
            Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let before = board.clone();
        let mut moves = Vec::new();
        MoveGenerator::get_all_moves(&mut board, Color::White, &mut moves);

        assert_eq!(board.pieces, before.pieces);
        assert_eq!(board.colors, before.colors);
        assert_eq!(board.side_to_move, before.side_to_move);
        assert_eq!(board.castling_rights, before.castling_rights);
        assert_eq!(board.en_passant_sq, before.en_passant_sq);
        assert_eq!(board.half_move_clock, before.half_move_clock);
        assert_eq!(board.full_move_clock, before.full_move_clock);
    }

    fn is_check(fen: &str, color: Color) -> bool {
        let board = Board::initialize_from_fen(fen);
        let king = Piece::new(color, PieceType::King);
        MoveGenerator::is_check(&board, color, board.pieces[king as usize])
    }

    mod check_detection {
        use super::*;

        mod not_in_check {
            use super::*;

            #[test]
            fn test_starting_position_white_not_in_check() {
                assert!(!is_check(
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                    Color::White
                ));
            }

            #[test]
            fn test_starting_position_black_not_in_check() {
                assert!(!is_check(
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                    Color::Black
                ));
            }

            #[test]
            fn test_kings_far_apart_not_in_check() {
                assert!(!is_check("k7/8/8/8/8/8/8/4K3 w - - 0 1", Color::White));
                assert!(!is_check("k7/8/8/8/8/8/8/4K3 w - - 0 1", Color::Black));
            }
        }

        mod rook_and_queen {
            use super::*;

            #[test]
            fn test_white_king_in_check_by_rook_on_same_rank() {
                // black rook a1, white king e1, same rank clear path
                assert!(is_check("7k/8/8/8/8/8/8/r3K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_in_check_by_rook_on_same_file() {
                // black rook e8, white king e1, same file clear path
                assert!(is_check("4r2k/8/8/8/8/8/8/4K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_not_in_check_rook_blocked_by_own_piece() {
                // black rook a1, own rook c1 blocks it, king e1, black king h8
                assert!(!is_check("7k/8/8/8/8/8/8/r1R1K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_not_in_check_rook_blocked_by_enemy_piece() {
                // black rook a1, black pawn b1 blocks it, king e1, black king h8
                assert!(!is_check("7k/8/8/8/8/8/8/rp2K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_in_check_by_queen_on_rank() {
                // black queen a1, white king e1
                assert!(is_check("7k/8/8/8/8/8/8/q3K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_in_check_by_queen_on_file() {
                // black queen e8, white king e1
                assert!(is_check("4q2k/8/8/8/8/8/8/4K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_black_king_in_check_by_rook_on_same_file() {
                // white rook e1, black king e8
                assert!(is_check("4k3/8/8/8/8/8/8/4RK2 w - - 0 1", Color::Black));
            }
        }

        mod bishop_and_diagonal {
            use super::*;

            #[test]
            fn test_white_king_in_check_by_bishop_diagonal() {
                // black bishop a6, white king d3, clear diagonal
                assert!(is_check("7k/8/b7/8/8/3K4/8/8 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_not_in_check_bishop_blocked_by_own_piece() {
                // white pawn c4 blocks diagonal from a6 to d3
                assert!(!is_check("7k/8/b7/8/2P5/3K4/8/8 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_in_check_by_queen_on_diagonal() {
                // black queen a6, white king d3
                assert!(is_check("7k/8/q7/8/8/3K4/8/8 w - - 0 1", Color::White));
            }

            #[test]
            fn test_black_king_in_check_by_bishop_diagonal() {
                // white bishop b5, black king e8
                assert!(is_check("4k3/8/8/1B6/8/8/8/4K3 w - - 0 1", Color::Black));
            }
        }

        mod knight {
            use super::*;

            #[test]
            fn test_white_king_in_check_by_knight() {
                // black knight d3 attacks e1
                assert!(is_check("7k/8/8/8/8/3n4/8/4K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_white_king_not_in_check_knight_wrong_square() {
                // black knight c3 does NOT attack e1
                assert!(!is_check("7k/8/8/8/8/2n5/8/4K3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_black_king_in_check_by_knight() {
                // white knight f6 attacks e8
                assert!(is_check("4k3/8/5N2/8/8/8/8/4K3 w - - 0 1", Color::Black));
            }
        }

        mod pawn_wall {
            use super::*;

            #[test]
            fn test_white_king_safe_behind_pawn_wall() {
                // full pawn wall on rank 2, black rook on e8 cannot reach king
                assert!(!is_check(
                    "3rk3/8/8/8/8/8/PPPPPPPP/4K3 w - - 0 1",
                    Color::White
                ));
            }

            #[test]
            fn test_white_king_exposed_after_pawn_gap() {
                // gap on e2, black rook on e8 reaches king on e1
                assert!(is_check(
                    "4rk3/8/8/8/8/8/PPPP1PPP/4K3 w - - 0 1",
                    Color::White
                ));
            }

            #[test]
            fn test_black_king_safe_behind_pawn_wall() {
                assert!(!is_check(
                    "4k3/pppppppp/8/8/8/8/8/4K3 b - - 0 1",
                    Color::Black
                ));
            }
        }

        mod own_pieces_not_confused_with_enemy {
            use super::*;

            #[test]
            fn test_own_rooks_do_not_trigger_check_detection() {
                assert!(!is_check("7k/8/8/8/8/8/8/R3K2R w - - 0 1", Color::White));
            }

            #[test]
            fn test_own_bishops_do_not_trigger_check_detection() {
                assert!(!is_check("7k/8/8/8/8/8/8/2B1KB2 w - - 0 1", Color::White));
            }

            #[test]
            fn test_own_queen_does_not_trigger_check_detection() {
                assert!(!is_check("7k/8/8/8/8/8/8/3QK3 w - - 0 1", Color::White));
            }

            #[test]
            fn test_own_piece_blocks_enemy_rook() {
                // black rook a1, own rook c1 blocks, king d1, black king h8
                assert!(!is_check("7k/8/8/8/8/8/8/r1RK4 w - - 0 1", Color::White));
            }
        }
    }

    mod legal_move_filtering {
        use super::*;

        mod pinned_pieces {
            use super::*;

            #[test]
            fn test_pinned_piece_cannot_move_off_pin_ray() {
                // white rook e4 pinned by black rook e8, king e1
                let moves = legal_moves("4r2k/8/8/8/4R3/8/8/4K3 w - - 0 1", Color::White);
                assert!(
                    !moves
                        .iter()
                        .any(|m| m.starts_with("e4") && !m.starts_with("e4e"))
                );
            }

            #[test]
            fn test_pinned_piece_can_move_along_pin_ray() {
                // white rook e4 pinned by black rook e8, can still move on e-file
                let moves = legal_moves("4r2k/8/8/8/4R3/8/8/4K3 w - - 0 1", Color::White);
                assert!(moves.contains(&"e4e5".to_string()));
                assert!(moves.contains(&"e4e8".to_string()));
            }

            #[test]
            fn test_diagonally_pinned_piece_cannot_move() {
                // white knight d2 pinned by black bishop a5, king e1
                // knight can never move along a pin ray
                let moves = legal_moves("7k/8/8/b7/8/8/3N4/4K3 w - - 0 1", Color::White);
                assert!(!moves.iter().any(|m| m.starts_with("d2")));
            }
        }

        mod must_escape_check {
            use super::*;

            #[test]
            fn test_king_must_move_when_in_check() {
                // black rook a1 gives check, king e1 must escape
                let moves = legal_moves("7k/8/8/8/8/8/8/r3K3 w - - 0 1", Color::White);
                assert!(moves.iter().all(|m| m.starts_with("e1")));
            }

            #[test]
            fn test_block_check_is_legal() {
                // black rook e8 checks king e1, white rook d4 can block on e4
                let moves = legal_moves("4r2k/8/8/8/3R4/8/8/4K3 w - - 0 1", Color::White);
                assert!(moves.contains(&"d4e4".to_string()));
            }

            #[test]
            fn test_capture_checker_is_legal() {
                // black rook e4 checks king e1, white rook a4 can capture
                let moves = legal_moves("7k/8/8/8/R3r3/8/8/4K3 w - - 0 1", Color::White);
                assert!(moves.contains(&"a4e4".to_string()));
            }

            #[test]
            fn test_move_that_walks_into_check_is_illegal() {
                // black rook f8, king e1 — e1f2 walks into check
                let moves = legal_moves("5r1k/8/8/8/8/8/8/4K3 w - - 0 1", Color::White);
                assert!(!moves.contains(&"e1f2".to_string()));
            }

            #[test]
            fn test_double_check_only_king_can_move() {
                // black rook e8 and black bishop b4 give double check
                // only king moves are legal
                let moves = legal_moves("4r2k/8/8/8/1b6/8/8/4K3 w - - 0 1", Color::White);
                assert!(moves.iter().all(|m| m.starts_with("e1")));
            }
        }
    }
}
