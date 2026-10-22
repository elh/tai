//! A complete hand and scoring call, also shown in the README.
use tai::{WinContext, WinSource, Wind, parse_hand, parse_tile, score};

fn main() -> Result<(), tai::ScoreError> {
    let hand = parse_hand("[111s] {5555z} [666z] 77s 56m 777z 3f")?;
    let win = WinContext {
        winning_tile: parse_tile("4m")?,
        winner: Wind::South,
        dealer: Wind::East,
        source: WinSource::Discard(Wind::West),
        discards: 10,
        last_tile: false,
        ready_on_original_hand: false,
    };
    let result = score(&hand, &win)?;
    assert_eq!(result.total, 42);
    println!("{} tai", result.total);
    for award in result.awards {
        println!("{}: {} tai", award.pattern, award.tai);
    }
    Ok(())
}
