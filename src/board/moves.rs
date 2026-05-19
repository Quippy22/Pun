use crate::board::{Board, Color, Piece};

/// Special promotion flags
const PROMOTIONS: [u16; 4] = [0b1000, 0b1010, 0b1100, 0b1110];

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
        (self.0 & 0b111111) as usize
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

pub struct MoveGenerator;

impl MoveGenerator {
    /// the main entry point of the move generator
    /// returns a vector of all possible moves for a given piece
    pub fn get_possible_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        match piece {
            Piece::WhitePawn | Piece::BlackPawn => {
                Self::get_all_pawn_moves(board, piece, available_moves)
            }
            Piece::WhiteKnight | Piece::BlackKnight => {
                Self::get_all_knight_moves(board, piece, available_moves)
            }
            Piece::WhiteBishop | Piece::BlackBishop => {
                Self::get_all_bishop_moves(board, piece, available_moves)
            }
            Piece::WhiteRook | Piece::BlackRook => {
                Self::get_all_rook_moves(board, piece, available_moves)
            }
            Piece::WhiteQueen | Piece::BlackQueen => {
                Self::get_all_queen_moves(board, piece, available_moves)
            }
            Piece::WhiteKing | Piece::BlackKing => {
                Self::get_all_king_moves(board, piece, available_moves)
            }
        }
    }
    fn get_all_pawn_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = match piece {
            Piece::WhitePawn => (board.pieces[piece as usize], Color::White),
            Piece::BlackPawn => (board.pieces[piece as usize].swap_bytes(), Color::Black),
            _ => panic!("Invalid pawn"),
        };

        while pieces != 0 {
            // get the index of the first piece
            let index: u16 = pieces.trailing_zeros() as u16;
            let pawn: u64 = 1 << index;
            let is_promotion: bool = index / 8 == 6;

            // -- Forward Moves --
            // also swap the board bytes
            if pawn << 8 & board.get_side_bitboard(&color).swap_bytes() == 0 {
                let flag = 0b0000;
                // if the pawn is on the 7th rank
                // add the promotion flag
                if is_promotion {
                    for p in PROMOTIONS.iter() {
                        available_moves.push(match color {
                            Color::White => Move(index | (index + 8) << 6 | (p | flag) << 12),
                            Color::Black => {
                                Move((index ^ 56) | ((index ^ 56) - 8) << 6 | (p | flag) << 12)
                            }
                        });
                    }
                } else {
                    available_moves.push(match color {
                        Color::White => Move(index | (index + 8) << 6 | flag << 12),
                        Color::Black => Move((index ^ 56) | ((index ^ 56) - 8) << 6 | flag << 12),
                    });
                }

                // if the pawn is on the 2nd rank
                // add the 2 square move
                if index / 8 == 1 && pawn << 16 & board.get_side_bitboard(&color).swap_bytes() == 0
                {
                    available_moves.push(match color {
                        Color::White => Move(index | (index + 16) << 6 | flag << 12),
                        Color::Black => Move((index ^ 56) | ((index ^ 56) - 16) << 6 | flag << 12),
                    });
                }
            }

            // -- Captures --
            // get all the enemy pieces
            let enemy_pieces = match color {
                Color::White => board.get_side_bitboard(&Color::Black),
                Color::Black => board.get_side_bitboard(&Color::White).swap_bytes(),
            };

            // set the flag to capture
            let flag = 0b0001;

            let file = index % 8;

            // 1. Capture Left (Towards the A-File)
            // A pawn can only capture left if it's NOT on the A-file (file 0)
            if file != 0 {
                if (pawn << 7) & enemy_pieces != 0 {
                    available_moves.push(match color {
                        Color::White => Move(index | (index + 7) << 6 | flag << 12),
                        Color::Black => Move((index ^ 56) | ((index ^ 56) - 9) << 6 | flag << 12),
                    });

                    //check if the pawn takes with a promotion
                    if is_promotion {
                        for p in PROMOTIONS.iter() {
                            available_moves.push(match color {
                                Color::White => Move(index | (index + 7) << 6 | (p | flag) << 12),
                                Color::Black => {
                                    Move((index ^ 56) | ((index ^ 56) - 9) << 6 | (p | flag) << 12)
                                }
                            });
                        }
                    }
                }
            }

            // 2. Capture Right (Towards the H-File)
            // A pawn can only capture right if it's NOT on the H-file (file 7)
            if file != 7 {
                if (pawn << 9) & enemy_pieces != 0 {
                    available_moves.push(match color {
                        Color::White => Move(index | (index + 9) << 6 | flag << 12),
                        Color::Black => Move((index ^ 56) | ((index ^ 56) - 7) << 6 | flag << 12),
                    });
                }

                //check if the pawn takes with a promotion
                if is_promotion {
                    for p in PROMOTIONS.iter() {
                        available_moves.push(match color {
                            Color::White => Move(index | (index + 9) << 6 | (p | flag) << 12),
                            Color::Black => {
                                Move((index ^ 56) | ((index ^ 56) - 7) << 6 | (p | flag) << 12)
                            }
                        });
                    }
                }
            }
            // -- En Passant --
            // TODO: implement en passant check

            // clear the bit
            pieces &= pieces - 1;
        }
    }
    fn get_all_knight_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {}
    fn get_all_bishop_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {}
    fn get_all_rook_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {}
    fn get_all_queen_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {}
    fn get_all_king_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {}
}
