use std::{collections::HashMap, fmt::Display, str::FromStr};

use anyhow::anyhow;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct Category(pub String);

#[derive(Deserialize, Debug, Clone)]
pub struct Handicraft {
    pub name: String,
    pub time: usize,
    pub quantity: usize,
    pub value: usize,
    pub category: Vec<Category>,
    pub materials: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub enum Popularity {
    Low,
    Average,
    High,
    VeryHigh,
}

impl Popularity {
    pub fn multiplier(&self) -> usize {
        match self {
            Popularity::Low => 0,
            Popularity::Average => 1,
            Popularity::High => 2,
            Popularity::VeryHigh => 3,
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

#[derive(Debug, Clone)]
pub enum Supply {
    Nonexistent,
    Insufficient,
    Sufficient,
    Surplus,
}

impl Supply {
    pub fn multiplier(&self) -> usize {
        match self {
            Supply::Nonexistent => 3,
            Supply::Insufficient => 2,
            Supply::Sufficient => 1,
            Supply::Surplus => 0,
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

#[derive(Debug, Clone)]
pub struct Agenda(pub Vec<Handicraft>);
