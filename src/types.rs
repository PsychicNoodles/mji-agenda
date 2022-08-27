use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

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

#[derive(Deserialize, Debug, Clone)]
pub struct RareItem {
    pub name: String,
}

impl Display for RareItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RareItemWithArea {
    pub name: String,
    pub area: String,
}

impl Display for RareItemWithArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
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

impl Display for RareItemVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RareItemVariant::RareItem(i) => i.fmt(f),
            RareItemVariant::WithArea(i) => i.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RareItemCount {
    pub rare: RareItemVariant,
    pub count: usize,
}
