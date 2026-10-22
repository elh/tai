//! Invalid hands and contradictory scoring context must return typed errors.
mod common;
use common::*;
use tai::*;

#[test]
fn chows_require_numbered_tiles_and_stay_inside_one_suit() {
    for first in tiles("1234567z") {
        let (h, c) = hand("123m 234p 678p 345s 11z", "1z", vec![Meld::Chow(first)]);
        assert_eq!(score(&h, &c), Err(ScoreError::InvalidChow));
    }
    let suits: [fn(u8) -> Tile; 3] = [Tile::Characters, Tile::Bamboo, Tile::Dots];
    for suit in suits {
        for start in [0, 8, 9, 255] {
            let (h, c) = hand(
                "123m 234p 678p 345s 11z",
                "1z",
                vec![Meld::Chow(suit(start))],
            );
            assert_eq!(score(&h, &c), Err(ScoreError::InvalidChow));
        }
        for start in 1..=7 {
            let (h, c) = hand(
                "111z 222z 333z 555z 66z",
                "6z",
                vec![Meld::Chow(suit(start))],
            );
            assert!(
                score(&h, &c).is_ok(),
                "valid chow rejected: {suit:?} {start}"
            );
        }
    }
}

#[test]
fn every_numbered_tile_location_rejects_invalid_ranks() {
    let suits: [fn(u8) -> Tile; 3] = [Tile::Characters, Tile::Bamboo, Tile::Dots];
    for suit in suits {
        for rank in [0, 10, 255] {
            for meld in [
                Meld::Pung(suit(rank)),
                Meld::Kong(suit(rank)),
                Meld::ConcealedKong(suit(rank)),
            ] {
                let (h, c) = hand("123m 234p 678p 345s 11z", "1z", vec![meld]);
                assert_eq!(score(&h, &c), Err(ScoreError::InvalidRank));
            }
            let (mut h, mut c) = base();
            c.winning_tile = suit(rank);
            assert_eq!(score(&h, &c), Err(ScoreError::InvalidRank));
            c.winning_tile = Tile::East;
            h.concealed[0] = suit(rank);
            assert_eq!(score(&h, &c), Err(ScoreError::InvalidRank));
        }
    }
}

#[test]
fn flower_only_wins_never_require_dummy_tile_data() {
    let (_, c) = base();
    let h = Hand {
        flowers: flowers(),
        ..Hand::default()
    };
    assert_eq!(score(&h, &c), Err(ScoreError::FlowerWinRequired));
    assert_eq!(score_flowers(&h.flowers, None).unwrap().total, 30);
}

#[test]
fn wrong_tile_and_meld_counts() {
    let (h, c) = base();
    for length in [0, 1, 13, 15, 17, 1000] {
        let mut bad = h.clone();
        bad.concealed.resize(length, tile("1m"));
        assert_eq!(score(&bad, &c), Err(ScoreError::WrongTileCount));
    }
    let mut bad = h;
    bad.melds = vec![pung("1m"); 6];
    assert_eq!(score(&bad, &c), Err(ScoreError::TooManyMelds));
}

#[test]
fn copy_limit_includes_winning_tile_and_all_melds() {
    let (h, c) = hand("11111m 234p 678p 345s 999s", "1m", vec![]);
    assert_eq!(
        score(&h, &c),
        Err(ScoreError::TooManyCopies { tile: tile("1m") })
    );
    let (h, c) = hand("123m 234p 678p 345s 11z", "1z", vec![kong("1m", true)]);
    assert_eq!(
        score(&h, &c),
        Err(ScoreError::TooManyCopies { tile: tile("1m") })
    );
    let (h, c) = hand("123m 234p 678p 11z", "1z", vec![pung("5z"), pung("5z")]);
    assert_eq!(
        score(&h, &c),
        Err(ScoreError::TooManyCopies { tile: tile("5z") })
    );
}

#[test]
fn robbed_kong_reserves_three_copies_in_the_opponents_pung() {
    for (notation, winning, melds) in [
        ("123m 456m 234p 678p 345s 11z", "1z", vec![]),
        ("111m 456m 234p 678p 345s 11z", "1m", vec![]),
        ("123m 234p 678p 345s 11z", "1m", vec![pung("1m")]),
    ] {
        let (h, mut c) = hand(notation, winning, melds);
        assert!(score(&h, &c).is_ok());
        c.source = WinSource::RobbedKong(Wind::West);
        assert_eq!(
            score(&h, &c),
            Err(ScoreError::TooManyCopies {
                tile: c.winning_tile
            })
        );
    }
}

#[test]
fn robbing_an_upgraded_pung_requires_a_prior_discard() {
    let (h, mut c) = hand("123m 456m 234p 678p 345s 11z", "3m", vec![]);
    c.source = WinSource::RobbedKong(Wind::West);
    assert!(score(&h, &c).is_ok());
    c.discards = 0;
    assert!(matches!(
        score(&h, &c),
        Err(ScoreError::InvalidContext { .. })
    ));
}

#[test]
fn not_every_seventeen_tiles_win_and_suits_do_not_wrap() {
    for (notation, winning) in [
        ("123m 456m 234p 678p 345s 12z", "1z"),
        ("89m 1p 234p 678p 345s 678s 11z", "1z"),
        ("123z 456m 234p 678p 345s 11m", "1m"),
    ] {
        let (h, c) = hand(notation, winning, vec![]);
        assert_eq!(score(&h, &c), Err(ScoreError::NotWinning));
    }
}

#[test]
fn invalid_flowers_are_rejected() {
    let (mut h, c) = base();
    h.flowers = vec![Flower::Plum; 2];
    let duplicate = Err(ScoreError::DuplicateFlower {
        flower: Flower::Plum,
    });
    assert_eq!(score(&h, &c), duplicate);
    assert_eq!(score_flowers(&[Flower::Plum; 8], None), duplicate);
    assert_eq!(
        score_flowers(&[Flower::Plum; 9], None),
        Err(ScoreError::InvalidFlowers)
    );
    for count in 0..8 {
        assert_eq!(
            score_flowers(&flowers()[..count], None),
            Err(ScoreError::InvalidFlowers)
        );
    }
    assert_eq!(
        score_flowers(&flowers()[..7], Some(Flower::Plum)),
        Err(ScoreError::InvalidFlowers)
    );
    assert_eq!(
        score_flowers(&flowers()[..6], Some(Flower::Winter)),
        Err(ScoreError::InvalidFlowers)
    );
    assert_eq!(
        score_flowers(&flowers(), Some(Flower::Winter)),
        Err(ScoreError::InvalidFlowers)
    );
}

#[test]
fn impossible_context_is_rejected() {
    let (h, c) = base();
    for (source, discards, last_tile) in [
        (WinSource::Discard(c.winner), 10, false),
        (WinSource::RobbedKong(c.winner), 10, false),
        (WinSource::Discard(Wind::West), 0, false),
        (WinSource::SelfDraw, 0, false),
        (WinSource::Discard(c.dealer), 1, true),
    ] {
        let context = WinContext {
            source,
            discards,
            last_tile,
            ..c.clone()
        };
        assert!(matches!(
            score(&h, &context),
            Err(ScoreError::InvalidContext { .. })
        ));
    }
}

#[test]
fn first_discard_must_come_from_the_dealer() {
    let (h, mut c) = base();
    for (dealer, from, discards, valid) in [
        (Wind::East, Wind::West, 1, false),
        (Wind::North, Wind::East, 1, false),
        (Wind::North, Wind::North, 1, true),
        (Wind::North, Wind::East, 2, true),
    ] {
        c.dealer = dealer;
        c.source = WinSource::Discard(from);
        c.discards = discards;
        let result = score(&h, &c);
        if valid {
            assert!(result.is_ok(), "{c:?}: {result:?}");
        } else {
            assert!(
                matches!(result, Err(ScoreError::InvalidContext { .. })),
                "{c:?}: {result:?}"
            );
        }
    }
}

#[test]
fn opening_wins_cannot_have_prior_calls() {
    let (h, mut c) = hand("123m 234p 678p 345s 11z", "1z", vec![pung("5z")]);
    c.discards = 1;
    c.source = WinSource::Discard(c.dealer);
    assert!(matches!(
        score(&h, &c),
        Err(ScoreError::InvalidContext { .. })
    ));
    c.discards = 0;
    c.winner = c.dealer;
    c.source = WinSource::SelfDraw;
    assert!(matches!(
        score(&h, &c),
        Err(ScoreError::InvalidContext { .. })
    ));
}

#[test]
fn opening_dealer_can_have_concealed_kong_supplements() {
    let (h, mut c) = hand("123m 234p 678p 345s 11z", "1z", vec![kong("5z", true)]);
    c.discards = 0;
    c.winner = c.dealer;
    c.source = WinSource::SelfDraw;
    assert_award(&score(&h, &c).unwrap(), Pattern::HeavenlyHand, 40);
}

#[test]
fn original_hand_ready_cannot_have_declared_sets_or_be_heavenly() {
    let (h, mut c) = hand("123m 234p 678p 345s 11z", "1z", vec![kong("5z", true)]);
    c.ready_on_original_hand = true;
    assert!(matches!(
        score(&h, &c),
        Err(ScoreError::InvalidContext { .. })
    ));
    let (h, mut c) = base();
    c.winner = c.dealer;
    c.source = WinSource::SelfDraw;
    c.discards = 0;
    c.ready_on_original_hand = true;
    assert!(matches!(
        score(&h, &c),
        Err(ScoreError::InvalidContext { .. })
    ));
}
