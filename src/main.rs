mod board;
use std::io::{self, BufRead};

use crate::board::moves::MoveGenerator;
use crate::board::{Board, Piece};

fn main() {
    let stdin = io::stdin();
    let mut iterator = stdin.lock().lines();
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut board = Board::initialize_from_fen(fen);

    while let Some(line) = iterator.next() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("id name Pun");
                println!("id author Quippy");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "position" => match tokens[1] {
                "fen" => {
                    let fen = tokens[2..].join(" ");
                    board = Board::initialize_from_fen(&fen);
                }
                "startpos" => {
                    if let Some(&last_move) = tokens.last() {
                        board.update_state(last_move);
                    }
                }
                _ => {}
            },
            "go" => {
                let mut moves = vec![];
                MoveGenerator::get_possible_moves(&board, Piece::WhitePawn, &mut moves);
                if !moves.is_empty() {
                    println!("bestmove {}", moves[3]);
                } else {
                    println!("bestmove 0000");
                }
            }
            "quit" => {
                break;
            }
            _ => {}
        }
    }
}
