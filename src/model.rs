use std::fmt;

/// Player identifiers. The dealer is supplied separately from the winner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[allow(missing_docs)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

/// Ordinary tiles. Numbered ranks must be 1..=9; scoring validates them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[allow(missing_docs)]
pub enum Tile {
    Characters(u8),
    Bamboo(u8),
    Dots(u8),
    East,
    South,
    West,
    North,
    Red,
    Green,
    White,
}

impl Tile {
    pub(crate) fn index(self) -> Result<usize, ScoreError> {
        let (offset, rank) = match self {
            Self::Characters(rank) => (0, rank),
            Self::Bamboo(rank) => (9, rank),
            Self::Dots(rank) => (18, rank),
            Self::East => return Ok(27),
            Self::South => return Ok(28),
            Self::West => return Ok(29),
            Self::North => return Ok(30),
            Self::Red => return Ok(31),
            Self::Green => return Ok(32),
            Self::White => return Ok(33),
        };
        if !(1..=9).contains(&rank) {
            return Err(ScoreError::InvalidRank);
        }
        Ok(offset + usize::from(rank - 1))
    }

    pub(crate) fn from_index(index: usize) -> Self {
        match index {
            0..=8 => Self::Characters(index as u8 + 1),
            9..=17 => Self::Bamboo(index as u8 - 8),
            18..=26 => Self::Dots(index as u8 - 17),
            27..=33 => [
                Self::East,
                Self::South,
                Self::West,
                Self::North,
                Self::Red,
                Self::Green,
                Self::White,
            ][index - 27],
            _ => unreachable!("internal tile index"),
        }
    }
}

/// A set declared before winning. Undeclared sets stay in `Hand::concealed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum Meld {
    /// Exposed sequence, specified by its lowest tile, e.g. `Chow(Tile::Dots(3))`.
    Chow(Tile),
    /// Exposed triplet.
    Pung(Tile),
    /// Exposed quad, including a pung upgraded to a kong.
    Kong(Tile),
    /// Quad declared concealed.
    ConcealedKong(Tile),
}

/// Tiles held before winning: exclude the winning tile and all declared melds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Hand {
    /// Loose tiles, excluding the winning tile. Exactly `16 - 3 * melds.len()`.
    pub concealed: Vec<Tile>,
    /// Declared sets; kongs count as one set. Defaults to empty in Swift.
    #[cfg_attr(feature = "uniffi", uniffi(default))]
    pub melds: Vec<Meld>,
    /// Total flowers and seasons held, 0..=8 (default 0). Identities do not affect
    /// these rules. Eight flowers use [`crate::score_flowers`].
    #[cfg_attr(feature = "uniffi", uniffi(default = 0))]
    pub flowers: u8,
}

impl From<Vec<Tile>> for Hand {
    fn from(concealed: Vec<Tile>) -> Self {
        Self {
            concealed,
            ..Self::default()
        }
    }
}

/// How the winning tile was obtained. Claimed tiles identify the supplying player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum WinSource {
    /// Wall or supplement draw.
    SelfDraw,
    /// Claim from the named player's discard.
    Discard(Wind),
    /// Claim while the named player upgrades a pung to a kong.
    RobbedKong(Wind),
}

/// Facts needed to score a tile win. Historical facts are supplied by the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct WinContext {
    /// The completing tile; exclude it from `Hand::concealed`.
    pub winning_tile: Tile,
    /// Winner's seat identifier.
    pub winner: Wind,
    /// Dealer's seat identifier.
    pub dealer: Wind,
    /// Draw, discard, or robbed kong.
    pub source: WinSource,
    /// All discards, including claimed ones and the winning discard.
    /// Zero with a dealer self-draw means the initial deal.
    pub discards: u16,
    /// Last wall draw for self-draw, or last discard for a claimed win.
    #[cfg_attr(feature = "uniffi", uniffi(default = false))]
    pub last_tile: bool,
    /// Readiness was declared on the original 16 tiles and retained without declaring sets.
    #[cfg_attr(feature = "uniffi", uniffi(default = false))]
    pub ready_on_original_hand: bool,
}

/// Scoring patterns. A score contains only the strongest tier in each bonus family.
/// `Display` gives a short English reason using Mahjong Time's terminology.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[allow(missing_docs)]
pub enum Pattern {
    /// The base 2 tai for a tile win. Flower-only wins exclude it.
    Winning,
    Flowers,
    NoFlowers,
    SevenFlowersRobbingEighth,
    EightFlowers,
    HonorPungs,
    MeldedKongs,
    ConcealedKongs,
    ClearDragon,
    ConcealedClearDragon,
    TwoConcealedTriplets,
    ThreeConcealedTriplets,
    FourConcealedTriplets,
    FiveConcealedTriplets,
    LittleThreeDragons,
    BigThreeDragons,
    LittleThreeWinds,
    BigThreeWinds,
    LittleFourWinds,
    BigFourWinds,
    ExposedHand,
    ConcealedHand,
    FullyConcealedHand,
    ChowHand,
    ChowHandNoHonorsOrFlowers,
    PungHand,
    NoHonors,
    NoHonorsOrFlowers,
    MixedOneSuit,
    PureOneSuit,
    SelfDraw,
    OneChanceChow,
    PairWait,
    LastWallTile,
    RobbingKong,
    LastDiscard,
    EarlyWin,
    VeryEarlyWin,
    ReadyOnOriginalHand,
    SevenPairsAndTriplet,
    HeavenlyHand,
    EarthlyHand,
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Winning => "Winning (base)",
            Self::Flowers => "Flowers and seasons",
            Self::NoFlowers => "No flowers or seasons",
            Self::SevenFlowersRobbingEighth => "Seven flowers and seasons, robbing the eighth",
            Self::EightFlowers => "All flowers and seasons",
            Self::HonorPungs => "Pungs/kongs of honors",
            Self::MeldedKongs => "Melded kongs",
            Self::ConcealedKongs => "Concealed kongs",
            Self::ClearDragon => "Clear dragon (three chows: 123, 456, 789 of one suit)",
            Self::ConcealedClearDragon => {
                "Concealed clear dragon (three chows: 123, 456, 789 of one suit)"
            }
            Self::TwoConcealedTriplets => "Two concealed triplets",
            Self::ThreeConcealedTriplets => "Three concealed triplets",
            Self::FourConcealedTriplets => "Four concealed triplets",
            Self::FiveConcealedTriplets => "Five concealed triplets",
            Self::LittleThreeDragons => "Little three dragons",
            Self::BigThreeDragons => "Big three dragons (three great scholars)",
            Self::LittleThreeWinds => "Little three winds",
            Self::BigThreeWinds => "Big three winds",
            Self::LittleFourWinds => "Little four winds",
            Self::BigFourWinds => "Big four winds",
            Self::ExposedHand => "Exposed hand",
            Self::ConcealedHand => "Concealed hand (claimed winning tile)",
            Self::FullyConcealedHand => "Fully concealed hand (self-drawn)",
            Self::ChowHand => "Chow hand",
            Self::ChowHandNoHonorsOrFlowers => "Chow hand with no honors or flowers",
            Self::PungHand => "Pung hand",
            Self::NoHonors => "No honors",
            Self::NoHonorsOrFlowers => "No flowers and no honors",
            Self::MixedOneSuit => "One suit and honors (mixed hand)",
            Self::PureOneSuit => "One suit only (clear hand)",
            Self::SelfDraw => "Self-drawn last tile",
            Self::OneChanceChow => "Out on a one-chance chow (edge or middle wait)",
            Self::PairWait => "Out on a pair",
            Self::LastWallTile => "Out on the last tile of the wall",
            Self::RobbingKong => "Out by robbing a kong",
            Self::LastDiscard => "Out on the last discard",
            Self::EarlyWin => "Early winning (6–9 discards)",
            Self::VeryEarlyWin => "Early winning (5 or fewer discards)",
            Self::ReadyOnOriginalHand => "Ready on original hand",
            Self::SevenPairsAndTriplet => "Seven pairs and a triplet",
            Self::HeavenlyHand => "Heavenly hand",
            Self::EarthlyHand => "Earthly hand",
        })
    }
}

/// One scoring award; repeated flowers, honor pungs, or kongs are aggregated by pattern.
///
/// A returned award is a snapshot: its description matches its pattern. Editing
/// these public fields does not recalculate the description or tai value.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Award {
    /// Scoring pattern; use this identifier for localization or other app logic.
    pub pattern: Pattern,
    /// Total tai awarded for this pattern.
    pub tai: u16,
    /// Short English reason using Mahjong Time's terminology; excludes the tai value.
    pub description: String,
}

/// Snapshot of the highest-scoring interpretation. No monetary conversion or cap is applied.
///
/// On return from [`crate::score`] or [`crate::score_flowers`], `total` is the sum
/// of `awards`. Editing these public fields does not recalculate related values.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Score {
    /// Sum of `awards` in tai, including the base award for tile wins.
    pub total: u16,
    /// Itemized awards, in deterministic rule order.
    pub awards: Vec<Award>,
}

impl Score {
    pub(crate) fn add(&mut self, pattern: Pattern, tai: u16) {
        if tai > 0 {
            self.total += tai;
            self.awards.push(Award {
                pattern,
                tai,
                description: pattern.to_string(),
            });
        }
    }
}

/// Invalid input or an incomplete winning hand.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ScoreError {
    /// Invalid tile/set notation, or not exactly one tile when parsing a single tile.
    InvalidNotation,
    /// Numbered tile rank is outside 1 through 9.
    InvalidRank,
    /// A chow must start at a numbered tile ranked 1 through 7.
    InvalidChow,
    /// More than five declared sets.
    TooManyMelds,
    /// Loose tile count is not `16 - 3 * melds.len()`.
    WrongTileCount,
    /// More than four copies, counting the opponent's pung when robbing a kong.
    TooManyCopies {
        /// Offending tile.
        tile: Tile,
    },
    /// Flower count is out of range or inconsistent with a flower-only win.
    InvalidFlowers,
    /// Eight flowers must be scored with `score_flowers`, without a tile hand.
    FlowerWinRequired,
    /// Contradictory win facts.
    InvalidContext {
        /// Explanation of the contradiction.
        reason: String,
    },
    /// Neither five sets and a pair nor seven pairs and a triplet.
    NotWinning,
}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNotation => {
                f.write_str("invalid MPSZ notation: parse_hand accepts [exposed sets], {concealed kongs}, and 0f..8f flower counts; parse_tile requires one tile")
            }
            Self::InvalidRank => f.write_str("tile rank must be 1..=9"),
            Self::InvalidChow => f.write_str("a chow must start at a suited tile ranked 1..=7"),
            Self::TooManyMelds => f.write_str("at most five declared melds are allowed"),
            Self::WrongTileCount => {
                f.write_str("expected 16 - 3 * meld count loose tiles, excluding the winning tile")
            }
            Self::TooManyCopies { tile } => write!(f, "more than four copies of {tile:?}"),
            Self::InvalidFlowers => {
                f.write_str("flowers must be 0..=8; a flower-only win requires eight held, or seven and robbing the eighth")
            }
            Self::FlowerWinRequired => f.write_str("use score_flowers for an eight-flower win"),
            Self::InvalidContext { reason } => f.write_str(reason),
            Self::NotWinning => f.write_str("tiles do not form a winning hand"),
        }
    }
}

impl std::error::Error for ScoreError {}
