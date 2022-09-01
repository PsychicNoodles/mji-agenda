mod types;

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Read},
};

use cached::proc_macro::cached;
use petgraph::{prelude::GraphMap, Directed};
use types::{
    Agenda, DataFile, Handicraft, HandicraftComponent, HandicraftName, HandicraftPopSupply,
    HandicraftPricingInfo, MaterialName, Popularity, RareItemCount, RareItemVariant, Supply,
};

const TIME_IN_CYCLE: usize = 24;

type HandicraftGraph<'a> = GraphMap<HandicraftComponent<'a>, u8, Directed>;

fn main() {
    let raw = fs::read_to_string("handicrafts.toml").unwrap();
    let data: DataFile = toml::from_str(&raw).unwrap();

    let (recipe_nodes, handicraft_graph) = create_material_graph(data.handicrafts.iter());

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

fn create_material_graph<'a, I>(
    handicrafts: I,
) -> (Vec<HandicraftComponent<'a>>, HandicraftGraph<'a>)
where
    I: Iterator<Item = &'a Handicraft>,
{
    let mut graph = GraphMap::new();
    let recipe_nodes: Vec<_> = handicrafts
        .map(|item| {
            let recipe_node = graph.add_node(HandicraftComponent::Handicraft(HandicraftName(
                item.name.as_str(),
            )));
            for mat in &item.materials {
                graph.add_edge(
                    recipe_node,
                    HandicraftComponent::Material(MaterialName(mat.0.as_str())),
                    u8::default(),
                );
            }
            recipe_node
        })
        .collect();
    (recipe_nodes, graph)
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

fn remove_unmakeable_recipes<'a>(
    recipe_nodes: &'a mut HashSet<HandicraftName>,
    rare_item_counts: Vec<RareItemCount>,
    handicraft_graph: &mut HandicraftGraph<'a>,
) {
    // must finish immutable borrow before removing nodes
    let unusable_recipes: HashSet<_> = rare_item_counts
        .into_iter()
        .filter(|item| item.count == 0)
        .flat_map(|item| {
            handicraft_graph
                .neighbors_directed(
                    HandicraftComponent::Material(MaterialName(item.name().to_string().as_str())),
                    petgraph::Direction::Incoming,
                )
                .collect::<HashSet<_>>()
        })
        .collect();
    for recipe in unusable_recipes {
        let handicraft_name = match recipe {
            HandicraftComponent::Handicraft(h) => h,
            //todo
            HandicraftComponent::Material(m) => panic!(),
        };
        let node = recipe_nodes.take(&handicraft_name).expect(&format!(
            "Rare item node was connected to non-existant recipe ({})",
            handicraft_name.0
        ));
        handicraft_graph.remove_node(HandicraftComponent::Handicraft(node));
    }
}

fn find_agendas<'a>(
    handicraft_pop_supply: Vec<HandicraftPopSupply>,
    rare_item_counts: Vec<RareItemCount>,
    mut recipe_nodes: HashSet<HandicraftName>,
    mut handicraft_graph: HandicraftGraph<'a>,
    handicraft_pricing_info: HashMap<HandicraftName, HandicraftPricingInfo>,
) -> Vec<Agenda> {
    remove_unmakeable_recipes(&mut recipe_nodes, rare_item_counts, &mut handicraft_graph);

    let all_agendas = recipe_nodes.iter().map(|start| {
        // max number of products per cycle (24 / 4)
        let mut agenda = Vec::with_capacity(11);
        agenda.push(start.clone());
        generate_agendas(
            &handicraft_graph,
            &handicraft_pricing_info,
            agenda,
            handicraft_pricing_info
                .get(start)
                .expect(&format!(
                    "Could not find pricing info for handicraft {:?}",
                    start
                ))
                .time,
        )
    });

    todo!();
}

enum AgendaGeneratorResult<'a> {
    Tail(Vec<HandicraftName<'a>>),
    Intermediate(Box<Vec<AgendaGeneratorResult<'a>>>),
}

fn generate_agendas<'a>(
    handicraft_graph: &HandicraftGraph<'a>,
    handicraft_pricing_info: &HashMap<HandicraftName, HandicraftPricingInfo>,
    agenda: Vec<HandicraftName>,
    elapsed: usize,
) -> AgendaGeneratorResult<'a> {
    // can't fit anything else in agenda
    if elapsed >= 21 {
        AgendaGeneratorResult::Tail(agenda)
    } else {
        let current = agenda.last().expect("Agenda is empty");
        let candidates = handicraft_graph
            .neighbors(HandicraftComponent::Handicraft(*current))
            .flat_map(|neighbor| {
                handicraft_graph.neighbors_directed(neighbor, petgraph::Direction::Incoming)
            })
            .filter(|recipe| {
                !agenda.contains(
                    &recipe
                        .try_into()
                        .expect(&format!("Material pointed towards material ({:?})", recipe)),
                )
            });
        AgendaGeneratorResult::Intermediate(Box::new(
            candidates
                .map(|c| {
                    let mut new_agenda = agenda.clone();
                    new_agenda.push(
                        c.try_into()
                            .expect(&format!("Tried adding material to agenda ({:?})", c)),
                    );
                    generate_agendas(
                        handicraft_graph,
                        handicraft_pricing_info,
                        new_agenda,
                        elapsed
                            + handicraft_pricing_info
                                .get(
                                    &c.try_into().expect(&format!(
                                        "Tried adding material to agenda ({:?})",
                                        c
                                    )),
                                )
                                .expect(&format!(
                                    "Could not find pricing info for handicraft {:?}",
                                    c
                                ))
                                .time,
                    )
                })
                .collect(),
        ))
    }
}

// full pricing formula uses workshop level and groove, which is TODO
// for now, just the basic formula
#[cached()]
fn calc_abs_pricing(
    efficiency_bonus: bool,
    popularity: Popularity,
    supply: Supply,
    handicraft: HandicraftPricingInfo,
) -> f64 {
    (if efficiency_bonus { 2.0 } else { 1.0 })
        * (popularity.multiplier() * supply.multiplier() * (handicraft.value as f64).floor())
            .floor()
}
