use std::io::{self, Write};

use mji_agenda::{create_handicraft_graph, find_agendas, RareItemVariant, WorkshopData};
use mji_agenda::{Handicraft, PopSupply, RareItemCount};

fn main() {
    let raw = include_bytes!("handicrafts.toml");
    // let raw = fs::read_to_string("src/handicrafts.toml").unwrap();
    let data: WorkshopData = String::from_utf8_lossy(raw)
        .parse::<toml::Value>()
        .unwrap()
        .try_into()
        .unwrap();

    // useful for mapping material connections, less useful for making agendas with efficiency bonus
    // let (recipe_nodes, handicraft_graph) = create_material_graph(data.handicrafts.iter());
    let (recipe_nodes, handicraft_graph) = create_handicraft_graph(data.handicrafts.iter());

    println!("Input amount of rare items in Isleventory");

    let mut stdin = io::stdin();
    let mut input_buf = String::new();
    let rare_item_counts: Vec<_> = data
        .rare
        .material
        .into_iter()
        .map(RareItemVariant::WithArea)
        .chain(data.rare.produce.into_iter().map(RareItemVariant::RareItem))
        .chain(
            data.rare
                .leavings
                .into_iter()
                .map(RareItemVariant::RareItem),
        )
        .map(|item| input_rare_item_count(&stdin, &mut input_buf, item))
        .collect();
    println!("rare_item_counts: {:?}", rare_item_counts);

    println!("Input popularity (L = Low, A = Average, H = High, V = Very High) and supply (N = Nonexistent, I = Insufficient, S = Sufficient, U = Surplus) for products");
    let handicraft_pop_supply = data
        .handicrafts
        .iter()
        .map(|item| {
            (
                item.name,
                input_product_pop_supply(&mut stdin, &mut input_buf, item),
            )
        })
        .collect();
    println!("handicraft_pop_supply: {:?}", handicraft_pop_supply);

    let agendas = find_agendas(
        &data.handicrafts,
        handicraft_pop_supply,
        rare_item_counts,
        recipe_nodes,
        handicraft_graph,
        data.handicrafts
            .iter()
            .map(|handicraft| (handicraft.name, handicraft.as_pricing_info()))
            .collect(),
    );

    println!("Outputting top 5 producing agendas");

    for agenda in agendas {
        print!("[{}]", agenda.total_value);
        let mut it = agenda
            .handicrafts
            .iter()
            .zip(agenda.values.iter())
            .peekable();
        while let Some((handicraft, value)) = it.next() {
            print!("{} ({})", handicraft, value);
            if it.peek().is_some() {
                print!(" -> ");
            }
        }
        println!();
    }
}

fn input_rare_item_count(
    stdin: &io::Stdin,
    input_buf: &mut String,
    rare: RareItemVariant,
) -> RareItemCount {
    print!("{}: ", rare.name());
    io::stdout().flush().unwrap();
    stdin
        .read_line(input_buf)
        .expect("Tried reading user input for rare item count");
    let count = input_buf
        .trim()
        .parse()
        .expect("Must be an unsigned integer");
    input_buf.clear();
    RareItemCount { rare, count }
}

fn input_product_pop_supply(
    stdin: &mut io::Stdin,
    input_buf: &mut String,
    handicraft: &Handicraft,
) -> PopSupply {
    print!("{} popularity: ", handicraft.name);
    io::stdout().flush().unwrap();
    stdin
        .read_line(input_buf)
        .expect("Tried reading user input for product popularity");
    let popularity = input_buf.trim().parse().expect("Must be a valid character");
    input_buf.clear();
    print!("{} supply: ", handicraft.name);
    io::stdout().flush().unwrap();
    stdin
        .read_line(input_buf)
        .expect("Tried reading user input for product supply");
    let supply = input_buf.trim().parse().expect("Must be a valid character");
    input_buf.clear();
    PopSupply { popularity, supply }
}
