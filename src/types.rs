use std::{collections::HashMap, str::FromStr};

use derive_more::Unwrap;
use serde::Deserialize;
use thiserror::Error;

#[derive(
    Deserialize, Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, strum_macros::Display,
)]
#[strum(serialize_all = "title_case")]
pub enum HandicraftName {
    IsleworksPotion,
    IsleworksFiresand,
    IsleworksWoodenChair,
    IsleworksGrilledClam,
    IsleworksNecklace,
    IsleworksCoralRing,
    IsleworksBarbut,
    IsleworksMacuahuitl,
    IsleworksSauerkraut,
    IsleworksBakedPumpkin,
    IsleworksTunic,
    IsleworksCulinaryKnife,
    IsleworksBrush,
    IsleworksBoiledEgg,
    IsleworksHora,
    IsleworksEarrings,
    IsleworksButter,
    IsleworksBrickCounter,
    BronzeSheep,
    IsleworksGrowthFormula,
    IsleworksGarnetRapier,
    IsleworksSpruceRoundShield,
    IsleworksSharkOil,
    IsleworksSilverEarCuffs,
    IsleworksSweetPopoto,
    IsleworksParsnipSalad,
    IsleworksCaramels,
    IsleworksRibbon,
    IsleworksRope,
    IsleworksCavaliersHat,
    IsleworksHorn,
    IsleworksSaltCod,
    IsleworksSquidInk,
    IsleworksEssentialDraught,
    IsleberryJam,
    IsleworksTomatoRelish,
    IsleworksOnionSoup,
    IslefishPie,
    IsleworksCornFlakes,
    IsleworksPickledRadish,
    IsleworksIronAxe,
    IsleworksQuartzRing,
    IsleworksPorcelainVase,
    IsleworksVegetableJuice,
    IsleworksPumpkinPudding,
    IsleworksSheepfluffRug,
    IsleworksGardenScythe,
    IsleworksBed,
    IsleworksScaleFingers,
    IsleworksCrook,
}

#[derive(
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Hash,
    strum_macros::Display,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "title_case")]
pub enum MaterialName {
    IslandAlyssum,
    IslandApple,
    IslandBranch,
    IslandCabbage,
    IslandClam,
    IslandClay,
    IslandCopperOre,
    IslandCoral,
    IslandCorn,
    IslandCottonBoll,
    IslandHammerhead,
    IslandHemp,
    IslandJellyfish,
    IslandLaver,
    IslandLimestone,
    IslandLog,
    IslandOnion,
    IslandPalmLeaf,
    IslandPalmLog,
    IslandParsnip,
    IslandPopoto,
    IslandPumpkin,
    IslandRadish,
    IslandRockSalt,
    IslandSand,
    IslandSap,
    IslandSilverOre,
    IslandSpruceLog,
    IslandSquid,
    IslandStone,
    IslandSugarcane,
    IslandTinsand,
    IslandIronOre,
    IslandQuartz,
    IslandLeucogranite,
    IslandTomato,
    IslandVine,
    IslandWheat,
    Isleberry,
    Islefish,
    Islewort,
    RawIslandGarnet,
    SanctuaryCarapace,
    SanctuaryClaw,
    SanctuaryEgg,
    SanctuaryFang,
    SanctuaryFeather,
    SanctuaryFleece,
    SanctuaryFur,
    SanctuaryHorn,
    SanctuaryMilk,
}
#[derive(
    Deserialize, Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, strum_macros::Display,
)]
#[strum(serialize_all = "title_case")]
pub enum CategoryName {
    Accessories,
    Arms,
    Attire,
    Concoctions,
    Confections,
    CreatureCreations,
    Foodstuffs,
    Furnishings,
    Ingredients,
    MarineMerchandise,
    Metalworks,
    PreservedFood,
    Sundries,
    Textiles,
    UnburiedTreasures,
    Woodworks,
}

// for the graph
// #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Unwrap)]
// pub enum MaterialGraphNode {
//     Handicraft(HandicraftName),
//     Material(MaterialName),
// }

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Unwrap)]
pub enum HandicraftGraphNode {
    Handicraft(HandicraftName),
    Category(CategoryName),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Handicraft {
    pub name: HandicraftName,
    pub time: usize,
    pub quantity: usize,
    pub value: usize,
    pub category: Vec<CategoryName>,
    pub materials: HashMap<MaterialName, usize>,
}

impl Handicraft {
    pub fn as_pricing_info(&self) -> HandicraftPricingInfo {
        HandicraftPricingInfo {
            time: self.time,
            quantity: self.quantity,
            value: self.value,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct HandicraftPricingInfo {
    pub time: usize,
    pub quantity: usize,
    pub value: usize,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Popularity {
    Low,
    Average,
    High,
    VeryHigh,
}

impl Popularity {
    pub fn multiplier(&self) -> f64 {
        match self {
            Popularity::Low => 0.8,
            Popularity::Average => 1.0,
            Popularity::High => 1.2,
            Popularity::VeryHigh => 1.4,
        }
    }
}

#[derive(Error, Debug)]
#[error("Invalid value for Popularity: {0}")]
pub struct PopularityDeserializeError(String);

impl FromStr for Popularity {
    type Err = PopularityDeserializeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Popularity::*;
        match s.to_lowercase().as_str() {
            "l" => Ok(Low),
            "a" => Ok(Average),
            "h" => Ok(High),
            "v" => Ok(VeryHigh),
            _ => Err(PopularityDeserializeError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Supply {
    Nonexistent,
    Insufficient,
    Sufficient,
    Surplus,
    Overflowing,
}

impl Supply {
    pub fn multiplier(&self) -> f64 {
        match self {
            Supply::Nonexistent => 1.6,
            Supply::Insufficient => 1.3,
            Supply::Sufficient => 1.0,
            Supply::Surplus => 0.8,
            Supply::Overflowing => 0.6,
        }
    }
}

#[derive(Error, Debug)]
#[error("Invalid value for Supply: {0}")]
pub struct SupplyDeserializeError(String);

impl FromStr for Supply {
    type Err = SupplyDeserializeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Supply::*;
        match s.to_lowercase().as_str() {
            "n" => Ok(Nonexistent),
            "i" => Ok(Insufficient),
            "s" => Ok(Sufficient),
            "u" => Ok(Surplus),
            "o" => Ok(Overflowing),
            _ => Err(SupplyDeserializeError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PopSupply {
    pub popularity: Popularity,
    pub supply: Supply,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RareItem {
    pub name: MaterialName,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RareItemWithArea {
    pub name: MaterialName,
    pub area: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RareItems {
    pub produce: Vec<RareItem>,
    pub material: Vec<RareItemWithArea>,
    pub leavings: Vec<RareItem>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorkshopData {
    pub handicrafts: Vec<Handicraft>,
    pub rare: RareItems,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RareItemVariant {
    RareItem(RareItem),
    WithArea(RareItemWithArea),
}

impl RareItemVariant {
    pub fn name(&self) -> &MaterialName {
        match self {
            RareItemVariant::RareItem(i) => &i.name,
            RareItemVariant::WithArea(i) => &i.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RareItemCount {
    pub rare: RareItemVariant,
    pub count: usize,
}

impl RareItemCount {
    pub fn name(&self) -> &MaterialName {
        self.rare.name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agenda {
    pub handicrafts: Vec<HandicraftName>,
    pub values: Vec<usize>,
    pub total_value: usize,
}

impl PartialOrd for Agenda {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Agenda {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_value.cmp(&other.total_value)
    }
}
