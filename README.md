# tai

A small Rust calculator for 16-tile Taiwanese mahjong, with typed Swift bindings.
Validates a hand and returns its highest tai score with an itemized, English breakdown.
Flowers are always enabled. Keep the winning tile separate from the held tiles.

```rust
use tai::{Hand, WinContext, WinSource, Wind, parse_tile, parse_tiles, score};

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
```

Notation: `m` characters, `p` dots, `s` bamboo; `1–7z` East/South/West/North/Red/Green/White.
Typed tiles work too: `Tile::Characters(3)`, `Meld::Pung(Tile::Red)`.
Flower-only wins use `score_flowers(&held_flowers, robbed_flower)`.

For iOS, run `./scripts/swift.sh ios`; add `target/swift/Tai.swift` and
`TaiFFI.xcframework` to Xcode (Do Not Embed).

```swift
let hand = Hand(concealed: try parseTiles(notation: "123456m 234678p 345s 1z"))
let win = WinContext(winningTile: try parseTile(notation: "1z"),
                     winner: .south, dealer: .east, source: .discard(.west), discards: 10)
let result = try score(hand: hand, context: win)
for award in result.awards {
    print("\(award.description): \(award.tai) tai")
}
```

Rules credited to **Mahjong Time**: [part 1](https://mahjongtime.com/mahjong-taiwanese-scoring.html),
[part 2](https://mahjongtime.com/mahjong-taiwanese-scoring-2.html).
[Conventions](docs/rules.md) · [Rust example](examples/score.rs) · [Swift example](tests/swift/main.swift).
