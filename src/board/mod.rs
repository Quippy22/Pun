mod display;
mod fen;
pub mod moves;

use crate::board::fen::FenData;
use crate::utils::string_to_square;

/// Used to acces the bitboards by color
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Black,
}
/// Used to acces the bitboards by pice type
#[derive(Clone, Copy, Debug)]
pub enum Piece {
    WhitePawn = 0,
    WhiteKnight = 1,
    WhiteBishop = 2,
    WhiteRook = 3,
    WhiteQueen = 4,
    WhiteKing = 5,
    BlackPawn = 6,
    BlackKnight = 7,
    BlackBishop = 8,
    BlackRook = 9,
    BlackQueen = 10,
    BlackKing = 11,
}

impl Piece {
    pub fn all() -> impl Iterator<Item = Self> {
        [
            Self::WhitePawn,
            Self::WhiteKnight,
            Self::WhiteBishop,
            Self::WhiteRook,
            Self::WhiteQueen,
            Self::WhiteKing,
            Self::BlackPawn,
            Self::BlackKnight,
            Self::BlackBishop,
            Self::BlackRook,
            Self::BlackQueen,
            Self::BlackKing,
        ]
        .into_iter()
    }

    pub fn color(&self) -> Color {
        if *self as usize <= 5 {
            Color::White
        } else {
            Color::Black
        }
    }
}

/// The board struct
#[derive(Debug)]
pub struct Board {
    /// The bitboards
    pub pieces: [u64; 12],
    /// The bitboards for the colors
    pub colors: [u64; 2],
    pub side_to_move: Color,
    /// The castling rights
    /// Bit 0 - White king can castle to the king side
    /// Bit 1 - White king can castle to the queen side
    /// Bit 2 - Black king can castle to the king side
    /// Bit 3 - Black king can castle to the queen side
    pub castling_rights: u8,
    /// The en passant square
    pub en_passant_sq: Option<u8>,
    /// The half move clock
    pub half_move_clock: u16,
    /// The full move clock
    pub full_move_clock: u16,
}

impl Board {
    pub fn initialize_from_fen(fen: &str) -> Self {
        let fen_data = FenData::parse(fen);
        let white_bitboard = Self::get_color_bitboard(&fen_data.pieces, Color::White);
        let black_bitboard = Self::get_color_bitboard(&fen_data.pieces, Color::Black);

        Self {
            pieces: fen_data.pieces,
            colors: [white_bitboard, black_bitboard],
            side_to_move: fen_data.side_to_move,
            castling_rights: fen_data.castling_rights,
            en_passant_sq: fen_data.en_passant_sq,
            half_move_clock: fen_data.half_move,
            full_move_clock: fen_data.full_move,
        }
    }

    pub fn get_color_bitboard(pieces: &[u64; 12], color: Color) -> u64 {
        let mut bitboard = 0u64;
        let range = match color {
            Color::White => 0..6,
            Color::Black => 6..12,
        };

        for i in range {
            bitboard |= pieces[i];
        }

        bitboard
    }

    pub fn get_side_bitboard(&self, color: Color) -> u64 {
        let mut bitboard = 0u64;
        let range = match color {
            Color::White => 0..6,
            Color::Black => 6..12,
        };
        for i in range {
            bitboard |= self.pieces[i];
        }

        bitboard
    }

    pub fn update_state(&mut self, uci_move: &str) {
        let from_str = &uci_move[0..2];
        let to_str = &uci_move[2..4];

        let from_sq = string_to_square(from_str);
        let to_sq = string_to_square(to_str);

        let from_mask: u64 = 1 << from_sq;
        let to_mask: u64 = 1 << to_sq;

        for bb in self.pieces.iter_mut() {
            if (*bb & to_mask) != 0 {
                *bb &= !to_mask;
                break;
            }
        }
        for bb in self.pieces.iter_mut() {
            if (*bb & from_mask) != 0 {
                *bb &= !from_mask;
                *bb |= to_mask;
                break;
            }
        }

        // update the self.colors
        self.colors[0] = self.get_side_bitboard(Color::White);
        self.colors[1] = self.get_side_bitboard(Color::Black);

        // flip the side to move
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }
}
