//! Internal representation and exhaustive decomposition of at most 17 loose tiles.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Chow,
    Pung,
    Kong,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Set {
    pub tile: usize,
    pub kind: Kind,
    pub concealed: bool,
}

impl Set {
    pub fn contains(self, tile: usize) -> bool {
        if self.kind == Kind::Chow {
            (self.tile..=self.tile + 2).contains(&tile)
        } else {
            self.tile == tile
        }
    }
}

pub(crate) struct Arrangement {
    pub pair: usize,
    pub sets: Vec<Set>,
}

pub(crate) fn arrangements(counts: &[u8; 34]) -> Vec<Arrangement> {
    let mut results = Vec::new();
    let mut counts = *counts;
    for pair in 0..34 {
        if counts[pair] >= 2 {
            counts[pair] -= 2;
            sets(&mut counts, &mut Vec::new(), pair, &mut results);
            counts[pair] += 2;
        }
    }
    results
}

fn sets(counts: &mut [u8; 34], path: &mut Vec<Set>, pair: usize, out: &mut Vec<Arrangement>) {
    let Some(tile) = counts.iter().position(|&n| n > 0) else {
        out.push(Arrangement {
            pair,
            sets: path.clone(),
        });
        return;
    };
    if counts[tile] >= 3 {
        counts[tile] -= 3;
        path.push(Set {
            tile,
            kind: Kind::Pung,
            concealed: true,
        });
        sets(counts, path, pair, out);
        path.pop();
        counts[tile] += 3;
    }
    if tile < 27 && tile % 9 <= 6 && counts[tile + 1] > 0 && counts[tile + 2] > 0 {
        for count in &mut counts[tile..tile + 3] {
            *count -= 1;
        }
        path.push(Set {
            tile,
            kind: Kind::Chow,
            concealed: true,
        });
        sets(counts, path, pair, out);
        path.pop();
        for count in &mut counts[tile..tile + 3] {
            *count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn exhaustive_six_rank_multisets_match_an_independent_shape_generator() {
        // Construct the reference from legal groups, independently of tile-first search.
        fn generate(counts: [u8; 6], depth: usize, start: usize, out: &mut HashSet<[u8; 6]>) {
            for pair in 0..6 {
                let mut hand = counts;
                hand[pair] += 2;
                if hand[pair] <= 4 {
                    out.insert(hand);
                }
            }
            if depth == 5 {
                return;
            }
            for set in start..10 {
                let mut hand = counts;
                if set < 6 {
                    hand[set] += 3;
                } else {
                    for count in &mut hand[set - 6..set - 3] {
                        *count += 1;
                    }
                }
                if hand.iter().all(|&count| count <= 4) {
                    generate(hand, depth + 1, set, out);
                }
            }
        }
        let mut expected = HashSet::new();
        generate([0; 6], 0, 0, &mut expected);
        for encoded in 0..5_u32.pow(6) {
            let mut value = encoded;
            let mut counts = [0; 34];
            for count in &mut counts[..6] {
                *count = (value % 5) as u8;
                value /= 5;
            }
            let total: u8 = counts.iter().sum();
            if total > 17 || total % 3 != 2 {
                continue;
            }
            let actual = arrangements(&counts);
            let key: [u8; 6] = counts[..6].try_into().unwrap();
            assert_eq!(!actual.is_empty(), expected.contains(&key), "{key:?}");
            for arrangement in actual {
                let mut rebuilt = [0; 34];
                rebuilt[arrangement.pair] += 2;
                for set in arrangement.sets {
                    if set.kind == Kind::Chow {
                        for count in &mut rebuilt[set.tile..set.tile + 3] {
                            *count += 1;
                        }
                    } else {
                        rebuilt[set.tile] += 3;
                    }
                }
                assert_eq!(rebuilt, counts);
            }
        }
    }
}
