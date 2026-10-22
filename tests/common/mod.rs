#![allow(dead_code)]
use tai::*;

pub fn tiles(notation: &str) -> Vec<Tile> {
    parse_tiles(notation).expect("valid test tile notation")
}

pub fn tile(notation: &str) -> Tile {
    tiles(notation)[0]
}

pub fn hand(complete_loose: &str, winning: &str, melds: Vec<Meld>) -> (Hand, WinContext) {
    let winning_tile = tile(winning);
    let mut concealed = tiles(complete_loose);
    let index = concealed
        .iter()
        .position(|&tile| tile == winning_tile)
        .unwrap();
    concealed.remove(index);
    (
        Hand {
            concealed,
            melds,
            flowers: vec![],
        },
        WinContext {
            winning_tile,
            winner: Wind::South,
            dealer: Wind::East,
            source: WinSource::Discard(Wind::West),
            discards: 10,
            last_tile: false,
            ready_on_original_hand: false,
        },
    )
}

pub fn base() -> (Hand, WinContext) {
    hand("123m 456m 234p 678p 345s 11z", "1z", vec![])
}

pub fn pung(value: &str) -> Meld {
    Meld::Pung(tile(value))
}
pub fn kong(value: &str, concealed: bool) -> Meld {
    if concealed {
        Meld::ConcealedKong(tile(value))
    } else {
        Meld::Kong(tile(value))
    }
}

pub fn award(score: &Score, pattern: Pattern) -> u16 {
    score
        .awards
        .iter()
        .filter(|a| a.pattern == pattern)
        .map(|a| a.tai)
        .sum()
}

#[track_caller]
pub fn assert_score(score: &Score, total: u16, awards: &[(Pattern, u16)]) {
    assert_eq!(score.total, total, "{score:?}");
    let actual: Vec<_> = score.awards.iter().map(|a| (a.pattern, a.tai)).collect();
    assert_eq!(actual, awards);
}

#[track_caller]
pub fn assert_award(score: &Score, pattern: Pattern, tai: u16) {
    assert_eq!(award(score, pattern), tai, "{pattern:?} in {score:?}");
}

pub fn flowers() -> Vec<Flower> {
    vec![
        Flower::Plum,
        Flower::Orchid,
        Flower::Chrysanthemum,
        Flower::Bamboo,
        Flower::Spring,
        Flower::Summer,
        Flower::Autumn,
        Flower::Winter,
    ]
}
