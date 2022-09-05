use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    iter, mem,
};

use cached::proc_macro::cached;
use petgraph::{prelude::GraphMap, Directed};
use crate::types::{
    Agenda, Handicraft, HandicraftGraphNode, HandicraftName, HandicraftPricingInfo, PopSupply,
    RareItemCount,
};

const TIME_IN_CYCLE: usize = 24;
const MIN_PRODUCT_TIME: usize = 4;

// type MaterialGraph = GraphMap<MaterialGraphNode, u8, Directed>;
type HandicraftGraph = GraphMap<HandicraftGraphNode, u8, Directed>;

// fn create_material_graph<'a, I>(handicrafts: I) -> (HashSet<HandicraftName>, MaterialGraph)
// where
//     I: Iterator<Item = &'a Handicraft>,
// {
//     let mut graph = GraphMap::new();
//     let recipe_nodes = handicrafts
//         .map(|item| {
//             for mat in &item.materials {
//                 let recipe_node = graph.add_node(MaterialGraphNode::Handicraft(item.name));
//                 graph.add_edge(
//                     recipe_node,
//                     MaterialGraphNode::Material(*mat.0),
//                     u8::default(),
//                 );
//             }
//             item.name
//         })
//         .collect();
//     (recipe_nodes, graph)
// }

pub fn create_handicraft_graph<'a, I>(handicrafts: I) -> (HashSet<HandicraftName>, HandicraftGraph)
where
    I: Iterator<Item = &'a Handicraft>,
{
    let mut graph = GraphMap::new();
    let recipe_nodes = handicrafts
        .map(|item| {
            for cat in &item.category {
                let recipe_node = graph.add_node(HandicraftGraphNode::Handicraft(item.name));
                graph.add_edge(
                    recipe_node,
                    HandicraftGraphNode::Category(*cat),
                    u8::default(),
                );
            }
            item.name
        })
        .collect();

    (recipe_nodes, graph)
}

fn remove_unmakeable_recipes(
    handicrafts: &Vec<Handicraft>,
    recipe_nodes: &mut HashSet<HandicraftName>,
    rare_item_counts: Vec<RareItemCount>,
    handicraft_graph: &mut HandicraftGraph,
) {
    let unusable_items: HashSet<_> = rare_item_counts
        .into_iter()
        .filter(|item| item.count == 0)
        .map(|item| *item.name())
        .collect();
    for h in handicrafts {
        if h.materials.keys().any(|mat| unusable_items.contains(mat)) {
            let node = recipe_nodes.take(&h.name).unwrap_or_else(|| {
                panic!(
                    "Rare item node was connected to non-existant recipe ({})",
                    h.name
                )
            });
            handicraft_graph.remove_node(HandicraftGraphNode::Handicraft(node));
        }
    }
}

pub fn find_agendas(
    handicrafts: &Vec<Handicraft>,
    handicraft_pop_supply: HashMap<HandicraftName, PopSupply>,
    rare_item_counts: Vec<RareItemCount>,
    mut recipe_nodes: HashSet<HandicraftName>,
    mut handicraft_graph: HandicraftGraph,
    handicraft_pricing_info: HashMap<HandicraftName, HandicraftPricingInfo>,
) -> BinaryHeap<Agenda> {
    remove_unmakeable_recipes(
        handicrafts,
        &mut recipe_nodes,
        rare_item_counts,
        &mut handicraft_graph,
    );

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
            .neighbors(HandicraftGraphNode::Handicraft(*current))
            .flat_map(|category| {
                handicraft_graph.neighbors_directed(category, petgraph::Direction::Incoming)
            })
            .map(|recipe| recipe.unwrap_handicraft())
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
                    let elapsed = elapsed + pricing_info.time;
                    new_agenda.push(recipe);
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
