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
#[derive(Debug, Clone)]
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
    const WHITE_KINGSIDE: u8 = 0b0001;
    const WHITE_QUEENSIDE: u8 = 0b0010;
    const BLACK_KINGSIDE: u8 = 0b0100;
    const BLACK_QUEENSIDE: u8 = 0b1000;

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

    fn refresh_colors(&mut self) {
        self.colors[0] = self.get_side_bitboard(Color::White);
        self.colors[1] = self.get_side_bitboard(Color::Black);
    }

    fn piece_at(&self, square: u8) -> Option<Piece> {
        let mask = 1u64 << square;
        Piece::all().find(|piece| self.pieces[*piece as usize] & mask != 0)
    }

    fn remove_piece_at(&mut self, square: u8) -> Option<Piece> {
        let mask = 1u64 << square;
        for piece in Piece::all() {
            let bb = &mut self.pieces[piece as usize];
            if (*bb & mask) != 0 {
                *bb &= !mask;
                return Some(piece);
            }
        }
        None
    }

    fn place_piece_at(&mut self, piece: Piece, square: u8) {
        self.pieces[piece as usize] |= 1u64 << square;
    }

    fn clear_castling_rights_for_rook_square(&mut self, square: u8) {
        self.castling_rights &= match square {
            0 => !Self::WHITE_QUEENSIDE,
            7 => !Self::WHITE_KINGSIDE,
            56 => !Self::BLACK_QUEENSIDE,
            63 => !Self::BLACK_KINGSIDE,
            _ => 0b1111,
        };
    }

    pub fn update_state(&mut self, uci_move: &str) {
        let from_str = &uci_move[0..2];
        let to_str = &uci_move[2..4];
        let promotion = uci_move.as_bytes().get(4).copied().map(char::from);

        let from_sq = string_to_square(from_str);
        let to_sq = string_to_square(to_str);
        let moving_color = self.side_to_move;
        let moving_piece = self
            .piece_at(from_sq)
            .unwrap_or_else(|| panic!("No piece found on {}", from_str));
        let target_piece = self.piece_at(to_sq);
        let prev_en_passant_sq = self.en_passant_sq;

        let is_pawn = matches!(moving_piece, Piece::WhitePawn | Piece::BlackPawn);

        // Clear the previous en passant square unless a new double pawn push creates one.
        self.en_passant_sq = None;

        // Handle en passant captures before moving the pawn.
        let mut captured_piece = target_piece;
        let mut captured_sq = if target_piece.is_some() { Some(to_sq) } else { None };

        if is_pawn
            && target_piece.is_none()
            && from_sq % 8 != to_sq % 8
            && prev_en_passant_sq == Some(to_sq)
        {
            let ep_capture_sq = match moving_color {
                Color::White => to_sq
                    .checked_sub(8)
                    .expect("white en passant capture square underflow"),
                Color::Black => to_sq
                    .checked_add(8)
                    .expect("black en passant capture square overflow"),
            };
            captured_piece = self.remove_piece_at(ep_capture_sq);
            captured_sq = Some(ep_capture_sq);
        }

        // Remove the moving piece from its origin square.
        self.remove_piece_at(from_sq);

        // If the move captured a rook on its home square, update castling rights.
        if let Some(Piece::WhiteRook | Piece::BlackRook) = captured_piece {
            if let Some(square) = captured_sq {
                self.clear_castling_rights_for_rook_square(square);
            }
        }

        // Update castling rights for a king or rook move.
        match moving_piece {
            Piece::WhiteKing => self.castling_rights &= !(Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE),
            Piece::BlackKing => self.castling_rights &= !(Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE),
            Piece::WhiteRook | Piece::BlackRook => self.clear_castling_rights_for_rook_square(from_sq),
            _ => {}
        }

        // Handle castling rook movement.
        match (moving_piece, from_sq, to_sq) {
            (Piece::WhiteKing, 4, 6) => {
                self.remove_piece_at(7);
                self.place_piece_at(Piece::WhiteRook, 5);
            }
            (Piece::WhiteKing, 4, 2) => {
                self.remove_piece_at(0);
                self.place_piece_at(Piece::WhiteRook, 3);
            }
            (Piece::BlackKing, 60, 62) => {
                self.remove_piece_at(63);
                self.place_piece_at(Piece::BlackRook, 61);
            }
            (Piece::BlackKing, 60, 58) => {
                self.remove_piece_at(56);
                self.place_piece_at(Piece::BlackRook, 59);
            }
            _ => {}
        }

        // Handle promotions or normal piece placement.
        let piece_to_place = if is_pawn {
            match promotion {
                Some('q') => match moving_color {
                    Color::White => Piece::WhiteQueen,
                    Color::Black => Piece::BlackQueen,
                },
                Some('r') => match moving_color {
                    Color::White => Piece::WhiteRook,
                    Color::Black => Piece::BlackRook,
                },
                Some('b') => match moving_color {
                    Color::White => Piece::WhiteBishop,
                    Color::Black => Piece::BlackBishop,
                },
                Some('n') => match moving_color {
                    Color::White => Piece::WhiteKnight,
                    Color::Black => Piece::BlackKnight,
                },
                Some(other) => panic!("Invalid promotion piece: {}", other),
                None => moving_piece,
            }
        } else {
            moving_piece
        };
        self.place_piece_at(piece_to_place, to_sq);

        // Set en passant target after a double pawn push.
        if is_pawn {
            match moving_color {
                Color::White if from_sq + 16 == to_sq => {
                    self.en_passant_sq = Some(from_sq + 8);
                }
                Color::Black if from_sq == to_sq + 16 => {
                    self.en_passant_sq = Some(from_sq - 8);
                }
                _ => {}
            }
        }

        // Update clocks.
        if is_pawn || captured_piece.is_some() {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock = self
                .half_move_clock
                .checked_add(1)
                .expect("halfmove clock overflow");
        }

        if matches!(moving_color, Color::Black) {
            self.full_move_clock = self
                .full_move_clock
                .checked_add(1)
                .expect("fullmove clock overflow");
        }

        self.refresh_colors();

        // flip the side to move
        self.side_to_move = match moving_color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }
}
