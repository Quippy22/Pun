# Pun

Pun is a chess engine written in Rust as a learning project.

The goal here is not to produce a tournament-ready engine first. The goal is to understand the parts that make a chess engine work:

- board representation
- FEN parsing
- move generation
- state updates
- CLI / UCI-style interaction
- testing the rules with small, focused positions

The project is intentionally built in small steps so each subsystem can be tested and reasoned about on its own.

## Roadmap

- [x] FEN parsing
- [x] Bitboard board representation
- [x] Pseudo-legal move generation
- [x] UCI-style command loop
- [ ] Legal move generation
- [ ] Castling
- [ ] En passant
- [ ] Make/unmake move support
- [ ] Perft testing
- [ ] Search
- [ ] Evaluation
- [ ] Checkmate and stalemate detection
- [ ] Time management


## Philosophy

This project is deliberately incremental.

Each feature should be small enough to test directly, and each test should describe a concrete chess rule or engine behavior. That makes the codebase useful both as a learning exercise and as a foundation for a real engine later.
