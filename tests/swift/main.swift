import Tai

// Complete construction with default empty melds/flowers and false optional flags.
let hand = Hand(concealed: try parseTiles(notation: "123456m 234678p 345s 1z"))
var context = WinContext(winningTile: try parseTile(notation: "1z"),
                         winner: .south, dealer: .east, source: .discard(.west), discards: 10)
let result = try score(hand: hand, context: context)
precondition(result.total == 8)
precondition(result.awards.map(\.description) == [
    "Winning (base)", "No flowers or seasons", "Chow hand",
    "Concealed hand (claimed winning tile)", "Out on a pair",
])
precondition(result.awards.map(\.tai).reduce(0, +) == result.total)
precondition(result.awards.contains { $0.pattern == .noFlowers && $0.tai == 1 })
precondition(hand.melds.isEmpty && hand.flowers.isEmpty)
precondition(!context.lastTile && !context.readyOnOriginalHand)

// The typed API needs no notation or nested suit/dragon records.
let typed: [Tile] = [.characters(1), .characters(2), .characters(3),
                    .characters(4), .characters(5), .characters(6),
                    .dots(2), .dots(3), .dots(4), .dots(6), .dots(7), .dots(8),
                    .bamboo(3), .bamboo(4), .bamboo(5), .east]
precondition(hand.concealed == typed)
let typedResult = try score(hand: Hand(concealed: typed), context: context)
precondition(typedResult == result)

// The reference's 16-tai subtotal plus the mandatory no-flowers bonus.
let referenceHand = Hand(concealed: try parseTiles(notation: "777p 1z"),
                         melds: [.pung(.south), .kong(.red), .pung(.characters(2)), .pung(.bamboo(6))])
let referenceScore = try score(hand: referenceHand, context: context)
precondition(referenceScore.total == 17)
precondition(referenceScore.awards == [
    Award(pattern: .winning, tai: 2, description: "Winning (base)"),
    Award(pattern: .noFlowers, tai: 1, description: "No flowers or seasons"),
    Award(pattern: .honorPungs, tai: 2, description: "Pungs/kongs of honors"),
    Award(pattern: .meldedKongs, tai: 1, description: "Melded kongs"),
    Award(pattern: .pairWait, tai: 1, description: "Out on a pair"),
    Award(pattern: .pungHand, tai: 10, description: "Pung hand"),
])
// Exercise ambiguous regular hands and the seven-pairs special shape through Swift.
for (notation, winning, expected): (String, Tile, UInt16) in [
    ("1112223334445556m", .characters(6), 97),
    ("113355m 2244p 66s 77z 11z", .east, 35),
] {
    var win = context
    win.winningTile = winning
    let scored = try score(hand: Hand(concealed: try parseTiles(notation: notation)), context: win)
    precondition(scored.total == expected)
}
var invalidContext = context
invalidContext.discards = 1
do {
    _ = try score(hand: hand, context: invalidContext)
    fatalError("first discard from a non-dealer was accepted")
} catch ScoreError.InvalidContext(let reason) {
    precondition(!reason.isEmpty)
}

let flowers: [Flower] = [.plum, .orchid, .chrysanthemum, .bamboo, .spring, .summer, .autumn, .winter]
let allFlowers = try scoreFlowers(flowers: flowers, robbed: nil)
precondition(allFlowers.total == 30 && allFlowers.awards.count == 1)
precondition(allFlowers.awards[0].description == "All flowers and seasons")
let robbed = try scoreFlowers(flowers: Array(flowers.prefix(7)), robbed: .winter)
precondition(robbed.total == 20)
precondition(robbed.awards[0].description == "Seven flowers and seasons, robbing the eighth")

// Every meld variant crosses the generated interface, along with the new tile enums.
for meld: Meld in [.chow(.characters(1)), .pung(.red), .kong(.red), .concealedKong(.red)] {
    let meldHand = Hand(concealed: Array(typed.dropFirst(3)), melds: [meld], flowers: [.plum])
    let meldScore = try score(hand: meldHand, context: context)
    precondition(meldScore.total > 0)
}
let kongHand = Hand(concealed: try parseTiles(notation: "46m 234678p 345s 11z"),
                    melds: [.concealedKong(.red)])
context.winningTile = .characters(5)
context.source = .robbedKong(.north)
context.lastTile = true
let kongScore = try score(hand: kongHand, context: context)
precondition(kongScore.awards.contains { $0.pattern == .concealedKongs && $0.tai == 2 })
precondition(kongScore.awards.contains { $0.pattern == .robbingKong && $0.tai == 1 })
context.winningTile = .east
do {
    _ = try score(hand: hand, context: context)
    fatalError("robbed kong accepted a fifth copy of East")
} catch ScoreError.TooManyCopies(let tile) {
    precondition(tile == .east)
}
context.source = .selfDraw
let drawnScore = try score(hand: hand, context: context)
precondition(drawnScore.awards.contains { $0.pattern == .fullyConcealedHand && $0.tai == 3 })

// Typed Rust errors survive the boundary; flower-only wins have one explicit route.
do {
    _ = try score(hand: Hand(concealed: [], flowers: flowers), context: context)
    fatalError("tile scoring accepted a flower-only win")
} catch ScoreError.FlowerWinRequired {}
do {
    _ = try parseTiles(notation: "8z")
    fatalError("invalid notation was accepted")
} catch ScoreError.InvalidNotation {}
let singleTile = try parseTile(notation: " 9p\n")
precondition(singleTile == .dots(9))
for notation in ["", "11z", "1m 2p", "8z"] {
    do {
        _ = try parseTile(notation: notation)
        fatalError("invalid single-tile notation was accepted: \(notation)")
    } catch ScoreError.InvalidNotation {}
}
context.winningTile = .dots(0)
do {
    _ = try score(hand: hand, context: context)
    fatalError("invalid rank was accepted")
} catch ScoreError.InvalidRank {}
do {
    _ = try scoreFlowers(flowers: [.plum, .plum], robbed: nil)
    fatalError("duplicate flower was accepted")
} catch ScoreError.DuplicateFlower(let flower) {
    precondition(flower == .plum)
}
for _ in 0..<100 {
    let repeated = try scoreFlowers(flowers: flowers, robbed: nil)
    precondition(repeated.total == 30)
}
print("Swift 6 integration passed: defaults, notation, typed tiles/melds, scoring, descriptions, errors, repeated calls")
