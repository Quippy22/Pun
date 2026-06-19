use super::*;

impl MoveGenerator {
    #[inline(always)]
    pub(super) fn is_check(board: &Board, color: Color, target_bb: u64) -> bool {
        let square = target_bb.trailing_zeros() as usize;
        Self::is_square_attacked(board, square, color.opposite())
    }

    #[inline(always)]
    fn is_square_attacked(board: &Board, square: usize, by_color: Color) -> bool {
        let occupancy = board.colors[0] | board.colors[1];

        Self::pawn_attacks_square(board, square, by_color)
            || Self::knight_attacks_square(board, square, by_color)
            || Self::king_attacks_square(board, square, by_color)
            || Self::bishop_or_queen_attacks_square(board, square, by_color, occupancy)
            || Self::rook_or_queen_attacks_square(board, square, by_color, occupancy)
    }

    #[inline(always)]
    fn pawn_attacks_square(board: &Board, square: usize, by_color: Color) -> bool {
        let pawns = board.pieces[Piece::new(by_color, PieceType::Pawn) as usize];
        let file = square % 8;
        let rank = square / 8;

        match by_color {
            Color::White => {
                if rank == 0 {
                    return false;
                }

                (file > 0 && pawns & (1u64 << (square - 9)) != 0)
                    || (file < 7 && pawns & (1u64 << (square - 7)) != 0)
            }
            Color::Black => {
                if rank == 7 {
                    return false;
                }

                (file > 0 && pawns & (1u64 << (square + 7)) != 0)
                    || (file < 7 && pawns & (1u64 << (square + 9)) != 0)
            }
        }
    }

    #[inline(always)]
    fn knight_attacks_square(board: &Board, square: usize, by_color: Color) -> bool {
        let knights = board.pieces[Piece::new(by_color, PieceType::Knight) as usize];
        let file = square as i32 % 8;
        let rank = square as i32 / 8;

        for (df, dr) in [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ] {
            let new_file = file + df;
            let new_rank = rank + dr;
            if !(0..8).contains(&new_file) || !(0..8).contains(&new_rank) {
                continue;
            }

            let target = (new_rank * 8 + new_file) as usize;
            if knights & (1u64 << target) != 0 {
                return true;
            }
        }

        false
    }

    #[inline(always)]
    fn king_attacks_square(board: &Board, square: usize, by_color: Color) -> bool {
        let king = board.pieces[Piece::new(by_color, PieceType::King) as usize];
        let file = square as i32 % 8;
        let rank = square as i32 / 8;

        for dr in -1..=1 {
            for df in -1..=1 {
                if dr == 0 && df == 0 {
                    continue;
                }

                let new_file = file + df;
                let new_rank = rank + dr;
                if !(0..8).contains(&new_file) || !(0..8).contains(&new_rank) {
                    continue;
                }

                let target = (new_rank * 8 + new_file) as usize;
                if king & (1u64 << target) != 0 {
                    return true;
                }
            }
        }

        false
    }

    #[inline(always)]
    fn bishop_or_queen_attacks_square(
        board: &Board,
        square: usize,
        by_color: Color,
        occupancy: u64,
    ) -> bool {
        let attackers = board.pieces[Piece::new(by_color, PieceType::Bishop) as usize]
            | board.pieces[Piece::new(by_color, PieceType::Queen) as usize];

        Self::ray_attacks_square(
            square,
            occupancy,
            attackers,
            &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
        )
    }

    #[inline(always)]
    fn rook_or_queen_attacks_square(
        board: &Board,
        square: usize,
        by_color: Color,
        occupancy: u64,
    ) -> bool {
        let attackers = board.pieces[Piece::new(by_color, PieceType::Rook) as usize]
            | board.pieces[Piece::new(by_color, PieceType::Queen) as usize];

        Self::ray_attacks_square(
            square,
            occupancy,
            attackers,
            &[(1, 0), (-1, 0), (0, 1), (0, -1)],
        )
    }

    #[inline(always)]
    fn ray_attacks_square(
        square: usize,
        occupancy: u64,
        attackers: u64,
        directions: &[(i32, i32)],
    ) -> bool {
        let start_file = square as i32 % 8;
        let start_rank = square as i32 / 8;

        for (df, dr) in directions {
            let mut file = start_file;
            let mut rank = start_rank;

            loop {
                file += df;
                rank += dr;

                if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                    break;
                }

                let target = (rank * 8 + file) as usize;
                let bit = 1u64 << target;
                if attackers & bit != 0 {
                    return true;
                }
                if occupancy & bit != 0 {
                    break;
                }
            }
        }

        false
    }
}
