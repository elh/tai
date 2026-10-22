use crate::{Hand, Meld, ScoreError, Tile};

/// Parse a hand using [`parse_tiles`]'s MPSZ notation with set and flower extensions.
/// Plain tiles stay concealed; `[123m]`, `[111z]`, and `[1111z]` declare an exposed
/// chow, pung, or kong. `{1111z}` declares a concealed kong. An upgraded pung uses
/// the exposed-kong form. Undeclared chows/pungs stay plain, as do loose quads.
/// Sets and loose tile groups may appear in any order, with optional whitespace.
/// Tile order within a set does not matter; brackets cannot nest.
/// One optional `0f`..`8f` group outside brackets gives the total flower/season
/// count, e.g. `3f` means three bonus tiles, not a tile identity. Omission means 0.
/// This count syntax is a custom extension, not standard MPSZ.
///
/// Exclude the winning tile; supply it and its source in [`crate::WinContext`].
/// Empty input returns an empty hand. [`crate::score`] validates whole-hand counts,
/// physical copy limits, and whether the hand wins.
///
/// # Errors
/// Returns [`ScoreError::InvalidNotation`] for invalid tiles, mismatched/nested
/// brackets, invalid sets, or repeated/out-of-range flower counts.
///
/// ```
/// use tai::{parse_hand, Meld, Tile};
/// let hand = parse_hand("[111s] {5555z} [666z] 77s 56m 777z 3f")?;
/// assert_eq!(hand.concealed.len(), 7);
/// assert_eq!(hand.melds[1], Meld::ConcealedKong(Tile::White));
/// assert_eq!(hand.flowers, 3);
/// # Ok::<(), tai::ScoreError>(())
/// ```
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn parse_hand(mut notation: &str) -> Result<Hand, ScoreError> {
    let mut hand = Hand::default();
    let mut flowers = None;
    while let Some(start) = notation.find(['[', '{', ']', '}']) {
        hand.concealed
            .extend(parse_groups(&notation[..start], &mut flowers)?);
        let close = match notation.as_bytes()[start] {
            b'[' => ']',
            b'{' => '}',
            _ => return Err(ScoreError::InvalidNotation),
        };
        notation = &notation[start + 1..];
        let end = notation.find(close).ok_or(ScoreError::InvalidNotation)?;
        hand.melds.push(parse_meld(&notation[..end], close == '}')?);
        notation = &notation[end + 1..];
    }
    hand.concealed.extend(parse_groups(notation, &mut flowers)?);
    hand.flowers = flowers.unwrap_or(0);
    Ok(hand)
}

fn parse_meld(notation: &str, concealed: bool) -> Result<Meld, ScoreError> {
    let mut tiles = parse_tiles(notation)?
        .into_iter()
        .map(Tile::index)
        .collect::<Result<Vec<_>, _>>()?;
    tiles.sort_unstable();
    match tiles.as_slice() {
        [a, b, c, d] if a == b && b == c && c == d => {
            let tile = Tile::from_index(*a);
            Ok(if concealed {
                Meld::ConcealedKong(tile)
            } else {
                Meld::Kong(tile)
            })
        }
        [a, b, c] if !concealed && a == b && b == c => Ok(Meld::Pung(Tile::from_index(*a))),
        [a, b, c] if !concealed && *a < 27 && a / 9 == c / 9 && *b == a + 1 && *c == a + 2 => {
            Ok(Meld::Chow(Tile::from_index(*a)))
        }
        _ => Err(ScoreError::InvalidNotation),
    }
}

/// Parse exactly one tile using [`parse_tiles`]'s MPSZ notation, e.g. `3m` or `1z`.
/// Surrounding whitespace is allowed.
///
/// # Errors
/// Returns [`ScoreError::InvalidNotation`] for invalid notation, empty input, or multiple tiles.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn parse_tile(notation: &str) -> Result<Tile, ScoreError> {
    match parse_tiles(notation)?.as_slice() {
        [tile] => Ok(*tile),
        _ => Err(ScoreError::InvalidNotation),
    }
}

/// Parse MPSZ tile groups: `m` characters, `p` dots, `s` bamboo, `z` honors.
/// Honors 1..=7 are East, South, West, North, White, Green, Red. Groups may touch
/// or be separated by whitespace. Empty input returns an empty list.
/// Use [`parse_hand`] for declared sets and flower counts. Red fives are unsupported.
///
/// This parses tiles only; [`crate::score`] checks hand size and physical copy limits.
///
/// # Errors
/// Returns [`ScoreError::InvalidNotation`] for missing suits, invalid ranks, or other text.
///
/// ```
/// use tai::{parse_tiles, Tile::*};
/// assert_eq!(parse_tiles("123m 11z")?, vec![Characters(1), Characters(2), Characters(3), East, East]);
/// # Ok::<(), tai::ScoreError>(())
/// ```
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn parse_tiles(notation: &str) -> Result<Vec<Tile>, ScoreError> {
    let mut flowers = None;
    let tiles = parse_groups(notation, &mut flowers)?;
    if flowers.is_some() {
        return Err(ScoreError::InvalidNotation);
    }
    Ok(tiles)
}

fn parse_groups(notation: &str, flowers: &mut Option<u8>) -> Result<Vec<Tile>, ScoreError> {
    let mut tiles = Vec::new();
    for group in notation.split_whitespace() {
        let mut start = 0;
        for (index, suit) in group.char_indices() {
            if suit.is_ascii_digit() {
                continue;
            }
            if index == start {
                return Err(ScoreError::InvalidNotation);
            }
            let ranks = &group[start..index];
            if suit == 'f' {
                if ranks.len() != 1 || ranks.as_bytes()[0] > b'8' || flowers.is_some() {
                    return Err(ScoreError::InvalidNotation);
                }
                *flowers = Some(ranks.as_bytes()[0] - b'0');
                start = index + 1;
                continue;
            }
            for rank in ranks.bytes().map(|b| b - b'0') {
                if rank == 0 {
                    return Err(ScoreError::InvalidNotation);
                }
                tiles.push(match suit {
                    'm' => Tile::Characters(rank),
                    'p' => Tile::Dots(rank),
                    's' => Tile::Bamboo(rank),
                    'z' if rank <= 7 => [
                        Tile::East,
                        Tile::South,
                        Tile::West,
                        Tile::North,
                        Tile::White,
                        Tile::Green,
                        Tile::Red,
                    ][usize::from(rank - 1)],
                    _ => return Err(ScoreError::InvalidNotation),
                });
            }
            start = index + 1;
        }
        if start != group.len() {
            return Err(ScoreError::InvalidNotation);
        }
    }
    Ok(tiles)
}
