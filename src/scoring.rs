use crate::decompose::{self, Kind, Set};
use crate::{Hand, Meld, Pattern, Score, ScoreError, Tile, WinContext, WinSource};

/// Validate and score a 16-tile hand plus its winning tile, selecting the best arrangement.
///
/// Every legal placement of the winning tile is considered. A claimed tile makes its
/// completed set exposed for pattern bonuses, but does not open the hand itself.
/// See the [rule conventions](index.html#rule-conventions) where the source is ambiguous.
/// Flowers are always enabled; eight-flower wins must use [`score_flowers`].
///
/// # Errors
/// Returns [`ScoreError`] for invalid tiles, counts, flowers, contradictory context,
/// or tiles that cannot form a winning hand.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn score(hand: &Hand, context: &WinContext) -> Result<Score, ScoreError> {
    let validated = validate(hand, context)?;
    let mut best: Option<Score> = None;
    let mut consider = |sets: &[Set], shape, wait| {
        let candidate = score_arrangement(hand, context, &validated, sets, shape, wait);
        if best
            .as_ref()
            .is_none_or(|best| candidate.total > best.total)
        {
            best = Some(candidate);
        }
    };
    for arrangement in decompose::arrangements(&validated.loose) {
        // Fixed melds cannot contain the newly obtained winning tile.
        let loose_len = arrangement.sets.len();
        let mut sets = arrangement.sets;
        sets.extend_from_slice(&validated.declared);
        let shape = Shape::Regular(arrangement.pair);
        if arrangement.pair == validated.winning {
            consider(&sets, shape, Wait::Pair);
        }
        for index in 0..loose_len {
            if sets[index].contains(validated.winning) {
                let wait = if sets[index].kind == Kind::Chow {
                    let start = sets[index].tile;
                    if validated.winning == start + 1
                        || (start % 9 == 0 && validated.winning == start + 2)
                        || (start % 9 == 6 && validated.winning == start)
                    {
                        Wait::OneChanceChow
                    } else {
                        Wait::Other
                    }
                } else {
                    Wait::Other
                };
                let mut candidate_sets = sets.clone();
                if context.source != WinSource::SelfDraw {
                    candidate_sets[index].concealed = false;
                }
                consider(&candidate_sets, shape, wait);
            }
        }
    }
    // A quad may supply two identical pairs, but a declared kong may not.
    if hand.melds.is_empty() {
        let odd: Vec<_> = validated
            .loose
            .iter()
            .enumerate()
            .filter(|&(_, &count)| count % 2 == 1)
            .collect();
        if let [(tile, 3)] = odd.as_slice() {
            let set = Set {
                tile: *tile,
                kind: Kind::Pung,
                concealed: context.source == WinSource::SelfDraw || *tile != validated.winning,
            };
            let wait = if *tile == validated.winning {
                Wait::Other
            } else {
                Wait::Pair
            };
            consider(&[set], Shape::SevenPairs, wait);
        }
    }
    best.ok_or(ScoreError::NotWinning)
}

/// Score a flower-only win: eight held flowers/seasons (30 tai), or seven held
/// before robbing the eighth (20 tai). Set `robbed_eighth` only for the latter;
/// seven held alone is not a flower-only win. Other patterns never contribute.
///
/// # Errors
/// Returns [`ScoreError::InvalidFlowers`] for any other count/event combination.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn score_flowers(flowers: u8, robbed_eighth: bool) -> Result<Score, ScoreError> {
    let mut score = Score::default();
    match (flowers, robbed_eighth) {
        (8, false) => score.add(Pattern::EightFlowers, 30),
        (7, true) => {
            score.add(Pattern::SevenFlowersRobbingEighth, 20);
        }
        _ => return Err(ScoreError::InvalidFlowers),
    }
    Ok(score)
}

struct Validated {
    loose: [u8; 34],
    all: [u8; 34],
    declared: Vec<Set>,
    winning: usize,
}

fn validate(hand: &Hand, context: &WinContext) -> Result<Validated, ScoreError> {
    if hand.flowers > 8 {
        return Err(ScoreError::InvalidFlowers);
    }
    if hand.flowers == 8 {
        return Err(ScoreError::FlowerWinRequired);
    }
    if hand.melds.len() > 5 {
        return Err(ScoreError::TooManyMelds);
    }
    if hand.concealed.len() != 16 - 3 * hand.melds.len() {
        return Err(ScoreError::WrongTileCount);
    }
    let winning = context.winning_tile.index()?;
    let mut loose = [0; 34];
    for tile in hand.concealed.iter().chain([&context.winning_tile]) {
        add_tile(&mut loose, tile.index()?)?;
    }
    let mut all = loose;
    let mut declared = Vec::with_capacity(hand.melds.len());
    for &meld in &hand.melds {
        let set = match meld {
            Meld::Chow(tile) => {
                let tile = tile.index().map_err(|_| ScoreError::InvalidChow)?;
                if tile >= 27 || tile % 9 > 6 {
                    return Err(ScoreError::InvalidChow);
                }
                Set {
                    tile,
                    kind: Kind::Chow,
                    concealed: false,
                }
            }
            Meld::Pung(tile) => Set {
                tile: tile.index()?,
                kind: Kind::Pung,
                concealed: false,
            },
            Meld::Kong(tile) | Meld::ConcealedKong(tile) => Set {
                tile: tile.index()?,
                kind: Kind::Kong,
                concealed: matches!(meld, Meld::ConcealedKong(_)),
            },
        };
        let count = if set.kind == Kind::Kong { 4 } else { 3 };
        for offset in 0..count {
            let tile = set.tile + if set.kind == Kind::Chow { offset } else { 0 };
            add_tile(&mut all, tile)?;
        }
        declared.push(set);
    }
    validate_context(hand, context)?;
    // The opponent retains three copies in the pung being upgraded.
    if matches!(context.source, WinSource::RobbedKong(_)) && all[winning] > 1 {
        return Err(ScoreError::TooManyCopies {
            tile: context.winning_tile,
        });
    }
    Ok(Validated {
        loose,
        all,
        declared,
        winning,
    })
}

fn add_tile(counts: &mut [u8; 34], index: usize) -> Result<(), ScoreError> {
    counts[index] += 1;
    if counts[index] > 4 {
        return Err(ScoreError::TooManyCopies {
            tile: Tile::from_index(index),
        });
    }
    Ok(())
}

fn heavenly(context: &WinContext) -> bool {
    context.discards == 0
        && context.winner == context.dealer
        && context.source == WinSource::SelfDraw
}

fn earthly(context: &WinContext) -> bool {
    context.discards == 1
        && matches!(context.source, WinSource::Discard(from) if from == context.dealer)
}

fn validate_context(hand: &Hand, context: &WinContext) -> Result<(), ScoreError> {
    let invalid = |reason: &str| {
        Err(ScoreError::InvalidContext {
            reason: reason.into(),
        })
    };
    match context.source {
        WinSource::Discard(from) | WinSource::RobbedKong(from) if from == context.winner => {
            return invalid("the winner cannot supply their own claimed tile");
        }
        WinSource::Discard(_) if context.discards == 0 => {
            return invalid("discard count must include the winning discard");
        }
        WinSource::Discard(from) if context.discards == 1 && from != context.dealer => {
            return invalid("the first discard must come from the dealer");
        }
        WinSource::RobbedKong(_) if context.discards == 0 => {
            return invalid("an upgraded pung requires a prior discard to claim");
        }
        WinSource::SelfDraw if context.discards == 0 && context.winner != context.dealer => {
            return invalid("only the dealer can self-draw before any discard");
        }
        _ => {}
    }
    if earthly(context) && !hand.melds.is_empty() {
        return invalid("a winner on the dealer's first discard cannot already have declared sets");
    }
    if heavenly(context)
        && hand
            .melds
            .iter()
            .any(|meld| !matches!(meld, Meld::ConcealedKong(_)))
    {
        return invalid("a win on the initial deal cannot contain exposed sets");
    }
    if (heavenly(context) || earthly(context)) && context.last_tile {
        return invalid("an opening win cannot be on the last wall tile or discard");
    }
    if context.ready_on_original_hand && (!hand.melds.is_empty() || heavenly(context)) {
        return invalid("original-hand readiness requires the undeclared original 16 tiles");
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wait {
    Pair,
    OneChanceChow,
    Other,
}

#[derive(Clone, Copy)]
enum Shape {
    Regular(usize), // Pair tile.
    SevenPairs,
}

fn score_arrangement(
    hand: &Hand,
    context: &WinContext,
    validated: &Validated,
    sets: &[Set],
    shape: Shape,
    wait: Wait,
) -> Score {
    use Pattern::*;
    let mut score = Score::default();
    score.add(Winning, 2);
    let honors = validated.all[27..].iter().any(|&n| n > 0);
    let no_flowers = hand.flowers == 0;
    let regular = matches!(shape, Shape::Regular(_));
    let all_chows = regular && sets.iter().all(|s| s.kind == Kind::Chow);
    // Composite absence bonuses replace their component bonuses.
    if all_chows && !honors && no_flowers {
        score.add(ChowHandNoHonorsOrFlowers, 10);
    } else {
        if !honors && no_flowers {
            score.add(NoHonorsOrFlowers, 3);
        } else {
            if !honors {
                score.add(NoHonors, 1);
            }
            if no_flowers {
                score.add(NoFlowers, 1);
            }
        }
        if all_chows {
            score.add(ChowHand, 3);
        }
    }
    score.add(Flowers, u16::from(hand.flowers));
    let count = |test: fn(&Set) -> bool| sets.iter().filter(|s| test(s)).count() as u16;
    score.add(HonorPungs, count(|s| s.tile >= 27 && s.kind != Kind::Chow));
    score.add(MeldedKongs, count(|s| s.kind == Kind::Kong && !s.concealed));
    score.add(
        ConcealedKongs,
        2 * count(|s| s.kind == Kind::Kong && s.concealed),
    );

    for suit in 0..3 {
        let dragon = |concealed: bool| {
            [0, 3, 6].iter().all(|start| {
                sets.iter().any(|s| {
                    s.kind == Kind::Chow
                        && s.tile == suit * 9 + start
                        && (!concealed || s.concealed)
                })
            })
        };
        if dragon(true) {
            score.add(ConcealedClearDragon, 10);
        } else if dragon(false) {
            score.add(ClearDragon, 5);
        }
    }
    match count(|s| s.kind != Kind::Chow && s.concealed) {
        2 => score.add(TwoConcealedTriplets, 2),
        3 => score.add(ThreeConcealedTriplets, 5),
        4 => score.add(FourConcealedTriplets, 15),
        5 => score.add(FiveConcealedTriplets, 40),
        _ => {}
    }
    let triplets = |range: std::ops::Range<usize>| {
        sets.iter()
            .filter(|s| s.kind != Kind::Chow && range.contains(&s.tile))
            .count()
    };
    let pair_in = |range: std::ops::Range<usize>| matches!(shape, Shape::Regular(pair) if range.contains(&pair));
    match (triplets(31..34), pair_in(31..34)) {
        (3, _) => score.add(BigThreeDragons, 30),
        (2, true) => score.add(LittleThreeDragons, 15),
        _ => {}
    }
    match (triplets(27..31), pair_in(27..31)) {
        (4, _) => score.add(BigFourWinds, 40),
        (3, true) => score.add(LittleFourWinds, 30),
        (3, false) => score.add(BigThreeWinds, 15),
        (2, true) => score.add(LittleThreeWinds, 5),
        _ => {}
    }
    let closed = validated.declared.iter().all(|s| s.concealed);
    match (closed, context.source == WinSource::SelfDraw) {
        (true, true) => score.add(FullyConcealedHand, 3),
        (true, false) => score.add(ConcealedHand, 1),
        (false, true) => score.add(SelfDraw, 1),
        (false, false) => {}
    }
    let exposed = hand.melds.len() == 5 && validated.declared.iter().all(|s| !s.concealed);
    if exposed {
        score.add(ExposedHand, 10);
    } else if wait == Wait::Pair {
        score.add(PairWait, 1);
    }
    if wait == Wait::OneChanceChow {
        score.add(OneChanceChow, 1);
    }
    if regular && sets.iter().all(|s| s.kind != Kind::Chow) {
        score.add(PungHand, 10);
    }
    let suits = validated.all[..27]
        .as_chunks::<9>()
        .0
        .iter()
        .filter(|suit| suit.iter().any(|&n| n > 0))
        .count();
    if suits == 1 {
        if honors {
            score.add(MixedOneSuit, 10);
        } else {
            score.add(PureOneSuit, 40);
        }
    }
    if context.last_tile {
        if context.source == WinSource::SelfDraw {
            score.add(LastWallTile, 1);
        } else {
            score.add(LastDiscard, 1);
        }
    }
    if matches!(context.source, WinSource::RobbedKong(_)) {
        score.add(RobbingKong, 1);
    }
    if heavenly(context) {
        score.add(HeavenlyHand, 40);
    } else if earthly(context) {
        score.add(EarthlyHand, 40);
    } else if context.discards <= 5 {
        score.add(VeryEarlyWin, 10);
    } else if context.discards < 10 {
        score.add(EarlyWin, 5);
    }
    if context.ready_on_original_hand {
        score.add(ReadyOnOriginalHand, 15);
    }
    if matches!(shape, Shape::SevenPairs) {
        score.add(SevenPairsAndTriplet, 30);
    }
    score
}
