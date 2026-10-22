//! Public notation helper: exact mapping, strict syntax, and composition with scoring.
mod common;
use common::*;
use tai::{Tile::*, *};

#[test]
fn single_tile_uses_the_same_notation_and_allows_surrounding_whitespace() {
    for (text, expected) in [
        ("1m", Characters(1)),
        ("9p", Dots(9)),
        ("4s", Bamboo(4)),
        ("1z", East),
        ("\t7z\n\u{2003}", White),
    ] {
        assert_eq!(parse_tile(text), Ok(expected), "{text:?}");
    }
}

#[test]
fn single_tile_rejects_empty_input_and_multiple_tiles() {
    for text in ["", " \t\n", "11z", "1m2p", "1m 1m"] {
        assert_eq!(
            parse_tile(text),
            Err(ScoreError::InvalidNotation),
            "{text:?}"
        );
    }
}

#[test]
fn all_thirty_four_tile_names_parse_exactly() {
    let expected: Vec<_> = (1..=9)
        .map(Characters)
        .chain((1..=9).map(Dots))
        .chain((1..=9).map(Bamboo))
        .chain([East, South, West, North, Red, Green, White])
        .collect();
    assert_eq!(
        parse_tiles("123456789m 123456789p 123456789s 1234567z").unwrap(),
        expected
    );
}

#[test]
fn groups_preserve_order_repetition_and_allow_whitespace() {
    let expected = vec![Dots(9), Dots(1), Red, East, East, Bamboo(2), Characters(3)];
    for text in [
        "91p511z2s3m",
        "91p 511z 2s 3m",
        "\t91p\n511z\r\n2s\u{2003}3m  ",
    ] {
        assert_eq!(parse_tiles(text).unwrap(), expected);
    }
    assert_eq!(parse_tiles("").unwrap(), vec![]);
    assert_eq!(parse_tiles(" \t\n").unwrap(), vec![]);
}

#[test]
fn malformed_notation_returns_a_typed_error() {
    for text in [
        "m", "1", "123m4", "0m", "10m", "8z", "9z", "12x", "123M", "1 m", "1 2m", "123mm", "1m,2p",
        "-1m", "１m", "1🀄", "🀄", "123m\0",
    ] {
        assert_eq!(
            parse_tiles(text),
            Err(ScoreError::InvalidNotation),
            "{text:?}"
        );
        assert_eq!(
            parse_tile(text),
            Err(ScoreError::InvalidNotation),
            "{text:?}"
        );
    }
}

#[test]
fn notation_leaves_physical_hand_validation_to_score() {
    let mut h = Hand::from(parse_tiles("11111m 234p 678p 345s 99s").unwrap());
    let mut c = base().1;
    c.winning_tile = Bamboo(9);
    assert_eq!(
        score(&h, &c),
        Err(ScoreError::TooManyCopies {
            tile: Characters(1)
        })
    );
    h.concealed = parse_tiles("123m").unwrap();
    assert_eq!(score(&h, &c), Err(ScoreError::WrongTileCount));
}

#[test]
fn complete_public_example_and_repeated_borrowed_calls() {
    let hand = Hand::from(parse_tiles("123456m 234678p 345s 1z").unwrap());
    let win = WinContext {
        winning_tile: parse_tile("1z").unwrap(),
        winner: Wind::South,
        dealer: Wind::East,
        source: WinSource::Discard(Wind::West),
        discards: 10,
        last_tile: false,
        ready_on_original_hand: false,
    };
    let first = score(&hand, &win).unwrap();
    assert_eq!(first.total, 8);
    assert_eq!(score(&hand, &win).unwrap(), first);
    assert!(hand.melds.is_empty() && hand.flowers.is_empty());
    assert_eq!(hand.concealed.len(), 16);
}
