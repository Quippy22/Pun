use crate::board::{Board, Color, Piece, PieceType};

const PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_PST: [i32; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0,
    0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 5, 10, 10, 10, 10, 10, 10, 5, 0, 0,
    0, 0, 0, 0, 0, 0,
];

const QUEEN_PST: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 5, 0, -10, -10, 0, 5, 5, 5, 5, 5, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const KING_PST: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

fn mirror_square(square: usize) -> usize {
    square ^ 56
}

fn pst_for(kind: PieceType) -> &'static [i32; 64] {
    match kind {
        PieceType::Pawn => &PAWN_PST,
        PieceType::Knight => &KNIGHT_PST,
        PieceType::Bishop => &BISHOP_PST,
        PieceType::Rook => &ROOK_PST,
        PieceType::Queen => &QUEEN_PST,
        PieceType::King => &KING_PST,
    }
}

/// Evaluates the board from the perspective of the side to move.
///
/// Positive scores favor the side whose turn it is to play.
pub fn evaluate(board: &Board) -> i32 {
    let mut white_score = 0;
    let mut black_score = 0;

    for piece in Piece::all() {
        let kind = piece.kind();
        let base_value = kind.value();
        let pst = pst_for(kind);
        let mut bitboard = board.pieces[piece as usize];

        while bitboard != 0 {
            let square = bitboard.trailing_zeros() as usize;
            bitboard &= bitboard - 1;

            let table_index = match piece.color() {
                Color::White => square,
                Color::Black => mirror_square(square),
            };
            let score = base_value + pst[table_index];

            match piece.color() {
                Color::White => white_score += score,
                Color::Black => black_score += score,
            }
        }
    }

    let material_balance = white_score - black_score;

    match board.side_to_move {
        Color::White => material_balance,
        Color::Black => -material_balance,
    }
}
