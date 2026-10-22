//! A pure calculator for Mahjong Time's Taiwanese (16-tile) scoring rules.
//!
//! [`Hand::concealed`] excludes the winning tile, which belongs in [`WinContext`].
//! [`score`] validates the input and selects the highest-scoring interpretation.
//! [`score_flowers`] handles flower-only wins. No game history or payments are managed.
//! See the [rule conventions](#rule-conventions) for scoring overlaps and edge cases.
//!
//! ```
//! use tai::{WinContext, WinSource, Wind, parse_hand, parse_tile, score};
//!
//! let hand = parse_hand("[111s] {5555z} [666z] 77s 56m 777z 3f")?;
//! let win = WinContext {
//!     winning_tile: parse_tile("4m")?,
//!     winner: Wind::South,
//!     dealer: Wind::East,
//!     source: WinSource::Discard(Wind::West),
//!     discards: 10,
//!     last_tile: false,
//!     ready_on_original_hand: false,
//! };
//! let result = score(&hand, &win)?;
//! assert_eq!(result.total, 42);
//! for award in result.awards {
//!     println!("{}: {} tai", award.pattern, award.tai);
//! }
//! # Ok::<(), tai::ScoreError>(())
//! ```
//!
#![doc = include_str!("../docs/rules.md")]

mod decompose;
mod model;
mod notation;
mod scoring;

pub use model::*;
pub use notation::{parse_hand, parse_tile, parse_tiles};
pub use scoring::{score, score_flowers};

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
