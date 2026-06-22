use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pun::board::Board;
use pun::perft::perft;
use std::time::Duration;

fn bench_perft(c: &mut Criterion) {
    let board =
        Board::initialize_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Low depth: fast feedback
    let mut low_depth_group = c.benchmark_group("perft_low_depth");
    for depth in 1..=4 {
        low_depth_group.bench_with_input(format!("depth_{}", depth), &depth, |b, &d| {
            b.iter(|| black_box(perft(&board, d)));
        });
    }
    low_depth_group.finish();

    // High depth: slower, needs more time
    let mut high_depth_group = c.benchmark_group("perft_high_depth");
    high_depth_group.measurement_time(Duration::from_secs(60)); // Give it more time

    high_depth_group.bench_with_input("depth_5", &5, |b, &d| {
        b.iter(|| black_box(perft(&board, d)));
    });

    // Reduce sample count for depth 6 to prevent timeouts
    high_depth_group.sample_size(10);
    high_depth_group.bench_with_input("depth_6", &6, |b, &d| {
        b.iter(|| black_box(perft(&board, d)));
    });

    high_depth_group.finish();
}

criterion_group!(benches, bench_perft);
criterion_main!(benches);
