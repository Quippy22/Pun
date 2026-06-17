use crate::board::Board;
use crate::board::moves::MoveGenerator;
use std::collections::HashMap;

pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    MoveGenerator::get_all_moves(board, board.side_to_move, &mut moves);

    let mut nodes = 0u64;
    for mv in &moves {
        let mut child = board.clone();
        child.make_move(mv);
        nodes += perft(&child, depth - 1);
    }

    nodes
}

pub fn divide(board: &Board, depth: u32) -> u64 {
    let mut moves = Vec::new();
    MoveGenerator::get_all_moves(board, board.side_to_move, &mut moves);

    let mut total = 0u64;
    for mv in &moves {
        let mut child = board.clone();
        child.make_move(mv);
        let nodes = perft(&child, depth - 1);
        total += nodes;
        println!("{}: {}", mv.to_uci(), nodes);
    }
    println!("\nTotal: {}", total);
    total
}

pub fn divide_map(board: &Board, depth: u32) -> HashMap<String, u64> {
    let mut moves = Vec::new();
    MoveGenerator::get_all_moves(board, board.side_to_move, &mut moves);

    let mut result = HashMap::new();
    for mv in &moves {
        let mut child = board.clone();
        child.make_move(mv);
        let nodes = perft(&child, depth - 1);
        result.insert(mv.to_uci(), nodes);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_depth_1() {
        let board =
            Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft(&board, 1), 20);
    }

    #[test]
    fn starting_position_depth_2() {
        let board =
            Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft(&board, 2), 400);
    }

    #[test]
    fn starting_position_depth_3() {
        let board =
            Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft(&board, 3), 8902);
    }

    #[test]
    fn starting_position_depth_4() {
        let board =
            Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft(&board, 4), 197281);
    }

    #[test]
    fn starting_position_depth_5() {
        let board =
            Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft(&board, 5), 4865609);
    }

    #[test]
    fn kiwipete_depth_1() {
        let board = Board::initialize_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );
        assert_eq!(perft(&board, 1), 48);
    }

    #[test]
    fn kiwipete_depth_2() {
        let board = Board::initialize_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );
        assert_eq!(perft(&board, 2), 2039);
    }

    #[test]
    fn kiwipete_depth_3() {
        let board = Board::initialize_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );
        assert_eq!(perft(&board, 3), 97862);
    }

    #[test]
    fn kiwipete_depth_4() {
        let board = Board::initialize_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );
        assert_eq!(perft(&board, 4), 4085603);
    }
}
