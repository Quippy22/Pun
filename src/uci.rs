use std::io::{self, BufRead, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::Duration;

use crate::board::moves::Move;
use crate::board::{Board, Color};
use crate::search::negmax::negmax;

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const DEFAULT_SEARCH_DEPTH: u32 = 4;

struct SearchJob {
    stop: Arc<AtomicBool>,
    result_rx: mpsc::Receiver<Option<(Move, i32)>>,
}

pub fn uci_loop() {
    let (command_tx, command_rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if command_tx.send(line).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut board = Board::initialize_from_fen(STARTPOS_FEN);
    let mut current_search: Option<SearchJob> = None;
    let mut search_poll = Duration::from_millis(25);

    let mut game_book = crate::board::moves::openings::GameBook::new();
    let mut moves_played: Vec<String> = Vec::new();

    loop {
        if let Some(job) = current_search.as_ref() {
            match job.result_rx.try_recv() {
                Ok(Some((mv, score))) => {
                    println!("info string [go] score {} playing {}", score, mv);
                    println!("bestmove {}", mv.to_uci());
                    io::stdout().flush().unwrap();
                    current_search = None;
                }
                Ok(None) => {
                    current_search = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    current_search = None;
                }
            }
        }

        let line = match command_rx.recv_timeout(search_poll) {
            Ok(line) => line,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if current_search.is_some() {
                    continue;
                }
                break;
            }
        };

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("id name Pun");
                println!("id author Quippy");
                println!("uciok");
                io::stdout().flush().unwrap();
            }
            "isready" => {
                println!("readyok");
                io::stdout().flush().unwrap();
            }
            "setoption" => {}
            "ucinewgame" => {
                board = Board::initialize_from_fen(STARTPOS_FEN);
                game_book = crate::board::moves::openings::GameBook::new();
                moves_played.clear();
            }
            "position" => {
                if let Some(job) = current_search.as_ref() {
                    job.stop.store(true, Ordering::Relaxed);
                }

                if tokens.len() < 2 {
                    continue;
                }

                let moves_idx = tokens.iter().position(|&t| t == "moves");

                match tokens[1] {
                    "startpos" => {
                        board = Board::initialize_from_fen(STARTPOS_FEN);
                        game_book = crate::board::moves::openings::GameBook::new();
                        moves_played.clear();
                    }
                    "fen" => {
                        let fen_end = moves_idx.unwrap_or(tokens.len());
                        let fen = tokens[2..fen_end].join(" ");
                        board = Board::initialize_from_fen(&fen);
                        game_book = crate::board::moves::openings::GameBook::new();
                        moves_played.clear();
                    }
                    _ => continue,
                }

                if let Some(idx) = moves_idx {
                    for mv in &tokens[idx + 1..] {
                        board.make_move(&Move::from_uci(mv));
                        moves_played.push(mv.to_string());
                    }
                }
            }
            "go" => {
                if let Some(job) = current_search.take() {
                    job.stop.store(true, Ordering::Relaxed);
                }

                if let Some(book_move) =
                    game_book.next_move(&moves_played, board.side_to_move == Color::Black)
                {
                    println!("bestmove {}", book_move);
                    io::stdout().flush().unwrap();
                    continue;
                }

                let depth = tokens
                    .iter()
                    .position(|&t| t == "depth")
                    .and_then(|idx| tokens.get(idx + 1))
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(DEFAULT_SEARCH_DEPTH);

                let stop = Arc::new(AtomicBool::new(false));
                let thread_stop = Arc::clone(&stop);
                let mut search_board = board.clone();
                let (result_tx, result_rx) = mpsc::channel();

                thread::spawn(move || {
                    let result = negmax(&mut search_board, depth, thread_stop);
                    let _ = result_tx.send(result);
                });

                current_search = Some(SearchJob { stop, result_rx });
            }
            "stop" => {
                if let Some(job) = current_search.as_ref() {
                    job.stop.store(true, Ordering::Relaxed);
                }
            }
            "quit" => {
                if let Some(job) = current_search.as_ref() {
                    job.stop.store(true, Ordering::Relaxed);
                }
                break;
            }
            _ => {}
        }

        search_poll = Duration::from_millis(25);
    }
}
