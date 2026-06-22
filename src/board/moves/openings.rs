use rand::RngExt;

const WHITE_OPENINGS: &[&[&str]] = &[
    // Italian Game: Giuoco Piano / Evans Gambit lines
    &[
        "e2e4", "e7e5", "g1f3", "b1c3", "g8f6", "f1c4", "f8c5", "d2d3",
    ],
    // Ruy Lopez: Mainline Morphy Defense
    &[
        "e2e4", "e7e5", "g1f3", "b1c3", "a7a6", "f1b5", "g8f6", "e1g1",
    ],
    // Queen's Gambit Declined: Classical Mainline
    &[
        "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7", "e2e3",
    ],
    // English Opening: Symmetrical
    &[
        "c2c4", "c7c5", "g1f3", "g8f6", "b1c3", "b8c6", "d2d4", "c5d4", "f3d4",
    ],
    // Reti / Catalan Style
    &[
        "g1f3", "d7d5", "g2g3", "g8f6", "f1g2", "e7e6", "e1g1", "f8e7", "c2c4",
    ],
];

// Black's responses, bucketed by White's actual first move
const BLACK_VS_E4: &[&[&str]] = &[
    // Petroff Defense
    &[
        "e7e5", "g1f3", "g8f6", "f3e5", "d7d6", "e5f3", "f6e4", "d2d4",
    ],
    // Sicilian Najdorf / Classical
    &[
        "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6",
    ],
    // French Defense: Winawer / Classical
    &[
        "e7e6", "d2d4", "d7d5", "b1c3", "g8f6", "c1g5", "f8e7", "e2e5", "f6d7",
    ],
    // Caro-Kann: Advance Variation
    &[
        "c7c6", "d2d4", "d7d5", "e4e5", "c1f5", "g1f3", "e7e6", "f1e2", "c7c5",
    ],
];

const BLACK_VS_D4: &[&[&str]] = &[
    // King's Indian Defense
    &[
        "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6", "g1f3", "e1g1",
    ],
    // Nimzo-Indian Defense
    &[
        "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "e2e3", "e1g1", "f1d3", "d7d5",
    ],
    // Slav Defense
    &[
        "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "b1c3", "e7e6", "e2e3", "b8d7",
    ],
];

const BLACK_VS_C4: &[&[&str]] = &[
    // Reversed Sicilian
    &[
        "e7e5", "b1c3", "g8f6", "g1f3", "b8c6", "g2g3", "d7d5", "c4d5", "f3d5",
    ],
    // Symmetrical English
    &[
        "c7c5", "g1f3", "b8c6", "d2d4", "c5d4", "f3d4", "g7g6", "e2e4", "f8g7",
    ],
];

const BLACK_VS_NF3: &[&[&str]] = &[
    // King's Indian setup vs Reti
    &[
        "g8f6", "c2c4", "g7g6", "g2g3", "f8g7", "f1g2", "e1g1", "b1c3", "d7d6",
    ],
    // Queen's Gambit style
    &[
        "d7d5", "d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8e7", "c1g5", "e1g1",
    ],
];

const BLACK_DEFAULT: &[&[&str]] = &[
    &["d7d5", "c2c4", "e7e6", "g1f3", "g8f6", "b1c3"],
    &["g8f6", "d2d4", "g7g6", "c2c4", "f8g7", "b1c3"],
];

pub struct GameBook {
    pub in_book: bool,
}

impl GameBook {
    pub fn new() -> Self {
        GameBook { in_book: true }
    }

    /// moves_played: UCI strings played so far this game (both sides).
    /// Returns Some(book move) or None if you should fall back to search.
    pub fn next_move(&mut self, moves_played: &[String], is_black: bool) -> Option<&'static str> {
        if !self.in_book {
            return None;
        }

        let offset = if is_black { 1 } else { 0 };

        // 1. Identify the relevant pool of lines
        let pool: &[&[&str]] = if !is_black {
            WHITE_OPENINGS
        } else {
            let first = match moves_played.first() {
                Some(f) => f,
                None => {
                    self.in_book = false;
                    return None;
                }
            };
            match first.as_str() {
                "e2e4" => BLACK_VS_E4,
                "d2d4" => BLACK_VS_D4,
                "c2c4" => BLACK_VS_C4,
                "g1f3" => BLACK_VS_NF3,
                _ => BLACK_DEFAULT,
            }
        };

        // 2. Filter pool to lines that match the moves played so far
        let relevant_played = &moves_played[offset..];
        let matching_lines: Vec<&&[&str]> = pool
            .iter()
            .filter(|line| {
                // The line must be at least as long as our history + the move we want to make
                if line.len() <= relevant_played.len() {
                    return false;
                }
                // The history must match the start of the book line
                relevant_played.iter().zip(line.iter()).all(|(p, e)| p == e)
            })
            .collect();

        if matching_lines.is_empty() {
            self.in_book = false;
            return None;
        }

        // 3. Pick a move from one of the matching lines
        let mut rng = rand::rng();
        let chosen_line = matching_lines[rng.random_range(0..matching_lines.len())];
        Some(chosen_line[relevant_played.len()])
    }
}
