use crate::board::moves::{Move, MoveGenerator};
use crate::board::{Board, Piece, PieceType};
use crate::search::transposition_table::{TTEntry, TranspositionTable};

use super::eval::evaluate;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const INF: i32 = 1_000_000;
const MATE_VALUE: i32 = 100_000;

/// Searches for the best move using negamax with alpha-beta pruning and Transposition Table.
pub fn negmax(
    board: &mut Board,
    tt: &TranspositionTable,
    depth: u32,
    stop: Arc<AtomicBool>,
) -> Option<(Move, i32)> {
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
        let score = match negmax_score(board, tt, depth.saturating_sub(1), -beta, -alpha, &stop) {
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

fn negmax_score(
    board: &mut Board,
    tt: &TranspositionTable,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    stop: &AtomicBool,
) -> Option<i32> {
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // Transposition Table Lookup
    if let Some(entry) = tt.get(board.zobrist_hash) {
        if entry.depth >= depth as u8 {
            return Some(entry.value);
        }
    }

    if depth == 0 {
        return Some(evaluate(board));
    }

    let mut moves = Vec::new();
    MoveGenerator::get_all_moves(board, board.side_to_move, &mut moves);

    if moves.is_empty() {
        let king_sq = board.pieces[Piece::new(board.side_to_move, PieceType::King) as usize];
        if MoveGenerator::is_check(board, board.side_to_move, king_sq) {
            return Some(-MATE_VALUE);
        } else {
            return Some(0); // Stalemate
        }
    }

    let mut best_score = -INF;
    let mut best_move = None;

    for mv in moves {
        if stop.load(Ordering::Relaxed) {
            return None;
        }

        board.make_move(&mv);
        let score = match negmax_score(board, tt, depth - 1, -beta, -alpha, stop) {
            Some(score) => -score,
            None => {
                board.unmake_move();
                return None;
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
        if alpha >= beta {
            break;
        }
    }

    // Transposition Table Store
    if let Some(mv) = best_move {
        tt.put(TTEntry {
            key: board.zobrist_hash,
            value: best_score,
            depth: depth as u8,
            flag: 0, // Simplified: Assuming EXACT
            best_move: mv,
        });
    }

    Some(best_score)
}
