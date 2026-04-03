use rand::RngExt;
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub const ALL: [Rarity; 4] = [
        Rarity::Common,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
        }
    }
}

pub struct FishCatch {
    pub name: String,
    pub rarity: Rarity,
}

const COMMON: &[&str] = &[
    "anchovy",
    "alewife",
    "bleak",
    "blowfish",
    "bluegill",
    "bowfin",
    "bream",
    "burbot",
    "butterfish",
    "capelin",
    "carp",
    "chub",
    "cisco",
    "crappie",
    "croaker",
    "dace",
    "darter",
    "drum",
    "filefish",
    "goby",
    "grunt",
    "gudgeon",
    "guppy",
    "ide",
    "killifish",
    "lamprey",
    "loach",
    "mahi",
    "minnow",
    "mudfish",
    "mullet",
    "pilchard",
    "pinfish",
    "pollock",
    "porgy",
    "remora",
    "roach",
    "rudd",
    "sardine",
    "sauger",
    "sculpin",
    "scup",
    "shad",
    "smelt",
    "sole",
    "sprat",
    "sunfish",
    "tench",
    "tilapia",
    "whiting",
];

const RARE: &[&str] = &[
    "bass",
    "bonefish",
    "bonito",
    "catfish",
    "clownfish",
    "cod",
    "dorado",
    "eel",
    "flounder",
    "gar",
    "grouper",
    "haddock",
    "halibut",
    "herring",
    "lionfish",
    "mackerel",
    "parrotfish",
    "perch",
    "pike",
    "puffer",
    "salmon",
    "seahorse",
    "snapper",
    "triggerfish",
    "trout",
    "tuna",
    "walleye",
    "wrasse",
];

const EPIC: &[&str] = &[
    "anglerfish",
    "barracuda",
    "hammerhead",
    "marlin",
    "moray",
    "piranha",
    "sailfish",
    "sawfish",
    "shark",
    "stingray",
    "sturgeon",
    "swordfish",
    "tarpon",
    "wahoo",
    "wolffish",
];

const LEGENDARY: &[&str] = &[
    "arapaima",
    "barreleye",
    "coelacanth",
    "megamouth",
    "mola",
    "oarfish",
    "paddlefish",
];

pub fn generate() -> FishCatch {
    let mut rng = rand::rng();
    let roll: f64 = rng.random();

    let (rarity, pool) = if roll < 0.03 {
        (Rarity::Legendary, LEGENDARY)
    } else if roll < 0.15 {
        (Rarity::Epic, EPIC)
    } else if roll < 0.40 {
        (Rarity::Rare, RARE)
    } else {
        (Rarity::Common, COMMON)
    };

    let name = pool.choose(&mut rng).unwrap().to_string();
    FishCatch { name, rarity }
}

pub fn fish_for_rarity(rarity: Rarity) -> &'static [&'static str] {
    match rarity {
        Rarity::Common => COMMON,
        Rarity::Rare => RARE,
        Rarity::Epic => EPIC,
        Rarity::Legendary => LEGENDARY,
    }
}

pub fn total_fish_count() -> usize {
    COMMON.len() + RARE.len() + EPIC.len() + LEGENDARY.len()
}
