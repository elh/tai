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
        ("5z", White),
        ("6z", Green),
        ("\t7z\n\u{2003}", Red),
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
        .chain([East, South, West, North, White, Green, Red])
        .collect();
    assert_eq!(
        parse_tiles("123456789m 123456789p 123456789s 1234567z").unwrap(),
        expected
    );
}

#[test]
fn groups_preserve_order_repetition_and_allow_whitespace() {
    let expected = vec![
        Dots(9),
        Dots(1),
        White,
        East,
        East,
        Bamboo(2),
        Characters(3),
    ];
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
        assert_eq!(
            parse_hand(text),
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

    h = parse_hand("[1111m] 23m 234p 678p 345s 99s").unwrap();
    c.winning_tile = Characters(1);
    assert_eq!(
        score(&h, &c),
        Err(ScoreError::TooManyCopies {
            tile: Characters(1)
        })
    );
    h = parse_hand("[111m][222m][333m][444m][555m][666m]").unwrap();
    assert_eq!(score(&h, &c), Err(ScoreError::TooManyMelds));
}

#[test]
fn complete_public_example_and_repeated_borrowed_calls() {
    let hand = parse_hand("[111s] {5555z} [666z] 77s 56m 777z 3f").unwrap();
    assert_eq!(
        hand,
        Hand {
            concealed: vec![
                Bamboo(7),
                Bamboo(7),
                Characters(5),
                Characters(6),
                Red,
                Red,
                Red
            ],
            melds: vec![
                Meld::Pung(Bamboo(1)),
                Meld::ConcealedKong(White),
                Meld::Pung(Green)
            ],
            flowers: 3,
        }
    );
    let win = WinContext {
        winning_tile: parse_tile("4m").unwrap(),
        winner: Wind::South,
        dealer: Wind::East,
        source: WinSource::Discard(Wind::West),
        discards: 10,
        last_tile: false,
        ready_on_original_hand: false,
    };
    let first = score(&hand, &win).unwrap();
    assert_score(
        &first,
        42,
        &[
            (Pattern::Winning, 2),
            (Pattern::Flowers, 3),
            (Pattern::HonorPungs, 3),
            (Pattern::ConcealedKongs, 2),
            (Pattern::TwoConcealedTriplets, 2),
            (Pattern::BigThreeDragons, 30),
        ],
    );
    assert_eq!(score(&hand, &win).unwrap(), first);
    let exposed = parse_hand("[111s] [5555z] [666z] 77s 56m 777z").unwrap();
    assert_eq!(score(&exposed, &win).unwrap().total, 37);
}

#[test]
fn mixed_sets_allow_interleaving_whitespace_and_unordered_tiles() {
    let expected = Hand {
        concealed: vec![Bamboo(7), Bamboo(7), Red, Red],
        melds: vec![
            Meld::Chow(Characters(1)),
            Meld::Pung(Bamboo(1)),
            Meld::Kong(White),
            Meld::ConcealedKong(Green),
        ],
        flowers: 0,
    };
    for text in [
        "[123m] [111s] [5555z] {6666z} 77s 77z",
        "7s[3m1m2m]7s[1s 11s]7z[5z 555z]7z{\n66z\u{2003}66z}",
    ] {
        assert_eq!(parse_hand(text).unwrap(), expected, "{text:?}");
    }
    let mut win = base().1;
    win.winning_tile = Red;
    win.source = WinSource::SelfDraw;
    let mut hand = parse_hand("[123m] [111s] [5555z] {6666z} 77s 77z").unwrap();
    hand.flowers = 1;
    let scored = score(&hand, &win).unwrap();
    assert_award(&scored, Pattern::Flowers, 1);
    assert_award(&scored, Pattern::MeldedKongs, 1);
    assert_award(&scored, Pattern::ConcealedKongs, 2);
    for (text, meld) in [
        ("[789p]", Meld::Chow(Dots(7))),
        ("[321s]", Meld::Chow(Bamboo(1))),
    ] {
        assert_eq!(parse_hand(text).unwrap().melds, vec![meld]);
    }
}

#[test]
fn plain_hands_do_not_infer_declared_sets() {
    for text in ["", " \t\n", "1111m", "123456m 234678p 345s 1z"] {
        assert_eq!(
            parse_hand(text).unwrap(),
            Hand::from(parse_tiles(text).unwrap())
        );
    }
    for text in ["[111s]", "{5555z}"] {
        assert_eq!(parse_tiles(text), Err(ScoreError::InvalidNotation));
    }
}

#[test]
fn flower_counts_are_optional_and_can_appear_between_groups() {
    let plain = "[123m] 77s {5555z}";
    let mut expected = parse_hand(plain).unwrap();
    assert_eq!(expected.flowers, 0);
    for count in [0, 3, 8] {
        expected.flowers = count;
        for text in [
            format!("{count}f {plain}"),
            format!("[123m]{count}f77s{{5555z}}"),
            format!("{plain}\n{count}f\t"),
        ] {
            assert_eq!(parse_hand(&text).unwrap(), expected, "{text:?}");
        }
        let text = format!("{count}f");
        assert_eq!(
            parse_hand(&text).unwrap(),
            Hand {
                flowers: count,
                ..Hand::default()
            }
        );
        assert_eq!(parse_tiles(&text), Err(ScoreError::InvalidNotation));
        assert_eq!(parse_tile(&text), Err(ScoreError::InvalidNotation));
    }
}

#[test]
fn invalid_flower_groups_are_rejected() {
    for text in [
        "f",
        "9f",
        "12f",
        "03f",
        "255f",
        "3 f",
        "3F",
        "-1f",
        "３f",
        "1f2f",
        "0f 0f",
        "0f[111s]3f",
        "3f{5555z}0f",
        "[3f]",
        "{4f}",
        "[111s0f]",
        "{5555z1f}",
        "3f1",
        "3f0m",
    ] {
        assert_eq!(
            parse_hand(text),
            Err(ScoreError::InvalidNotation),
            "{text:?}"
        );
    }
    for text in ["1m0f", "1f1m"] {
        assert_eq!(parse_tiles(text), Err(ScoreError::InvalidNotation));
        assert_eq!(parse_tile(text), Err(ScoreError::InvalidNotation));
    }
}

#[test]
fn malformed_brackets_and_invalid_sets_are_rejected() {
    for text in [
        "[]",
        "{}",
        "[123m",
        "{1111z",
        "123m]",
        "1111z}",
        "[123m}",
        "{1111z]",
        "[[111s]]",
        "[{1111s}]",
        "{[1111s]}",
        "[11z]",
        "[11111z]",
        "[123456m]",
        "[112m]",
        "[124m]",
        "[123z]",
        "[1m1p1s]",
        "[89m1s]",
        "[89s1p]",
        "[89p1z]",
        "{111z}",
        "{1234m}",
        "[8z8z8z]",
        "[🀄]",
        "1[111s]m",
        "[111s]m",
        "[111s] + 1z",
        "[111s][222p]]",
    ] {
        assert_eq!(
            parse_hand(text),
            Err(ScoreError::InvalidNotation),
            "{text:?}"
        );
    }
}

#[test]
fn winning_tile_source_controls_the_completed_pungs_concealment() {
    let hand = parse_hand("[123m] 111p 222s 33z 444z 55s").unwrap();
    let mut win = base().1;
    win.winning_tile = West;
    assert_award(
        &score(&hand, &win).unwrap(),
        Pattern::ThreeConcealedTriplets,
        5,
    );
    win.source = WinSource::SelfDraw;
    assert_award(
        &score(&hand, &win).unwrap(),
        Pattern::FourConcealedTriplets,
        15,
    );
}

#[test]
fn five_declared_sets_can_be_fully_exposed_or_concealed_kongs() {
    let mut win = base().1;
    win.winning_tile = Bamboo(1);
    let exposed = parse_hand("[111m] [444m] [222p] [666p] [555z] 1s").unwrap();
    assert_award(&score(&exposed, &win).unwrap(), Pattern::ExposedHand, 10);
    let concealed = parse_hand("{1111m} {4444m} {2222p} {6666p} {5555z} 1s").unwrap();
    let scored = score(&concealed, &win).unwrap();
    assert_award(&scored, Pattern::ConcealedKongs, 10);
    assert_award(&scored, Pattern::ConcealedHand, 1);
    assert_award(&scored, Pattern::FiveConcealedTriplets, 40);
}
