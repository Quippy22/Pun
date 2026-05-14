use crate::board::Board;

use std::fmt;

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

        // Helper to create the grid with a label
        let to_labeled_map = |n: u64, label: &str| {
            let mut map = format!("\n--- {} ---\n", label);
            for rank in (0..8).rev() {
                for file in 0..8 {
                    let square = rank * 8 + file;
                    let bit = (n >> square) & 1;
                    map.push_str(if bit == 1 { "1 " } else { "0 " });
                }
                map.push('\n');
            }
            map
        };

        // Combine all pieces into one large formatted block
        let mut pieces_output = String::from("");
        for i in 0..12 {
            pieces_output.push_str(&to_labeled_map(self.pieces[i], piece_labels[i]));
        }

        // Combine colors
        let mut colors_output = String::from("");
        colors_output.push_str(&to_labeled_map(self.colors[0], "White Mask"));
        colors_output.push_str(&to_labeled_map(self.colors[1], "Black Mask"));

        f.debug_struct("Board")
            .field("pieces", &format_args!("{}", pieces_output))
            .field("colors", &format_args!("{}", colors_output))
            .field("side_to_move", &self.side_to_move)
            .field("castling", &self.castling_rights)
            .field("en_passant", &self.en_passant_sq)
            .finish()
    }
}
