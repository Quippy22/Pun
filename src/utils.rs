pub fn string_to_square(s: &str) -> u8 {
    let bytes = s.as_bytes();

    // File: 'a' is 0, 'b' is 1... 'h' is 7
    let file = bytes[0] - b'a';

    // Rank: '1' is 0, '2' is 1... '8' is 7
    let rank = bytes[1] - b'1';

    (rank * 8) + file
}
