pub(crate) mod king;
pub(crate) mod knight;
pub(crate) mod pawn;
pub(crate) mod sliders;

use crate::board::{Board, Color, Piece};

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
    /// Generates all pseudo-legal moves for a side.
    pub fn get_all_moves(board: &Board, color: Color, available_moves: &mut Vec<Move>) {
        for piece in Piece::all() {
            if piece.color() == color && board.pieces[piece as usize] != 0 {
                Self::get_possible_moves(board, piece, available_moves);
            }
        }
    }

    /// Generates all pseudo-legal moves for a single piece type.
    pub fn get_possible_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        match piece {
            Piece::WhitePawn | Piece::BlackPawn => {
                Self::get_all_pawn_moves(board, piece, available_moves)
            }
            Piece::WhiteKnight | Piece::BlackKnight => {
                Self::get_all_knight_moves(board, piece, available_moves)
            }
            Piece::WhiteBishop | Piece::BlackBishop => {
                Self::get_all_bishop_moves(board, piece, available_moves)
            }
            Piece::WhiteRook | Piece::BlackRook => {
                Self::get_all_rook_moves(board, piece, available_moves)
            }
            Piece::WhiteQueen | Piece::BlackQueen => {
                Self::get_all_queen_moves(board, piece, available_moves)
            }
            Piece::WhiteKing | Piece::BlackKing => {
                Self::get_all_king_moves(board, piece, available_moves)
            }
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
