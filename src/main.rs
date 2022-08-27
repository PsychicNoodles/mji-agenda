mod types;

use std::{
    fs,
    io::{self, Read},
};

use types::{Agenda, DataFile, Handicraft, HandicraftPopSupply, RareItemCount, RareItemVariant};

const TIME_IN_CYCLE: usize = 24;

fn main() {
    let raw = fs::read_to_string("handicrafts.toml").unwrap();
    let data: DataFile = toml::from_str(&raw).unwrap();

    println!("Input amount of rare items in Isleventory");

    let mut stdin = io::stdin();
    let mut input_buf = String::new();
    let rare_item_counts: Vec<_> = data
        .rare
        .produce
        .into_iter()
        .map(RareItemVariant::RareItem)
        .chain(
            data.rare
                .material
                .into_iter()
                .map(RareItemVariant::WithArea),
        )
        .chain(
            data.rare
                .leavings
                .into_iter()
                .map(RareItemVariant::RareItem),
        )
        .map(|item| input_rare_item_count(&stdin, &mut input_buf, item))
        .collect();

    println!("Input popularity (L = Low, A = Average, H = High, V = Very High) and supply (N = Nonexistent, I = Insufficient, S = Sufficient, U = Surplus) for products");
    let handicraft_pop_supplys: Vec<_> = data
        .handicrafts
        .into_iter()
        .map(|item| input_product_pop_supply(&mut stdin, item))
        .collect();
}

fn input_rare_item_count(
    stdin: &io::Stdin,
    input_buf: &mut String,
    rare: RareItemVariant,
) -> RareItemCount {
    print!("{}: ", rare.name());
    stdin
        .read_line(input_buf)
        .expect("Tried reading user input for rare item count");
    let count = input_buf.parse().expect("Must be an unsigned integer");
    println!();
    input_buf.clear();
    RareItemCount { rare, count }
}

fn input_product_pop_supply(stdin: &mut io::Stdin, handicraft: Handicraft) -> HandicraftPopSupply {
    let mut input_buf = [0; 1];
    print!("{} popularity: ", handicraft.name);
    stdin
        .read_exact(&mut input_buf)
        .expect("Tried reading user input for product popularity");
    let popularity = String::from_utf8_lossy(&input_buf)
        .parse()
        .expect("Must be a valid character");
    println!();
    print!("{} supply: ", handicraft.name);
    stdin
        .read_exact(&mut input_buf)
        .expect("Tried reading user input for product supply");
    let supply = String::from_utf8_lossy(&input_buf)
        .parse()
        .expect("Must be a valid character");
    println!();
    HandicraftPopSupply {
        handicraft,
        popularity,
        supply,
    }
}

fn find_agendas(
    handicraft_pop_supply: Vec<HandicraftPopSupply>,
    rare_item_counts: Vec<RareItemCount>,
) -> Vec<Agenda> {
    let usable_rare_items: Vec<_> = rare_item_counts
        .into_iter()
        .filter(|item| item.count > 0)
        .collect();
}
