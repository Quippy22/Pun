mod display;
mod fen;
pub mod moves;

use crate::board::fen::FenData;
use crate::board::moves::Move;

/// Side-to-move and color indexing for board bitboards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    /// White side.
    White,
    /// Black side.
    Black,
}

impl Color {
    /// Returns the opposite color.
    pub fn opposite(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// The type of piece, independent of color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn all() -> impl Iterator<Item = Self> {
        [
            Self::Pawn,
            Self::Knight,
            Self::Bishop,
            Self::Rook,
            Self::Queen,
            Self::King,
        ]
        .into_iter()
    }

    pub fn value(&self) -> i32 {
        match self {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000,
        }
    }
}

/// Piece identifiers and their backing bitboard indices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Piece {
    /// White pawn bitboard index.
    WhitePawn = 0,
    /// White knight bitboard index.
    WhiteKnight = 1,
    /// White bishop bitboard index.
    WhiteBishop = 2,
    /// White rook bitboard index.
    WhiteRook = 3,
    /// White queen bitboard index.
    WhiteQueen = 4,
    /// White king bitboard index.
    WhiteKing = 5,
    /// Black pawn bitboard index.
    BlackPawn = 6,
    /// Black knight bitboard index.
    BlackKnight = 7,
    /// Black bishop bitboard index.
    BlackBishop = 8,
    /// Black rook bitboard index.
    BlackRook = 9,
    /// Black queen bitboard index.
    BlackQueen = 10,
    /// Black king bitboard index.
    BlackKing = 11,
}

impl Piece {
    /// Constructs a `Piece` from a color and piece type.
    pub fn new(color: Color, kind: PieceType) -> Self {
        match (color, kind) {
            (Color::White, PieceType::Pawn) => Piece::WhitePawn,
            (Color::White, PieceType::Knight) => Piece::WhiteKnight,
            (Color::White, PieceType::Bishop) => Piece::WhiteBishop,
            (Color::White, PieceType::Rook) => Piece::WhiteRook,
            (Color::White, PieceType::Queen) => Piece::WhiteQueen,
            (Color::White, PieceType::King) => Piece::WhiteKing,
            (Color::Black, PieceType::Pawn) => Piece::BlackPawn,
            (Color::Black, PieceType::Knight) => Piece::BlackKnight,
            (Color::Black, PieceType::Bishop) => Piece::BlackBishop,
            (Color::Black, PieceType::Rook) => Piece::BlackRook,
            (Color::Black, PieceType::Queen) => Piece::BlackQueen,
            (Color::Black, PieceType::King) => Piece::BlackKing,
        }
    }

    /// Returns every piece enum value in board storage order.
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

    /// Returns the side that owns this piece.
    pub fn color(&self) -> Color {
        if *self as usize <= 5 {
            Color::White
        } else {
            Color::Black
        }
    }

    /// Returns the piece of the opposite color with the same kind.
    pub fn opposite(&self) -> Self {
        // achives that by creating a new piece with the opposite color and the same kind
        Piece::new(self.color().opposite(), self.kind())
    }

    /// Returns the type of piece regardless of color
    pub fn kind(&self) -> PieceType {
        match self {
            Piece::WhitePawn | Piece::BlackPawn => PieceType::Pawn,
            Piece::WhiteKnight | Piece::BlackKnight => PieceType::Knight,
            Piece::WhiteBishop | Piece::BlackBishop => PieceType::Bishop,
            Piece::WhiteRook | Piece::BlackRook => PieceType::Rook,
            Piece::WhiteQueen | Piece::BlackQueen => PieceType::Queen,
            Piece::WhiteKing | Piece::BlackKing => PieceType::King,
        }
    }
}

/// Complete board state tracked by the engine.
#[derive(Debug, Clone)]
pub struct Board {
    /// One bitboard per piece type.
    pub pieces: [u64; 12],
    /// One occupancy bitboard per color.
    pub colors: [u64; 2],
    /// Side to move.
    pub side_to_move: Color,
    /// Castling rights bit mask.
    ///
    /// Bit 0: white king side
    /// Bit 1: white queen side
    /// Bit 2: black king side
    /// Bit 3: black queen side
    pub castling_rights: u8,
    /// En passant target square, if any.
    pub en_passant_sq: Option<u8>,
    /// Half-move clock for the 50-move rule.
    pub half_move_clock: u16,
    /// Full move number.
    pub full_move_clock: u16,
}

impl Board {
    /// Castling-right mask for white king side.
    const WHITE_KINGSIDE: u8 = 0b0001;
    /// Castling-right mask for white queen side.
    const WHITE_QUEENSIDE: u8 = 0b0010;
    /// Castling-right mask for black king side.
    const BLACK_KINGSIDE: u8 = 0b0100;
    /// Castling-right mask for black queen side.
    const BLACK_QUEENSIDE: u8 = 0b1000;

    /// Builds a board state from a FEN string.
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

    /// ORs together all piece bitboards for a given color.
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

    /// Returns the current occupancy mask for one side.
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

    /// Applies a UCI move string to the board state.
    ///
    /// This handles:
    /// - captures
    /// - pawn promotions
    /// - en passant
    /// - castling rook movement
    /// - castling-right updates
    /// - en passant target updates
    /// - halfmove/fullmove clocks
    pub fn make_move(&mut self, mv: &Move) {
        // 1. Extract move data
        let from_sq = mv.start_pos() as u8;
        let to_sq = mv.end_pos() as u8;

        // 1.1 Get the moving piece and the target piece
        let moving_color = self.side_to_move;
        let moving_piece = self
            .piece_at(from_sq)
            .unwrap_or_else(|| panic!("No piece found on square {}", from_sq));
        let target_piece = self.piece_at(to_sq);

        // 2. Treat en passant specially
        let prev_en_passant_sq = self.en_passant_sq;

        let is_pawn = moving_piece.kind() == PieceType::Pawn;

        // En passant only lasts for the immediately following move.
        self.en_passant_sq = None;

        // Standard captures remove the target square. En passant is special
        // because the captured pawn sits behind the destination square.
        let mut captured_piece = target_piece;
        let mut captured_sq = if target_piece.is_some() {
            Some(to_sq)
        } else {
            None
        };

        // Remove the captured piece
        if target_piece.is_some() {
            self.remove_piece_at(to_sq);
        }

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

        // Remove the moving piece before any special re-placement.
        self.remove_piece_at(from_sq);

        // A double pawn push exposes the skipped square as a new en passant target.
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

        // 3. Treat castling rights
        // Capturing a rook on its home square removes the corresponding right.
        if captured_piece.is_some_and(|p| p.kind() == PieceType::Rook)
            && let Some(square) = captured_sq
        {
            self.clear_castling_rights_for_rook_square(square);
        }

        // Moving a king or rook also removes castling rights.
        match moving_piece.kind() {
            PieceType::King => {
                if moving_color == Color::White {
                    self.castling_rights &= !(Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE);
                } else {
                    self.castling_rights &= !(Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE);
                }
            }
            PieceType::Rook => {
                self.clear_castling_rights_for_rook_square(from_sq);
            }
            _ => {}
        }

        // Castling is encoded as a king move, but the rook must move too.
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

        // 4. Treat promotions
        // Promotions replace the pawn with the promoted piece.
        let piece_to_place = if is_pawn && mv.is_promotion() {
            let kind = match mv.special_flag() & !0b0001 {
                0b1000 => PieceType::Queen,
                0b1010 => PieceType::Rook,
                0b1100 => PieceType::Bishop,
                0b1110 => PieceType::Knight,
                _ => panic!("Invalid promotion flag: {:b}", mv.special_flag()),
            };
            Piece::new(moving_color, kind)
        } else {
            moving_piece
        };
        self.place_piece_at(piece_to_place, to_sq);

        // 5. Refresh the board state
        // Pawn moves and captures reset the halfmove clock.
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

        // Refresh cached occupancy and swap the side to move.
        self.refresh_colors();

        self.side_to_move = moving_color.opposite();
    }

    // -- HELPERS --

    /// Refreshes the cached occupancy masks after a board mutation.
    pub fn refresh_colors(&mut self) {
        self.colors[0] = self.get_side_bitboard(Color::White);
        self.colors[1] = self.get_side_bitboard(Color::Black);
    }

    /// Returns the piece sitting on a square, if any.
    fn piece_at(&self, square: u8) -> Option<Piece> {
        let mask = 1u64 << square;
        Piece::all().find(|piece| self.pieces[*piece as usize] & mask != 0)
    }

    /// Removes and returns the piece on a square.
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

    /// Places a piece on a square without clearing anything first.
    fn place_piece_at(&mut self, piece: Piece, square: u8) {
        self.pieces[piece as usize] |= 1u64 << square;
    }

    /// Clears castling rights when a rook leaves or is captured on a home square.
    fn clear_castling_rights_for_rook_square(&mut self, square: u8) {
        self.castling_rights &= match square {
            0 => !Self::WHITE_QUEENSIDE,
            7 => !Self::WHITE_KINGSIDE,
            56 => !Self::BLACK_QUEENSIDE,
            63 => !Self::BLACK_KINGSIDE,
            _ => 0b1111,
        };
    }
}
