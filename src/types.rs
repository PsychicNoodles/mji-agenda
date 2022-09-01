use std::{collections::HashMap, fmt::Display, str::FromStr};

use anyhow::anyhow;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct Category(pub String);

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct HandicraftName<'a>(pub &'a str);

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct MaterialName<'a>(pub &'a str);

// for the graph
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum HandicraftComponent<'a> {
    Handicraft(HandicraftName<'a>),
    Material(MaterialName<'a>),
}

// todo make this more succinct

#[derive(Debug, Error)]
#[error("Expected handicraft but was not")]
pub struct HandicraftComponentNotHandicraft;

impl<'a> TryFrom<&HandicraftComponent<'a>> for HandicraftName<'a> {
    type Error = HandicraftComponentNotHandicraft;

    fn try_from(value: &HandicraftComponent<'a>) -> Result<Self, Self::Error> {
        match value {
            HandicraftComponent::Handicraft(h) => Ok(*h),
            HandicraftComponent::Material(_) => Err(Self::Error {}),
        }
    }
}

impl<'a> TryFrom<&HandicraftComponent<'a>> for MaterialName<'a> {
    type Error = HandicraftComponentNotMaterial;

    fn try_from(value: &HandicraftComponent<'a>) -> Result<Self, Self::Error> {
        match value {
            HandicraftComponent::Handicraft(_) => Err(Self::Error {}),
            HandicraftComponent::Material(m) => Ok(*m),
        }
    }
}

impl<'a> TryFrom<HandicraftComponent<'a>> for HandicraftName<'a> {
    type Error = HandicraftComponentNotHandicraft;

    fn try_from(value: HandicraftComponent<'a>) -> Result<Self, Self::Error> {
        match value {
            HandicraftComponent::Handicraft(h) => Ok(h),
            HandicraftComponent::Material(_) => Err(Self::Error {}),
        }
    }
}

#[derive(Debug, Error)]
#[error("Expected material but was not")]
pub struct HandicraftComponentNotMaterial;

impl<'a> TryFrom<HandicraftComponent<'a>> for MaterialName<'a> {
    type Error = HandicraftComponentNotMaterial;

    fn try_from(value: HandicraftComponent<'a>) -> Result<Self, Self::Error> {
        match value {
            HandicraftComponent::Handicraft(_) => Err(Self::Error {}),
            HandicraftComponent::Material(m) => Ok(m),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Handicraft {
    pub name: String,
    pub time: usize,
    pub quantity: usize,
    pub value: usize,
    pub category: Vec<Category>,
    pub materials: HashMap<String, usize>,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct HandicraftPopSupply {
    pub handicraft: Handicraft,
    pub popularity: Popularity,
    pub supply: Supply,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RareItem {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RareItemWithArea {
    pub name: String,
    pub area: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RareItems {
    pub produce: Vec<RareItem>,
    pub material: Vec<RareItemWithArea>,
    pub leavings: Vec<RareItem>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DataFile {
    pub handicrafts: Vec<Handicraft>,
    pub rare: RareItems,
}

#[derive(Debug, Clone)]
pub enum RareItemVariant {
    RareItem(RareItem),
    WithArea(RareItemWithArea),
}

impl RareItemVariant {
    pub fn name(&self) -> &str {
        match self {
            RareItemVariant::RareItem(i) => &i.name,
            RareItemVariant::WithArea(i) => &i.name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RareItemCount {
    pub rare: RareItemVariant,
    pub count: usize,
}

impl RareItemCount {
    pub fn name(&self) -> &str {
        self.rare.name()
    }
}

#[derive(Debug, Clone)]
pub struct Agenda(pub Vec<String>);
