use crate::{ScoreError, Tile};

/// Parse exactly one tile using [`parse_tiles`]'s MPSZ-style notation, e.g. `3m` or `1z`.
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

/// Parse MPSZ-style tile groups: `m` characters, `p` dots, `s` bamboo, `z` honors.
/// Honors 1..=7 are East, South, West, North, Red, Green, White; this dragon order
/// differs from standard MPSZ (White, Green, Red). Groups may touch or be separated
/// by whitespace. Empty input returns an empty list.
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
    let mut tiles = Vec::new();
    for group in notation.split_whitespace() {
        let mut start = 0;
        for (index, suit) in group.char_indices() {
            if ('1'..='9').contains(&suit) {
                continue;
            }
            if index == start {
                return Err(ScoreError::InvalidNotation);
            }
            for rank in group[start..index].bytes().map(|b| b - b'0') {
                tiles.push(match suit {
                    'm' => Tile::Characters(rank),
                    'p' => Tile::Dots(rank),
                    's' => Tile::Bamboo(rank),
                    'z' if rank <= 7 => Tile::from_index(26 + usize::from(rank)),
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
