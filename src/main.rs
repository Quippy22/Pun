mod board;
use crate::board::Board;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::initialize_from_fen(fen);
    println!("{:#?}", board);
}
