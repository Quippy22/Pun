use crate::board::{Board, Color, Piece};

/// Special promotion flags
const PROMOTIONS: [u16; 4] = [0b1000, 0b1010, 0b1100, 0b1110];
/// The 8 possible knight moves as bit shifts
const KNIGHT_MOVES: [i8; 8] = [17, 15, 10, 6, -17, -15, -10, -6];
/// The 8 possible king moves as bit shifts
const KING_MOVES: [i8; 8] = [-9, -8, -7, -1, 1, 7, 8, 9];
/// The 4 possible bishop directions
const BISHOP_DIRECTIONS: [i16; 4] = [-9, -7, 7, 9];
/// The 4 possible rook directions
const ROOK_DIRECTIONS: [i16; 4] = [-1, 1, -8, 8];

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
    /// The main entry point of the move generator.
    /// Returns a vector of all possible moves for a given piece
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

    /// Returns the bitboard and the color of the piece.
    /// Automatically rotates the bitboard for the black pieces
    fn get_bitboard(board: &Board, piece: Piece) -> (u64, Color) {
        let (pieces, color) = match piece {
            Piece::WhitePawn => (board.pieces[piece as usize], Color::White),
            Piece::BlackPawn => (board.pieces[piece as usize].swap_bytes(), Color::Black),
            Piece::WhiteKnight => (board.pieces[piece as usize], Color::White),
            Piece::BlackKnight => (board.pieces[piece as usize].swap_bytes(), Color::Black),
            Piece::WhiteBishop => (board.pieces[piece as usize], Color::White),
            Piece::BlackBishop => (board.pieces[piece as usize].swap_bytes(), Color::Black),
            Piece::WhiteRook => (board.pieces[piece as usize], Color::White),
            Piece::BlackRook => (board.pieces[piece as usize].swap_bytes(), Color::Black),
            Piece::WhiteQueen => (board.pieces[piece as usize], Color::White),
            Piece::BlackQueen => (board.pieces[piece as usize].swap_bytes(), Color::Black),
            Piece::WhiteKing => (board.pieces[piece as usize], Color::White),
            Piece::BlackKing => (board.pieces[piece as usize].swap_bytes(), Color::Black),
        };
        (pieces, color)
    }

    /// Returns the bitboards for the own and enemy pieces
    /// Automatically rotates the bitboards for the black pieces
    fn get_sides(board: &Board, color: &Color) -> (u64, u64) {
        let (own_bitboard, enemy_bitboard) = match color {
            Color::White => (
                board.get_side_bitboard(&Color::White),
                board.get_side_bitboard(&Color::Black),
            ),
            Color::Black => (
                board.get_side_bitboard(&Color::Black).swap_bytes(),
                board.get_side_bitboard(&Color::White).swap_bytes(),
            ),
        };

        (own_bitboard, enemy_bitboard)
    }

    fn get_all_pawn_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, &color);
        let mut index: u16;
        let mut pawn: u64;
        let mut is_promotion: bool;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            pawn = 1 << index;
            is_promotion = index / 8 == 6;

            let mut flag = 0b0000;

            // -- Forward Moves --
            // check for pieces in front of the pawn
            if pawn << 8 & (own_pieces | enemy_pieces) == 0 {
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
                if index / 8 == 1 && pawn << 16 & (own_pieces | enemy_pieces) == 0 {
                    available_moves.push(match color {
                        Color::White => Move(index | (index + 16) << 6 | flag << 12),
                        Color::Black => Move((index ^ 56) | ((index ^ 56) - 16) << 6 | flag << 12),
                    });
                }
            }

            // -- Captures --
            // get all the enemy pieces
            // set the flag to capture
            flag |= 0b0001;

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

    fn get_all_knight_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let mut index: u16;
        let mut knight: u64;
        let (own_pieces, enemy_pieces) = Self::get_sides(board, &color);

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            knight = 1 << index;

            // check all the possible moves
            for m in KNIGHT_MOVES.iter() {
                let (moved_knight, target_index) = if m.is_negative() {
                    (knight >> m.abs() as u16, index.wrapping_sub(m.abs() as u16))
                } else {
                    (knight << m.abs() as u16, index.wrapping_add(m.abs() as u16))
                };
                // check if the move is valid
                if moved_knight == 0
                    || ((target_index % 8) as i16 - (index % 8) as i16).abs() > 2
                    || (moved_knight & own_pieces) != 0
                {
                    continue;
                }

                // check for capture
                let flag = if moved_knight & enemy_pieces != 0 {
                    0b0001
                } else {
                    0b0000
                };
                available_moves.push(match color {
                    Color::White => Move(index | target_index << 6 | flag << 12),
                    Color::Black => Move((index ^ 56) | (target_index ^ 56) << 6 | flag << 12),
                });
            }

            pieces &= pieces - 1;
        }
    }

    fn get_all_king_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, &color);
        let index: u16 = pieces.trailing_zeros() as u16;
        let king: u64 = 1 << index;

        // check all the possible moves
        for m in KING_MOVES.iter() {
            let (moved_king, target_index) = if m.is_negative() {
                (king >> m.abs() as u16, index.wrapping_sub(m.abs() as u16))
            } else {
                (king << m.abs() as u16, index.wrapping_add(m.abs() as u16))
            };
            // check if the move is valid
            if moved_king == 0
                || ((target_index % 8) as i16 - (index % 8) as i16).abs() > 1
                || (moved_king & own_pieces) != 0
            {
                continue;
            }

            // check for capture
            let flag = if moved_king & enemy_pieces != 0 {
                0b0001
            } else {
                0b0000
            };
            available_moves.push(match color {
                Color::White => Move(index | target_index << 6 | flag << 12),
                Color::Black => Move((index ^ 56) | (target_index ^ 56) << 6 | flag << 12),
            });
        }

        // TODO: check for castling
    }
    /// --- Slider Pieces ---
    // Shared parallel raycaster for Bishop, Rook, or Queen
    #[inline(always)]
    fn raycast_moves(
        index: u16,
        directions: &[i16],
        own_pieces: u64,
        enemy_pieces: u64,
        color: &Color,
        is_diagonal: bool,
        available_moves: &mut Vec<Move>,
    ) {
        // Track which directions are still active.
        // Index matches the direction array index
        let mut active_directions = 0b1111u8;

        // Keep track of the current square index for each of the 4 vectors
        let mut current_indices = [index as i16; 4];

        // a sliding piece can travel a maximum of 7 squares away
        for _ in 1..=7 {
            // if all 4 rays have hit an obstacle or the edge, exit
            if active_directions == 0 {
                break;
            }

            for i in 0..directions.len() {
                // if this specific direction is already blocked, skip it
                if (active_directions & (1 << i)) == 0 {
                    continue;
                }

                let prev_file = current_indices[i] % 8;
                current_indices[i] += directions[i];
                let current_idx = current_indices[i];

                // -- Check the Bounds --
                if current_idx < 0 || current_idx > 63 {
                    active_directions &= !(1 << i); // Deactivate this direction
                    continue;
                }

                let current_file = current_idx % 8;
                if is_diagonal {
                    if (current_file - prev_file).abs() != 1 {
                        active_directions &= !(1 << i);
                        continue;
                    }
                } else {
                    if directions[i].abs() >= 8 {
                        if current_file != prev_file {
                            active_directions &= !(1 << i);
                            continue;
                        }
                    } else {
                        if (current_file - prev_file).abs() != 1 {
                            active_directions &= !(1 << i);
                            continue;
                        }
                    }
                }

                // -- Check for obstacles --
                let target_index = current_idx as u16;
                let target_bit = 1 << target_index;

                if (target_bit & own_pieces) != 0 {
                    active_directions &= !(1 << i); // blocked completely
                    continue;
                }

                // -- Check for capture --
                let is_capture = (target_bit & enemy_pieces) != 0;
                let flag = if is_capture { 0b0001 } else { 0b0000 };

                available_moves.push(match color {
                    Color::White => Move(index | target_index << 6 | flag << 12),
                    Color::Black => Move((index ^ 56) | (target_index ^ 56) << 6 | flag << 12),
                });

                // Terminate ray after hitting the enemy
                if is_capture {
                    active_directions &= !(1 << i);
                }
            }
        }
    }

    fn get_all_bishop_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, &color);
        let mut index: u16;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            Self::raycast_moves(
                index,
                &BISHOP_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                &color,
                true,
                available_moves,
            );

            pieces &= pieces - 1;
        }
    }

    fn get_all_rook_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, &color);
        let mut index: u16;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            Self::raycast_moves(
                index,
                &ROOK_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                &color,
                false,
                available_moves,
            );

            pieces &= pieces - 1;
        }
    }

    fn get_all_queen_moves(board: &Board, piece: Piece, available_moves: &mut Vec<Move>) {
        let (mut pieces, color) = Self::get_bitboard(board, piece);
        let (own_pieces, enemy_pieces) = Self::get_sides(board, &color);
        let mut index: u16;

        while pieces != 0 {
            index = pieces.trailing_zeros() as u16;
            Self::raycast_moves(
                index,
                &BISHOP_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                &color,
                true,
                available_moves,
            );
            Self::raycast_moves(
                index,
                &ROOK_DIRECTIONS,
                own_pieces,
                enemy_pieces,
                &color,
                false,
                available_moves,
            );

            pieces &= pieces - 1;
        }
    }
}
