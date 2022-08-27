mod types;

use std::{fs, io};

use types::{DataFile, RareItemCount, RareItemVariant};

fn main() {
    let raw = fs::read_to_string("handicrafts.toml").unwrap();
    let data: DataFile = toml::from_str(&raw).unwrap();

    println!("Input amount of rare items in Isleventory");
}

fn input_rare_item_counts(items: Vec<RareItemVariant>) -> Vec<RareItemCount> {
    let stdin = io::stdin();
    let mut input_buf = String::new();
    items
        .iter()
        .map(|item| {
            print!("{}: ", item);
            stdin.read_line(&mut input_buf);
            let count = input_buf.parse().expect("Must be an unsigned integer");
            RareItemCount {
                rare: item.clone(),
                count,
            }
        })
        .collect()
}
