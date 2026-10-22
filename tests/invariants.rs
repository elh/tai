//! Deterministic generated hands exercise combinations of declarations and winning sources.
mod common;
use common::*;
use tai::*;

struct Random(u64);
impl Random {
    fn next(&mut self, max: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 32) as usize % max
    }
}

#[test]
fn generated_legal_hands_are_order_invariant_with_consistent_breakdowns() {
    let pool = tiles("123456789m 123456789p 123456789s 1234567z");
    let mut rng = Random(0x746169);
    let mut tested = 0;
    for _ in 0..2000 {
        let mut loose = Vec::new();
        let mut melds = Vec::new();
        for _ in 0..5 {
            let t = pool[rng.next(pool.len())];
            let suits: [fn(u8) -> Tile; 3] = [Tile::Characters, Tile::Dots, Tile::Bamboo];
            let suit = suits[rng.next(3)];
            let start = rng.next(7) as u8 + 1;
            match rng.next(6) {
                0 => loose.extend((start..start + 3).map(suit)),
                1 => loose.extend([t; 3]),
                2 => melds.push(Meld::Chow(suit(start))),
                3 => melds.push(Meld::Pung(t)),
                4 => melds.push(Meld::ConcealedKong(t)),
                _ => melds.push(Meld::Kong(t)),
            }
        }
        loose.extend([pool[rng.next(pool.len())]; 2]);
        let winning_tile = loose.remove(rng.next(loose.len()));
        let mut h = Hand {
            concealed: loose,
            melds,
            flowers: rng.next(8) as u8,
        };
        let mut c = base().1;
        c.winning_tile = winning_tile;
        c.source = [
            WinSource::SelfDraw,
            WinSource::Discard(Wind::West),
            WinSource::RobbedKong(Wind::North),
        ][rng.next(3)];
        c.last_tile = rng.next(2) == 0;
        let result = match score(&h, &c) {
            Ok(result) => result,
            Err(ScoreError::TooManyCopies { .. }) => continue,
            other => panic!("generated valid arrangement rejected: {other:?}; {h:?}; {c:?}"),
        };
        tested += 1;
        assert_eq!(result.total, result.awards.iter().map(|a| a.tai).sum());
        for (index, a) in result.awards.iter().enumerate() {
            assert!(a.tai > 0);
            assert!(
                result.awards[index + 1..]
                    .iter()
                    .all(|b| a.pattern != b.pattern)
            );
        }
        h.concealed.reverse();
        h.melds.reverse();
        assert_eq!(score(&h, &c).unwrap(), result);
    }
    assert!(tested > 500, "only {tested} valid generated hands");
}

#[test]
fn every_suit_permutation_preserves_the_score() {
    let (h, c) = hand("222234m 555p 777s 111z 99s", "2m", vec![]);
    let expected = score(&h, &c).unwrap();
    let suits: [fn(u8) -> Tile; 3] = [Tile::Characters, Tile::Bamboo, Tile::Dots];
    for permutation in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let map = |tile| match tile {
            Tile::Characters(rank) => suits[permutation[0]](rank),
            Tile::Bamboo(rank) => suits[permutation[1]](rank),
            Tile::Dots(rank) => suits[permutation[2]](rank),
            other => other,
        };
        let mut mapped_h = h.clone();
        mapped_h.concealed = h.concealed.iter().copied().map(map).collect();
        let mut mapped_c = c.clone();
        mapped_c.winning_tile = map(c.winning_tile);
        assert_eq!(score(&mapped_h, &mapped_c).unwrap(), expected);
    }
}
