# tai

A small Rust calculator for 16-tile Taiwanese mahjong, with typed Swift bindings.
Validates a hand and returns its highest tai score with an itemized, English breakdown.
Flowers are always enabled. Keep the winning tile separate from the held tiles.

```rust
use tai::{WinContext, WinSource, Wind, parse_hand, parse_tile, score};

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
```

**Hand notation:** MPSZ (`m` characters, `p` dots, `s` bamboo;
`1–7z` East/South/West/North/White/Green/Red) with these extensions:

- Plain tiles are concealed; `[123m]`, `[111z]`, `[1111z]` are exposed chows, pungs, kongs.
- `{1111z}` is a declared concealed kong. Loose quads stay plain; upgraded pungs use `[...]`.
- `3f` means three flowers/seasons in total. One `0f`–`8f` group is allowed outside
  brackets; omission means zero. This is a custom count extension, not tile identities.
- Groups may touch or have whitespace; tile order within a set does not matter. No nesting.

Supply the winning tile separately. `hand.flowers` is a count (`u8` / Swift `UInt8`).
`parse_tiles` reads plain tiles; `parse_tile` requires one. Red fives are unsupported.
Typed `Hand`/`Meld` values work too. Flower-only wins use `score_flowers(8, false)`
(30 tai) or `score_flowers(7, true)` for robbing the eighth (20 tai).

For iOS, run `./scripts/swift.sh ios`; add `target/swift/Tai.swift` and
`TaiFFI.xcframework` to Xcode (Do Not Embed).

```swift
let hand = try parseHand(notation: "[111s] {5555z} [666z] 77s 56m 777z 3f")
let win = WinContext(winningTile: try parseTile(notation: "4m"),
                     winner: .south, dealer: .east, source: .discard(.west), discards: 10)
let result = try score(hand: hand, context: win)
for award in result.awards {
    print("\(award.description): \(award.tai) tai")
}
```

Rules credited to **Mahjong Time**: [part 1](https://mahjongtime.com/mahjong-taiwanese-scoring.html),
[part 2](https://mahjongtime.com/mahjong-taiwanese-scoring-2.html).
[Conventions](docs/rules.md) · [Rust example](examples/score.rs) · [Swift example](tests/swift/main.swift).
