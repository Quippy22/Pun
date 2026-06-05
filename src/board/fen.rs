use crate::board::{Color, Piece};
use crate::utils::string_to_square;

pub struct FenData {
    pub pieces: [u64; 12],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant_sq: Option<u8>,
    pub half_move: u16,
    pub full_move: u16,
}

impl FenData {
    pub fn parse(fen: &str) -> FenData {
        // split the fen into rows
        // the last element also containst the pieces
        let mut board = fen.split('/').collect::<Vec<&str>>();
        // split the last element into parts
        let mut parts = board.last().unwrap().split_whitespace();
        // remove the last element
        board.pop();
        // get the last row
        let pieces = parts.next().unwrap();
        // add the pieces to the board
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
