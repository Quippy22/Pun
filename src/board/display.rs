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

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_file = (b'a' + (self.start_pos() % 8) as u8) as char;
        let start_rank = (b'1' + (self.start_pos() / 8) as u8) as char;
        let end_file = (b'a' + (self.end_pos() % 8) as u8) as char;
        let end_rank = (b'1' + (self.end_pos() / 8) as u8) as char;

        write!(f, "{}{}{}{}", start_file, start_rank, end_file, end_rank)
    }
}
