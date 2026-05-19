use crate::board::{Board, Color, Piece};

/// Wrapper around u16
/// holds the starting position
/// the ending position
/// and a special flag:
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
    /// returns the current square of the piece
    pub fn start_pos(&self) -> usize {
        (self.0 & 0xFF) as usize
    }
    /// returns the end square of the piece
    pub fn end_pos(&self) -> usize {
        ((self.0 >> 6) & 0b111111) as usize
    }
    /// returns the special flag of the move
    pub fn special_flag(&self) -> u16 {
        self.0 >> 12 as u16
    }
    /// checks if the move is a capture
    pub fn is_capture(&self) -> bool {
        (self.special_flag() & 0b0001) == 1
    }
    /// checks if the move is a promotion
    pub fn is_promotion(&self) -> bool {
        (self.special_flag() & 0b1000) == 0b1000
    }
    /// checks if the move is En Passant
    pub fn is_en_passant(&self) -> bool {
        self.special_flag() == 0b0011
    }
    //checks if the move is a castle
    pub fn is_castle(&self) -> bool {
        self.special_flag() == 0b0100 || self.special_flag() == 0b0110
    }
}

