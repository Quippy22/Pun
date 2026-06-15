use std::io::{self, BufRead, Write};

use rand::seq::SliceRandom;

use crate::board::Board;
use crate::board::moves::{Move, MoveGenerator};

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn uci_loop() {
    let stdin = io::stdin();
    let mut board = Board::initialize_from_fen(STARTPOS_FEN);
    let mut moves: Vec<Move> = vec![];

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("info string [uci] handshake received");
                println!("id name Pun");
                println!("id author Quippy");
                println!("uciok");
                io::stdout().flush().unwrap();
            }

            "isready" => {
                println!("info string [isready] engine is ready");
                println!("readyok");
                io::stdout().flush().unwrap();
            }

            "position" => {
                if tokens.len() < 2 {
                    println!("info string [position] no arguments given, ignoring");
                    io::stdout().flush().unwrap();
                    continue;
                }

                let moves_idx = tokens.iter().position(|&t| t == "moves");

                match tokens[1] {
                    "startpos" => {
                        println!("info string [position] startpos received, resetting board");
                        board = Board::initialize_from_fen(STARTPOS_FEN);
                    }
                    "fen" => {
                        let fen_end = moves_idx.unwrap_or(tokens.len());
                        let fen = tokens[2..fen_end].join(" ");
                        println!("info string [position] fen received: {}", fen);
                        board = Board::initialize_from_fen(&fen);
                    }
                    other => {
                        println!("info string [position] unknown subcommand: {}", other);
                        io::stdout().flush().unwrap();
                        continue;
                    }
                }

                if let Some(idx) = moves_idx {
                    let move_list = &tokens[idx + 1..];
                    println!(
                        "info string [position] applying {} move(s): {}",
                        move_list.len(),
                        move_list.join(", ")
                    );
                    for mv in move_list {
                        board.make_move(&Move::from_uci(mv));
                    }
                } else {
                    println!("info string [position] no moves to apply");
                }

                io::stdout().flush().unwrap();
            }

            "go" => {
                // Find all the possible moves for the current board
                MoveGenerator::get_all_moves(&board, board.side_to_move, &mut moves);
                moves.shuffle(&mut rand::rng());

                if !moves.is_empty() {
                    println!("info string [go] playing {}", moves[0]);
                    println!("bestmove {}", moves[0].to_uci());
                    // board.make_move(&moves[0]);
                } else {
                    println!("info string [go] no moves found, sending null move");
                    println!("bestmove 0000");
                }
                io::stdout().flush().unwrap();

                // clear the moves vector
                moves.clear();
            }

            "quit" => {
                println!("info string [quit] shutting down");
                io::stdout().flush().unwrap();
                break;
            }

            other => {
                println!("info string [unknown] unrecognized command: {}", other);
                io::stdout().flush().unwrap();
            }
        }
    }
}
