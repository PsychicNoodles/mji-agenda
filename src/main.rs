mod types;

use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    fs,
    io::{self, Write},
    iter, mem,
};

use cached::proc_macro::cached;
use petgraph::{prelude::GraphMap, Directed};
use types::{
    Agenda, DataFile, Handicraft, HandicraftComponent, HandicraftName, HandicraftPricingInfo,
    PopSupply, RareItemCount, RareItemVariant,
};

const TIME_IN_CYCLE: usize = 24;
const MIN_PRODUCT_TIME: usize = 4;

type HandicraftGraph = GraphMap<HandicraftComponent, u8, Directed>;

fn main() {
    let raw = fs::read_to_string("handicrafts.toml").unwrap();
    let data: DataFile = raw.parse::<toml::Value>().unwrap().try_into().unwrap();

    let (recipe_nodes, handicraft_graph) = create_material_graph(data.handicrafts.iter());

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

fn create_material_graph<'a, I>(handicrafts: I) -> (HashSet<HandicraftName>, HandicraftGraph)
where
    I: Iterator<Item = &'a Handicraft>,
{
    let mut graph = GraphMap::new();
    let recipe_nodes = handicrafts
        .map(|item| {
            for mat in &item.materials {
                let material_node = graph.add_node(HandicraftComponent::Handicraft(item.name));
                graph.add_edge(
                    material_node,
                    HandicraftComponent::Material(*mat.0),
                    u8::default(),
                );
            }
            item.name
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

fn remove_unmakeable_recipes(
    recipe_nodes: &mut HashSet<HandicraftName>,
    rare_item_counts: Vec<RareItemCount>,
    handicraft_graph: &mut HandicraftGraph,
) {
    // must finish immutable borrow before removing nodes
    let unusable_recipes: HashSet<_> = rare_item_counts
        .into_iter()
        .filter(|item| item.count == 0)
        .flat_map(|item| {
            handicraft_graph
                .neighbors_directed(
                    HandicraftComponent::Material(*item.name()),
                    petgraph::Direction::Incoming,
                )
                .collect::<HashSet<_>>()
        })
        .collect();
    for recipe in unusable_recipes {
        let handicraft_name = recipe.try_into().unwrap_or_else(|_| {
            panic!(
                "Rare item node was connected to another material ({:?})",
                recipe
            )
        });
        let node = recipe_nodes.take(&handicraft_name).unwrap_or_else(|| {
            panic!(
                "Rare item node was connected to non-existant recipe ({})",
                handicraft_name
            )
        });
        handicraft_graph.remove_node(HandicraftComponent::Handicraft(node));
    }
}

fn find_agendas(
    handicraft_pop_supply: HashMap<HandicraftName, PopSupply>,
    rare_item_counts: Vec<RareItemCount>,
    mut recipe_nodes: HashSet<HandicraftName>,
    mut handicraft_graph: HandicraftGraph,
    handicraft_pricing_info: HashMap<HandicraftName, HandicraftPricingInfo>,
) -> BinaryHeap<Agenda> {
    remove_unmakeable_recipes(&mut recipe_nodes, rare_item_counts, &mut handicraft_graph);

    recipe_nodes
        .iter()
        .map(|start| {
            // max potential number of products per cycle (24 / 4)
            let mut agenda = Vec::with_capacity(11);
            agenda.push(*start);
            generate_agendas(
                &handicraft_graph,
                &handicraft_pricing_info,
                agenda,
                handicraft_pricing_info
                    .get(start)
                    .unwrap_or_else(|| {
                        panic!("Could not find pricing info for handicraft {:?}", start)
                    })
                    .time,
            )
        })
        .flat_map(IntoIterator::into_iter)
        .map(|products| calc_agenda(products, &handicraft_pop_supply, &handicraft_pricing_info))
        .collect()
}

#[derive(Debug)]
enum AgendaGeneratorResult {
    Tail(Vec<HandicraftName>),
    Intermediate(Vec<AgendaGeneratorResult>),
}

impl IntoIterator for AgendaGeneratorResult {
    type Item = Vec<HandicraftName>;

    type IntoIter = AgendaGeneratorResultIterator;

    fn into_iter(self) -> Self::IntoIter {
        AgendaGeneratorResultIterator {
            children: vec![self],
            parent: None,
        }
    }
}

// https://aloso.github.io/2021/03/09/creating-an-iterator
#[derive(Debug, Default)]
struct AgendaGeneratorResultIterator {
    children: Vec<AgendaGeneratorResult>,
    parent: Option<Box<AgendaGeneratorResultIterator>>,
}

impl Iterator for AgendaGeneratorResultIterator {
    type Item = Vec<HandicraftName>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.children.pop() {
            Some(AgendaGeneratorResult::Tail(m)) => Some(m),
            Some(AgendaGeneratorResult::Intermediate(i)) => {
                *self = AgendaGeneratorResultIterator {
                    children: i,
                    parent: Some(Box::new(mem::take(self))),
                };
                self.next()
            }
            None => match self.parent.take() {
                Some(p) => {
                    *self = *p;
                    self.next()
                }
                None => None,
            },
        }
    }
}

fn generate_agendas(
    handicraft_graph: &HandicraftGraph,
    handicraft_pricing_info: &HashMap<HandicraftName, HandicraftPricingInfo>,
    agenda: Vec<HandicraftName>,
    elapsed: usize,
) -> AgendaGeneratorResult {
    // can't fit anything else in agenda
    if elapsed > (TIME_IN_CYCLE - MIN_PRODUCT_TIME) {
        AgendaGeneratorResult::Tail(agenda)
    } else {
        let current = agenda.last().expect("Agenda is empty");
        let candidates = handicraft_graph
            .neighbors(HandicraftComponent::Handicraft(*current))
            .flat_map(|neighbor| {
                handicraft_graph.neighbors_directed(neighbor, petgraph::Direction::Incoming)
            })
            .map(|recipe| -> HandicraftName {
                recipe
                    .try_into()
                    .unwrap_or_else(|_| panic!("Material pointed towards material ({:?})", recipe))
            })
            .filter(|recipe| current != recipe)
            .map(|recipe| {
                (
                    recipe,
                    handicraft_pricing_info.get(&recipe).unwrap_or_else(|| {
                        panic!("Could not find pricing info for handicraft {:?}", recipe)
                    }),
                )
            })
            .filter(|(_, pricing_info)| pricing_info.time + elapsed <= TIME_IN_CYCLE);
        AgendaGeneratorResult::Intermediate(
            candidates
                .map(|(recipe, pricing_info)| {
                    let mut new_agenda = agenda.clone();
                    let new_handicraft: HandicraftName = recipe.try_into().unwrap_or_else(|_| {
                        panic!("Tried adding material to agenda ({:?})", recipe)
                    });
                    let elapsed = elapsed + pricing_info.time;
                    new_agenda.push(new_handicraft);
                    generate_agendas(
                        handicraft_graph,
                        handicraft_pricing_info,
                        new_agenda,
                        elapsed,
                    )
                })
                .collect(),
        )
    }
}

fn calc_agenda(
    agenda: Vec<HandicraftName>,
    handicraft_pop_supplies: &HashMap<HandicraftName, PopSupply>,
    handicraft_pricing_info: &HashMap<HandicraftName, HandicraftPricingInfo>,
) -> Agenda {
    let pricing: Vec<_> = agenda
        .iter()
        .zip(iter::once(false).chain(iter::repeat(true)))
        .map(|(handicraft, efficiency_bonus)| {
            calc_abs_pricing(
                efficiency_bonus,
                *handicraft_pop_supplies.get(handicraft).unwrap_or_else(|| {
                    panic!(
                        "Agenda had handicraft without popularity/supply ({})",
                        handicraft
                    )
                }),
                *handicraft_pricing_info.get(handicraft).unwrap_or_else(|| {
                    panic!(
                        "Agenda had handicraft without pricing info ({})",
                        handicraft
                    )
                }),
            )
        })
        .collect();
    Agenda {
        handicrafts: agenda,
        total_value: pricing.iter().sum(),
        values: pricing,
    }
}

// full pricing formula uses workshop level and groove, which is TODO
// for now, just the basic formula
#[cached()]
fn calc_abs_pricing(
    efficiency_bonus: bool,
    pop_supply: PopSupply,
    handicraft: HandicraftPricingInfo,
) -> usize {
    (if efficiency_bonus { 2 } else { 1 })
        * (pop_supply.popularity.multiplier()
            * pop_supply.supply.multiplier()
            * (handicraft.value as f64).floor())
        .floor() as usize
}
