//! Rule values, overlap conventions, and winning-tile placement regressions.
mod common;
use common::*;
use tai::{Pattern::*, *};

#[test]
fn simple_hand_has_exact_breakdown() {
    let (h, c) = base();
    let result = score(&h, &c).unwrap();
    assert_score(
        &result,
        8,
        &[
            (Winning, 2),
            (NoFlowers, 1),
            (ChowHand, 3),
            (ConcealedHand, 1),
            (PairWait, 1),
        ],
    );
    assert_eq!(
        result
            .awards
            .iter()
            .map(|a| a.description.as_str())
            .collect::<Vec<_>>(),
        [
            "Winning (base)",
            "No flowers or seasons",
            "Chow hand",
            "Concealed hand (claimed winning tile)",
            "Out on a pair"
        ],
    );
}

#[test]
fn reference_hand_with_mandatory_flower_scoring() {
    // Four declared sets and one concealed pung; two honor sets, one exposed kong.
    // The reference's 16-tai subtotal plus the no-flowers bonus, now always enabled.
    let (h, c) = hand(
        "777p 11z",
        "1z",
        vec![pung("2z"), kong("5z", false), pung("2m"), pung("6s")],
    );
    assert_score(
        &score(&h, &c).unwrap(),
        17,
        &[
            (Winning, 2),
            (NoFlowers, 1),
            (HonorPungs, 2),
            (MeldedKongs, 1),
            (PairWait, 1),
            (PungHand, 10),
        ],
    );
}

#[test]
fn flowers_are_always_enabled() {
    let (mut h, c) = base();
    for count in [0, 1, 7] {
        h.flowers = flowers()[..count].to_vec();
        let result = score(&h, &c).unwrap();
        assert_award(&result, Flowers, count as u16);
        assert_award(&result, NoFlowers, u16::from(count == 0));
    }
}

#[test]
fn flower_wins_are_exclusive() {
    assert_score(
        &score_flowers(&flowers(), None).unwrap(),
        30,
        &[(EightFlowers, 30)],
    );
    assert_score(
        &score_flowers(&flowers()[..7], Some(Flower::Winter)).unwrap(),
        20,
        &[(SevenFlowersRobbingEighth, 20)],
    );
    let (mut h, c) = base();
    h.flowers = flowers();
    assert_eq!(score(&h, &c), Err(ScoreError::FlowerWinRequired));
}

#[test]
fn absence_and_chow_bonuses_do_not_double_count() {
    let (mut h, c) = hand("123m 456m 234p 678p 345s 99s", "9s", vec![]);
    h.flowers = vec![Flower::Plum];
    let held = score(&h, &c).unwrap();
    assert_award(&held, NoHonors, 1);
    assert_award(&held, ChowHand, 3);
    h.flowers = vec![];
    let clean = score(&h, &c).unwrap();
    assert_award(&clean, ChowHandNoHonorsOrFlowers, 10);
    for p in [NoHonors, NoFlowers, NoHonorsOrFlowers, ChowHand] {
        assert_award(&clean, p, 0);
    }

    let (mut h, c) = hand("111m 456m 234p 678p 345s 99s", "9s", vec![]);
    h.flowers = vec![];
    let combined = score(&h, &c).unwrap();
    assert_award(&combined, NoHonorsOrFlowers, 3);
    assert_award(&combined, NoHonors, 0);
    assert_award(&combined, NoFlowers, 0);
}

#[test]
fn all_honor_pungs_count_even_without_matching_seat_or_round() {
    let (h, c) = hand("111z 222z 333z 555z 666z 77z", "7z", vec![]);
    let result = score(&h, &c).unwrap();
    assert_award(&result, HonorPungs, 5);
    assert_award(&result, MixedOneSuit, 0);
    assert_award(&result, PureOneSuit, 0);
}

#[test]
fn concealed_and_exposed_kongs_and_hand_closure() {
    let (h, c) = hand(
        "123m 234p 678p 99s",
        "9s",
        vec![kong("1z", true), kong("5z", false)],
    );
    let result = score(&h, &c).unwrap();
    assert_award(&result, ConcealedKongs, 2);
    assert_award(&result, MeldedKongs, 1);
    assert_award(&result, HonorPungs, 2);
    assert_award(&result, ConcealedHand, 0);
    let (h, mut c) = hand("123m 234p 678p 345s 11z", "1z", vec![kong("5z", true)]);
    assert_award(&score(&h, &c).unwrap(), ConcealedHand, 1);
    c.source = WinSource::SelfDraw;
    let result = score(&h, &c).unwrap();
    assert_award(&result, FullyConcealedHand, 3);
    assert_award(&result, SelfDraw, 0);
    assert_award(&result, ConcealedHand, 0);
}

#[test]
fn clear_dragon_depends_on_the_three_sequences_not_the_entire_hand() {
    let (h, c) = hand("123m 456m 789m 234p 11z", "1z", vec![pung("5z")]);
    assert_award(&score(&h, &c).unwrap(), ConcealedClearDragon, 10);
    let (h, c) = hand(
        "456m 789m 234p 345s 11z",
        "1z",
        vec![Meld::Chow(Tile::Characters(1))],
    );
    let result = score(&h, &c).unwrap();
    assert_award(&result, ClearDragon, 5);
    assert_award(&result, ConcealedClearDragon, 0);
    let (h, c) = hand("123m 456p 789s 234p 345s 11z", "1z", vec![]);
    assert_award(&score(&h, &c).unwrap(), ClearDragon, 0);
}

#[test]
fn claimed_chow_opens_its_clear_dragon_but_not_the_hand() {
    let (h, mut c) = hand("123m 456m 789m 234p 345s 11z", "3m", vec![]);
    for source in [
        WinSource::SelfDraw,
        WinSource::Discard(Wind::West),
        WinSource::RobbedKong(Wind::West),
    ] {
        c.source = source;
        let result = score(&h, &c).unwrap();
        let drawn = source == WinSource::SelfDraw;
        assert_award(&result, ConcealedClearDragon, if drawn { 10 } else { 0 });
        assert_award(&result, ClearDragon, if drawn { 0 } else { 5 });
        assert_award(&result, FullyConcealedHand, if drawn { 3 } else { 0 });
        assert_award(&result, ConcealedHand, if drawn { 0 } else { 1 });
    }
    // Place the discard in 234m to keep the 123m/456m/789m pattern concealed.
    let (h, c) = hand("123m 456m 789m 234m 567p 11z", "3m", vec![]);
    assert_award(&score(&h, &c).unwrap(), ConcealedClearDragon, 10);
}

#[test]
fn concealed_triplet_tiers_replace_each_other() {
    let pungs = ["111m", "444m", "222p", "666p", "555z"];
    let tiers = [
        (TwoConcealedTriplets, 2),
        (ThreeConcealedTriplets, 5),
        (FourConcealedTriplets, 15),
        (FiveConcealedTriplets, 40),
    ];
    for count in 2..=5 {
        let complete = format!("{} 11s", pungs[..count].join(" "));
        let melds = pungs[count..]
            .iter()
            .map(|s| pung(&format!("{}{}", &s[..1], &s[3..])))
            .collect();
        let (h, c) = hand(&complete, "1s", melds);
        let result = score(&h, &c).unwrap();
        for (index, &(pattern, tai)) in tiers.iter().enumerate() {
            assert_award(&result, pattern, if index + 2 == count { tai } else { 0 });
        }
        assert_award(&result, PungHand, 10);
    }
}

#[test]
fn discarded_winning_pung_is_not_a_concealed_triplet() {
    let (h, mut c) = hand("111m 444m 222p 666p 555z 11s", "5z", vec![]);
    let discarded = score(&h, &c).unwrap();
    assert_award(&discarded, FourConcealedTriplets, 15);
    assert_award(&discarded, FiveConcealedTriplets, 0);
    assert_award(&discarded, ConcealedHand, 1);
    c.source = WinSource::SelfDraw;
    assert_award(&score(&h, &c).unwrap(), FiveConcealedTriplets, 40);
}

#[test]
fn kongs_count_as_concealed_triplets() {
    let (h, c) = hand(
        "123m 234p 345s 11z",
        "1z",
        vec![kong("5z", true), kong("6z", true)],
    );
    let result = score(&h, &c).unwrap();
    assert_award(&result, TwoConcealedTriplets, 2);
    assert_award(&result, ConcealedKongs, 4);
}

#[test]
fn dragon_and_wind_families() {
    let cases = [
        ("555z 666z 77z 123m 234p 345s", LittleThreeDragons, 15),
        ("555z 666z 777z 123m 234p 99s", BigThreeDragons, 30),
        ("111z 222z 33z 123m 234p 345s", LittleThreeWinds, 5),
        ("111z 222z 333z 123m 234p 99s", BigThreeWinds, 15),
        ("111z 222z 333z 44z 123m 234p", LittleFourWinds, 30),
        ("111z 222z 333z 444z 123m 99p", BigFourWinds, 40),
    ];
    let families = [
        LittleThreeDragons,
        BigThreeDragons,
        LittleThreeWinds,
        BigThreeWinds,
        LittleFourWinds,
        BigFourWinds,
    ];
    for (notation, pattern, tai) in cases {
        let (h, c) = hand(notation, "1m", vec![]);
        let result = score(&h, &c).unwrap();
        for p in families {
            assert_award(&result, p, if p == pattern { tai } else { 0 });
        }
    }
}

#[test]
fn suit_bonuses_use_every_tile_including_melds_and_pair() {
    for (notation, pattern, tai) in [
        ("123m 456m 789m 234m 678m 11z", MixedOneSuit, 10),
        ("123m 456m 789m 234m 678m 55m", PureOneSuit, 40),
    ] {
        let (h, c) = hand(notation, "1m", vec![]);
        assert_award(&score(&h, &c).unwrap(), pattern, tai);
    }
    let (h, c) = hand("123m 456m 789m 234m 55m", "5m", vec![pung("9p")]);
    let result = score(&h, &c).unwrap();
    assert_award(&result, PureOneSuit, 0);
    assert_award(&result, MixedOneSuit, 0);
}

#[test]
fn pair_edge_closed_and_two_sided_waits() {
    for (sequence, win, expected) in [
        ("123m", "3m", 1),
        ("789m", "7m", 1),
        ("456m", "5m", 1),
        ("456m", "4m", 0),
        ("456m", "6m", 0),
        ("123m", "1m", 0),
        ("789m", "9m", 0),
    ] {
        let (h, c) = hand(&format!("{sequence} 234p 678p 345s 678s 11z"), win, vec![]);
        let result = score(&h, &c).unwrap();
        assert_award(&result, OneChanceChow, expected);
        assert_award(&result, PairWait, 0);
    }
    let (h, c) = base();
    assert_award(&score(&h, &c).unwrap(), PairWait, 1);
}

#[test]
fn exposed_hand_replaces_pair_wait_but_concealed_kong_prevents_it() {
    let melds = vec![pung("1m"), pung("4m"), pung("2p"), pung("6p"), pung("5z")];
    let (mut h, mut c) = hand("11s", "1s", melds);
    let result = score(&h, &c).unwrap();
    assert_award(&result, ExposedHand, 10);
    assert_award(&result, PairWait, 0);
    c.source = WinSource::SelfDraw;
    let drawn = score(&h, &c).unwrap();
    assert_award(&drawn, ExposedHand, 10);
    assert_award(&drawn, SelfDraw, 1);
    h.melds[4] = kong("5z", true);
    let result = score(&h, &c).unwrap();
    assert_award(&result, ExposedHand, 0);
    assert_award(&result, PairWait, 1);
}

#[test]
fn winning_tile_placement_and_decomposition_are_optimized() {
    // The discard can complete the pair (five concealed pungs) or a chow.
    let (h, c) = hand("11122233344455566m", "6m", vec![]);
    let result = score(&h, &c).unwrap();
    assert_award(&result, FiveConcealedTriplets, 40);
    assert_award(&result, PungHand, 10);
    assert_award(&result, PairWait, 1);
    assert_eq!(result.total, 97);
    // Same tiles can place a discard in a chow or pung; preserve the concealed pung.
    let (h, c) = hand("222234m 555p 777s 111z 99s", "2m", vec![]);
    assert_award(&score(&h, &c).unwrap(), FourConcealedTriplets, 15);
}

#[test]
fn seven_pairs_allows_identical_pairs_and_claiming_the_triplet() {
    for notation in [
        "11m 33m 55m 22p 44p 66s 77z 111z",
        "1111m 55m 22p 44p 66s 77z 111z",
    ] {
        let (h, c) = hand(notation, "1z", vec![]);
        let result = score(&h, &c).unwrap();
        assert_award(&result, SevenPairsAndTriplet, 30);
        assert_award(&result, HonorPungs, 1);
        assert_award(&result, PairWait, 0);
        assert_award(&result, PungHand, 0);
    }
    let (h, c) = hand("11m 33m 55m 22p 44p 66s 77z 111z", "7z", vec![]);
    assert_award(&score(&h, &c).unwrap(), PairWait, 1);
    let (h, c) = hand("11m 33m 55m 22p 44p 66s 77z", "7z", vec![pung("1z")]);
    assert_eq!(score(&h, &c), Err(ScoreError::NotWinning));
}

#[test]
fn special_and_regular_interpretations_compete() {
    let (h, c) = hand("11223344556677888m", "8m", vec![]);
    let result = score(&h, &c).unwrap();
    assert_award(&result, SevenPairsAndTriplet, 30);
    assert_award(&result, PureOneSuit, 40);
}

#[test]
fn draw_robbery_and_last_tile_bonuses() {
    let (h, mut c) = hand("123m 456m 234p 678p 345s 11z", "3m", vec![]);
    c.last_tile = true;
    for (source, expected) in [
        (WinSource::SelfDraw, LastWallTile),
        (WinSource::Discard(Wind::West), LastDiscard),
        (WinSource::RobbedKong(Wind::West), LastDiscard),
    ] {
        c.source = source;
        let result = score(&h, &c).unwrap();
        assert_award(&result, expected, 1);
        assert_award(
            &result,
            RobbingKong,
            u16::from(matches!(source, WinSource::RobbedKong(_))),
        );
        assert_award(&result, SelfDraw, 0); // Replaced by fully concealed on a draw.
    }
    let (h, mut c) = hand("123m 234p 678p 11z", "1z", vec![pung("5z"), pung("6z")]);
    c.source = WinSource::SelfDraw;
    assert_award(&score(&h, &c).unwrap(), SelfDraw, 1);
}

#[test]
fn early_discard_boundaries() {
    let (h, mut c) = base();
    c.source = WinSource::SelfDraw;
    for (discards, early, very_early) in [(1, 0, 10), (5, 0, 10), (6, 5, 0), (9, 5, 0), (10, 0, 0)]
    {
        c.discards = discards;
        let result = score(&h, &c).unwrap();
        assert_award(&result, EarlyWin, early);
        assert_award(&result, VeryEarlyWin, very_early);
    }
}

#[test]
fn dealer_identity_drives_opening_wins() {
    let (h, mut c) = base();
    c.dealer = Wind::South;
    c.source = WinSource::SelfDraw;
    c.discards = 0;
    let result = score(&h, &c).unwrap();
    assert_award(&result, HeavenlyHand, 40);
    assert_award(&result, VeryEarlyWin, 0);
    c.winner = Wind::North;
    c.source = WinSource::Discard(Wind::South);
    c.discards = 1;
    let result = score(&h, &c).unwrap();
    assert_award(&result, EarthlyHand, 40);
    assert_award(&result, VeryEarlyWin, 0);
}

#[test]
fn original_hand_ready_bonus_is_explicit() {
    let (h, mut c) = base();
    assert_award(&score(&h, &c).unwrap(), ReadyOnOriginalHand, 0);
    c.ready_on_original_hand = true;
    c.discards = 5;
    let result = score(&h, &c).unwrap();
    assert_award(&result, ReadyOnOriginalHand, 15);
    assert_award(&result, VeryEarlyWin, 10);
}
