use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pun::board::Board;
use pun::search::negmax::negmax;
use pun::search::transposition_table::TranspositionTable;
use std::sync::{Arc, atomic::AtomicBool};

fn bench_search(c: &mut Criterion) {
    let board =
        Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let tt = TranspositionTable::new(16); // 16MB TT
    let stop = Arc::new(AtomicBool::new(false));

    c.bench_function("negmax_depth_6", |b| {
        b.iter(|| {
            let mut b = board.clone();
            negmax(black_box(&mut b), &tt, 6, stop.clone())
        });
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
