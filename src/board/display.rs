use crate::board::Board;
use crate::board::moves::Move;

use std::fmt;

impl Board {
    /// Prints every bitboard with its piece label for data verification
    pub fn print_data(&self) {
        let piece_labels = [
            "WhitePawn",
            "WhiteKnight",
            "WhiteBishop",
            "WhiteRook",
            "WhiteQueen",
            "WhiteKing",
            "BlackPawn",
            "BlackKnight",
            "BlackBishop",
            "BlackRook",
            "BlackQueen",
            "BlackKing",
        ];

        println!("Board Data:");
        println!("=================");

        for (i, piece) in piece_labels.iter().enumerate() {
            println!("\n--- {} (Index {}) ---", piece, i);
            self.print_bitboard_map(self.pieces[i]);
        }

        println!("\n--- Occupancy Masks ---");
        println!("White Mask:");
        self.print_bitboard_map(self.colors[0]);
        println!("Black Mask:");
        self.print_bitboard_map(self.colors[1]);

        println!("\nMetadata:");
        println!("Side to move:    {:?}", self.side_to_move);
        println!("Castling Rights: {:b}", self.castling_rights);
        println!("En Passant:      {:?}", self.en_passant_sq);
    }

    // Private helper to keep the bitboard printing logic in one place
    fn print_bitboard_map(&self, bb: u64) {
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let square = rank * 8 + file;
                let bit = (bb >> square) & 1;
                print!("{} ", if bit == 1 { "1" } else { "0" });
            }
            println!();
        }
        println!("  a b c d e f g h");
    }
}

// Keep your standard Display impl here too
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  +-----------------+")?;
        for rank in (0..8).rev() {
            write!(f, "{} | ", rank + 1)?;
            for file in 0..8 {
                let square = rank * 8 + file;
                let mut piece_char = '.';
                for (i, bb) in self.pieces.iter().enumerate() {
                    if (bb >> square) & 1 == 1 {
                        piece_char = match i {
                            0 => 'P',
                            1 => 'N',
                            2 => 'B',
                            3 => 'R',
                            4 => 'Q',
                            5 => 'K',
                            6 => 'p',
                            7 => 'n',
                            8 => 'b',
                            9 => 'r',
                            10 => 'q',
                            11 => 'k',
                            _ => '.',
                        };
                        break;
                    }
                }
                write!(f, "{} ", piece_char)?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "  +-----------------+")?;
        writeln!(f, "    a b c d e f g h")?;
        Ok(())
    }
}

/// Formats a chess move into an explicit, human-readable debugging format.
///
/// Evaluates the unique move types using internal flag patterns:
/// - Castles: `O-O` (Kingside `0b0100`) or `O-O-O` (Queenside `0b0110`).
/// - En Passant: Appends an `x` and ` e.p.` suffix (e.g., `e5xd6 e.p.`).
/// - Captures: Injects an `x` between squares (e.g., `e2xe4`).
/// - Promotions: Appends the specific piece character (`q`, `r`, `b`, `n`).
///
/// # Explicit Flag Map:
/// - `0b0000` -> Normal Move
/// - `0b0001` -> Capture
/// - `0b0011` -> En Passant
/// - `0b0100` -> Kingside Castle
/// - `0b0110` -> Queenside Castle
/// - `0b1000` -> Queen Promotion
/// - `0b1010` -> Rook Promotion
/// - `0b1100` -> Bishop Promotion
/// - `0b1110` -> Knight Promotion
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1. Evaluate Castles first using your exact flags
        match self.special_flag() {
            0b0100 => return write!(f, "O-O"),
            0b0110 => return write!(f, "O-O-O"),
            _ => {} // Not a castle, proceed to standard coordinates
        }

        let start = self.start_pos();
        let end = self.end_pos();

        let start_file = (b'a' + (start % 8) as u8) as char;
        let start_rank = (b'1' + (start / 8) as u8) as char;
        let end_file = (b'a' + (end % 8) as u8) as char;
        let end_rank = (b'1' + (end / 8) as u8) as char;

        // 2. Check for captures (Normal Capture OR En Passant)
        let is_hit = self.is_capture() || self.is_en_passant();
        let separator = if is_hit { "x" } else { "" };

        // Print core path (e.g., "e2e4" or "e2xe4")
        write!(
            f,
            "{}{}{}{}{}",
            start_file, start_rank, separator, end_file, end_rank
        )?;

        // 3. Handle En Passant markers
        if self.is_en_passant() {
            write!(f, " e.p.")?;
        }

        // 4. Handle all explicit promotion options cleanly
        if self.is_promotion() {
            match self.special_flag() & !0b0001 {
                0b1000 => write!(f, "q")?,
                0b1010 => write!(f, "r")?,
                0b1100 => write!(f, "b")?,
                0b1110 => write!(f, "n")?,
                _ => {}
            }
        }

        Ok(())
    }
}
