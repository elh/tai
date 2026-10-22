//! A complete hand and scoring call, also shown in the README.
use tai::{Hand, WinContext, WinSource, Wind, parse_tile, parse_tiles, score};

fn main() -> Result<(), tai::ScoreError> {
    let hand = Hand::from(parse_tiles("123456m 234678p 345s 1z")?);
    let win = WinContext {
        winning_tile: parse_tile("1z")?,
        winner: Wind::South,
        dealer: Wind::East,
        source: WinSource::Discard(Wind::West),
        discards: 10,
        last_tile: false,
        ready_on_original_hand: false,
    };
    let result = score(&hand, &win)?;
    assert_eq!(result.total, 8);
    println!("{} tai", result.total);
    for award in result.awards {
        println!("{}: {} tai", award.pattern, award.tai);
    }
    Ok(())
}
