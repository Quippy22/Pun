use crate::board::{Color, Piece};
use crate::utils::string_to_square;

/// Parsed FEN fields used to initialize a `Board`.
pub struct FenData {
    /// Piece bitboards in board storage order.
    pub pieces: [u64; 12],
    /// Side to move.
    pub side_to_move: Color,
    /// Castling-right bit mask.
    pub castling_rights: u8,
    /// En passant target square, if present.
    pub en_passant_sq: Option<u8>,
    /// Halfmove clock from the FEN.
    pub half_move: u16,
    /// Fullmove number from the FEN.
    pub full_move: u16,
}

impl FenData {
    /// Parses a full FEN string into structured board data.
    pub fn parse(fen: &str) -> FenData {
        // Split the board layout from the rest of the FEN. The last slash
        // segment still contains the side-to-move and metadata fields.
        let mut board = fen.split('/').collect::<Vec<&str>>();
        let mut parts = board.last().unwrap().split_whitespace();
        board.pop();
        let pieces = parts.next().unwrap();
        // Put the last board row back so all 8 ranks can be processed uniformly.
        board.push(pieces);

        let side = match parts.next().unwrap() {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Invalid side"),
        };
        let castle: u8 = {
            let chars = parts.next().unwrap().chars().collect::<Vec<char>>();
            let mut rights: u8 = 0b0000;
            for c in chars {
                match c {
                    'K' => rights |= 0b0001,
                    'Q' => rights |= 0b0010,
                    'k' => rights |= 0b0100,
                    'q' => rights |= 0b1000,
                    '-' => {}
                    _ => panic!("Invalid castling rights"),
                }
            }
            rights
        };
        let en_passant_sq = {
            let string = parts.next().unwrap();
            match string {
                "-" => None,
                string => Some(string_to_square(string)),
            }
        };
        let half_move: u16 = parts.next().unwrap().parse().unwrap();
        let full_move: u16 = parts.next().unwrap().parse().unwrap();

        Self {
            pieces: Self::pieces_to_bitboard(board),
            side_to_move: side,
            castling_rights: castle,
            en_passant_sq,
            half_move,
            full_move,
        }
    }

    fn pieces_to_bitboard(pieces: Vec<&str>) -> [u64; 12] {
        // create an empty bitboard
        let mut bitboard = [0u64; 12];
        // iterate line by line
        for (i, line) in pieces.iter().enumerate() {
            // the FEN starts upside down, so we need to reverse the ranks
            let rank = 7 - i as u8;
            // manually index the file
            let mut file: u8 = 0;
            // iterate char by char
            for c in line.chars() {
                if c.is_ascii_digit() {
                    // if it's a digit,
                    // skip that number of empty squares
                    file += c.to_digit(10).unwrap() as u8;
                } else {
                    // get the piece
                    let piece = match c {
                        'P' => Piece::WhitePawn,
                        'R' => Piece::WhiteRook,
                        'N' => Piece::WhiteKnight,
                        'B' => Piece::WhiteBishop,
                        'Q' => Piece::WhiteQueen,
                        'K' => Piece::WhiteKing,
                        'p' => Piece::BlackPawn,
                        'r' => Piece::BlackRook,
                        'n' => Piece::BlackKnight,
                        'b' => Piece::BlackBishop,
                        'q' => Piece::BlackQueen,
                        'k' => Piece::BlackKing,
                        _ => panic!("Invalid piece"),
                    };
                    bitboard[piece as usize] |= 1u64 << (rank * 8 + file);

                    file += 1;
                }
            }
        }
        bitboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper --
    /// Bit index for a square given algebraic notation rank/file.
    /// rank 1-8, file 0-7 (a=0 … h=7).
    fn sq(file: u8, rank: u8) -> u64 {
        1u64 << ((rank - 1) * 8 + file)
    }

    mod bitboards {
        use super::*;
        /// Starting position — every piece on its canonical square.
        /// Tests both digit skipping (the pawn ranks use a single "8")
        /// and letter parsing for every piece type.
        #[test]
        fn test_pieces_starting_position() {
            let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
            let data = FenData::parse(fen);

            // White pawns on rank 2 (a2-h2)
            assert_eq!(
                data.pieces[Piece::WhitePawn as usize],
                0x0000_0000_0000_FF00
            );
            // Black pawns on rank 7 (a7-h7)
            assert_eq!(
                data.pieces[Piece::BlackPawn as usize],
                0x00FF_0000_0000_0000
            );

            // White back rank
            assert_eq!(data.pieces[Piece::WhiteRook as usize], sq(0, 1) | sq(7, 1));
            assert_eq!(
                data.pieces[Piece::WhiteKnight as usize],
                sq(1, 1) | sq(6, 1)
            );
            assert_eq!(
                data.pieces[Piece::WhiteBishop as usize],
                sq(2, 1) | sq(5, 1)
            );
            assert_eq!(data.pieces[Piece::WhiteQueen as usize], sq(3, 1));
            assert_eq!(data.pieces[Piece::WhiteKing as usize], sq(4, 1));

            // Black back rank
            assert_eq!(data.pieces[Piece::BlackRook as usize], sq(0, 8) | sq(7, 8));
            assert_eq!(
                data.pieces[Piece::BlackKnight as usize],
                sq(1, 8) | sq(6, 8)
            );
            assert_eq!(
                data.pieces[Piece::BlackBishop as usize],
                sq(2, 8) | sq(5, 8)
            );
            assert_eq!(data.pieces[Piece::BlackQueen as usize], sq(3, 8));
            assert_eq!(data.pieces[Piece::BlackKing as usize], sq(4, 8));
        }

        /// Rank encoded as a single digit "8" → all squares empty.
        #[test]
        fn test_pieces_empty_rank_single_digit() {
            // Only kings present; everything else must be zero
            let fen = "8/8/8/8/8/8/8/4K2k w - - 0 1";
            let data = FenData::parse(fen);

            assert_eq!(data.pieces[Piece::WhiteKing as usize], sq(4, 1)); // e1
            assert_eq!(data.pieces[Piece::BlackKing as usize], sq(7, 1)); // h1
            // Verify all other bitboards are empty
            for piece in [
                Piece::WhitePawn,
                Piece::WhiteKnight,
                Piece::WhiteBishop,
                Piece::WhiteRook,
                Piece::WhiteQueen,
                Piece::BlackPawn,
                Piece::BlackKnight,
                Piece::BlackBishop,
                Piece::BlackRook,
                Piece::BlackQueen,
            ] {
                assert_eq!(data.pieces[piece as usize], 0, "{piece:?} should be empty");
            }
        }

        /// Rank with mixed digits and letters: "3r2r1" → rooks on d and g files.
        #[test]
        fn test_pieces_mixed_digits_and_letters() {
            // rank 8: . . . r . . r .   (d8, g8)
            let fen = "3r2r1/8/8/8/8/8/8/4K2k w - - 0 1";
            let data = FenData::parse(fen);

            assert_eq!(
                data.pieces[Piece::BlackRook as usize],
                sq(3, 8) | sq(6, 8) // d8 = file 3, g8 = file 6
            );
        }

        /// Multiple consecutive digit groups: "2p3P1" on the same rank.
        #[test]
        fn test_pieces_consecutive_digit_groups() {
            // rank 5: . . p . . . P .   (c5 black pawn, g5 white pawn)
            let fen = "8/8/8/2p3P1/8/8/8/4K2k w - - 0 1";
            let data = FenData::parse(fen);

            assert_eq!(data.pieces[Piece::BlackPawn as usize], sq(2, 5)); // c5
            assert_eq!(data.pieces[Piece::WhitePawn as usize], sq(6, 5)); // g5
        }

        /// A fully-packed rank with no digits at all: "RNBQKBNR".
        #[test]
        fn test_pieces_fully_packed_rank_no_digits() {
            let fen = "8/8/8/8/8/8/8/RNBQKBNR w - - 0 1";
            let data = FenData::parse(fen);

            assert_eq!(data.pieces[Piece::WhiteRook as usize], sq(0, 1) | sq(7, 1));
            assert_eq!(
                data.pieces[Piece::WhiteKnight as usize],
                sq(1, 1) | sq(6, 1)
            );
            assert_eq!(
                data.pieces[Piece::WhiteBishop as usize],
                sq(2, 1) | sq(5, 1)
            );
            assert_eq!(data.pieces[Piece::WhiteQueen as usize], sq(3, 1));
            assert_eq!(data.pieces[Piece::WhiteKing as usize], sq(4, 1));
        }
    }

    mod side_to_move {
        use super::*;

        #[test]
        fn test_side_to_move_white() {
            let fen = "8/8/8/8/8/8/8/4K2k w - - 0 1";
            assert!(matches!(FenData::parse(fen).side_to_move, Color::White));
        }

        #[test]
        fn test_side_to_move_black() {
            let fen = "8/8/8/8/8/8/8/4K2k b - - 0 1";
            assert!(matches!(FenData::parse(fen).side_to_move, Color::Black));
        }
    }

    mod castling_rights {
        use super::*;
        #[test]
        fn test_castling_all_rights() {
            let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
            assert_eq!(FenData::parse(fen).castling_rights, 0b1111);
        }

        #[test]
        fn test_castling_no_rights() {
            let fen = "8/8/8/8/8/8/8/4K2k w - - 0 1";
            assert_eq!(FenData::parse(fen).castling_rights, 0b0000);
        }

        #[test]
        fn test_castling_white_kingside_only() {
            let fen = "8/8/8/8/8/8/8/4K2k w K - 0 1";
            assert_eq!(FenData::parse(fen).castling_rights, 0b0001);
        }

        #[test]
        fn test_castling_black_queenside_only() {
            let fen = "8/8/8/8/8/8/8/4K2k w q - 0 1";
            assert_eq!(FenData::parse(fen).castling_rights, 0b1000);
        }
    }

    mod en_passant {
        use super::*;
        #[test]
        fn test_en_passant_none() {
            let fen = "8/8/8/8/8/8/8/4K2k w - - 0 1";
            assert_eq!(FenData::parse(fen).en_passant_sq, None);
        }

        #[test]
        fn test_en_passant_e6() {
            let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
            // e6 = file 4, rank 6 → index = (6-1)*8 + 4 = 44
            assert_eq!(FenData::parse(fen).en_passant_sq, Some(44));
        }

        #[test]
        fn test_en_passant_a3() {
            let fen = "8/8/8/8/Pp6/8/8/4K2k b - a3 0 1";
            // a3 = file 0, rank 3 → index = 2*8 + 0 = 16
            assert_eq!(FenData::parse(fen).en_passant_sq, Some(16));
        }
    }

    mod move_counters {
        use super::*;
        #[test]
        fn test_move_counters_initial() {
            let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
            let data = FenData::parse(fen);
            assert_eq!(data.half_move, 0);
            assert_eq!(data.full_move, 1);
        }

        #[test]
        fn test_move_counters_midgame() {
            let fen = "r1bqkb1r/pp3ppp/2n1pn2/3p4/3P4/2N1PN2/PP3PPP/R1BQKB1R w KQkq - 2 7";
            let data = FenData::parse(fen);
            assert_eq!(data.half_move, 2);
            assert_eq!(data.full_move, 7);
        }

        #[test]
        fn test_move_counters_large_values() {
            let fen = "8/8/8/8/8/8/8/4K2k w - - 100 250";
            let data = FenData::parse(fen);
            assert_eq!(data.half_move, 100);
            assert_eq!(data.full_move, 250);
        }
    }
}
