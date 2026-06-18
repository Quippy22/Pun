use crate::board::Board;
use crate::board::moves::{Move, MoveGenerator};

use super::eval::evaluate;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const INF: i32 = 1_000_000;

/// Searches for the best move using negamax with alpha-beta pruning.
pub fn negmax(board: &mut Board, depth: u32, stop: Arc<AtomicBool>) -> Option<(Move, i32)> {
    let mut moves = Vec::new();
    MoveGenerator::get_all_moves(board, board.side_to_move, &mut moves);

    let mut best_move = None;
    let mut best_score = -INF;
    let mut alpha = -INF;
    let beta = INF;

    for mv in moves {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        board.make_move(&mv);
        let score = match negamax_score(board, depth.saturating_sub(1), -beta, -alpha, &stop) {
            Some(score) => -score,
            None => {
                board.unmake_move();
                break;
            }
        };
        board.unmake_move();

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
    }

    best_move.map(|mv| (mv, best_score))
}

fn negamax_score(
    board: &mut Board,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    stop: &AtomicBool,
) -> Option<i32> {
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    if depth == 0 {
        return Some(evaluate(board));
    }

    let mut moves = Vec::new();
    MoveGenerator::get_all_moves(board, board.side_to_move, &mut moves);

    if moves.is_empty() {
        return Some(evaluate(board));
    }

    let mut best_score = -INF;

    for mv in moves {
        if stop.load(Ordering::Relaxed) {
            return None;
        }

        board.make_move(&mv);
        let score = match negamax_score(board, depth - 1, -beta, -alpha, stop) {
            Some(score) => -score,
            None => {
                board.unmake_move();
                return None;
            }
        };
        board.unmake_move();

        if score > best_score {
            best_score = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    Some(best_score)
}
