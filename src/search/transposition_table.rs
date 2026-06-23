use crate::board::moves::Move;
use std::sync::Mutex;

/// Transposition table entry.
/// The key is a hash of the board state.
/// The value is the best evaluation of the position.
/// The depth is the depth of the best move.
/// Best move is of type Move for easier assignment.
#[derive(Clone, Copy, Default)]
pub struct TTEntry {
    pub key: u64,
    pub value: i32,
    pub depth: u8,
    pub flag: u8,
    pub best_move: Move,
}

/// A fixed-size hash table used to store previously evaluated board positions.
///
/// The table uses a contiguous `Vec` for cache-friendly access, wrapped in a
/// `Mutex` for safe shared access across threads during search.
///
/// # Indexing Logic
/// The indexing uses the modulo operator (`hash % size`) to map the large
/// (64-bit) Zobrist hash space into the finite, valid index range of the
/// array (0 to `size - 1`). This provides an extremely fast mapping
/// operation essential for performance in search.
///
/// # Collision Policy
/// This implementation uses a "replace" policy where new entries overwrite old ones
/// at the same index in the event of a collision.
pub struct TranspositionTable {
    table: Mutex<Vec<TTEntry>>,
    size: usize,
}

impl TranspositionTable {
    pub fn new(size_in_mb: usize) -> Self {
        // Calculate number of entries: (Size in bytes) / (Size of TTEntry in bytes)
        let num_entries = (size_in_mb * 1024 * 1024) / std::mem::size_of::<TTEntry>();

        // Initialize with default entries (key will be 0, indicating empty)
        let table = Mutex::new(vec![TTEntry::default(); num_entries]);

        Self {
            table,
            size: num_entries,
        }
    }

    // Get an entry based on the current board hash
    pub fn get(&self, hash: u64) -> Option<TTEntry> {
        let index = (hash as usize) % self.size;
        let table = self.table.lock().unwrap();
        let entry = table[index];

        // Check if the key matches (verifies it's not a collision)
        if entry.key == hash && entry.depth != 0 {
            Some(entry)
        } else {
            None
        }
    }

    // Store an entry
    pub fn put(&self, entry: TTEntry) {
        let index = (entry.key as usize) % self.size;
        let mut table = self.table.lock().unwrap();
        table[index] = entry;
    }
}

/// Zobrist hashing constants for unique board state identification.
pub struct ZobristKeys {
    /// 64 squares for each of the 12 piece types.
    pub pieces: [[u64; 64]; 12],
    /// 16 possible combinations of castling rights.
    pub castling: [u64; 16],
    /// 8 possible en passant files, plus 1 for no EP target.
    pub ep_file: [u64; 9],
    /// Unique key added if it is Black's turn to move.
    pub side_to_move: u64,
}

/// Generates the static random keys using a seeded pseudo-random number generator.
///
/// A fixed seed is used to ensure the keys are deterministic across
/// program executions, which is required for the Transposition Table
/// to function correctly.
const fn next_state(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

const fn generate_keys() -> ZobristKeys {
    let mut state = 123456789u64;

    let mut pieces = [[0; 64]; 12];
    let mut i = 0;
    while i < 12 {
        let mut j = 0;
        while j < 64 {
            pieces[i][j] = next_state(&mut state);
            j += 1;
        }
        i += 1;
    }

    let mut castling = [0; 16];
    let mut i = 0;
    while i < 16 {
        castling[i] = next_state(&mut state);
        i += 1;
    }

    let mut ep_file = [0; 9];
    let mut i = 0;
    while i < 9 {
        ep_file[i] = next_state(&mut state);
        i += 1;
    }

    ZobristKeys {
        pieces,
        castling,
        ep_file,
        side_to_move: next_state(&mut state),
    }
}

/// Global, immutable Zobrist keys initialized at program startup.
pub static KEYS: ZobristKeys = generate_keys();
