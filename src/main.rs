use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    vec,
};

use serde::{Deserialize, Serialize};

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct Args {
    /// Name of the breed start parent
    #[arg(short, long)]
    parent: String,

    /// Name of the breed target
    #[arg(short, long)]
    target: String,
}

fn main() {
    let args = Args::parse();
    let c = reqwest::blocking::Client::new();
    let mut breeds = vec![];
    // try to get all breeds from breeds.json
    if let Ok(mut f) = std::fs::File::open("breeds.json") {
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        breeds = serde_json::from_str(&s).unwrap();
    } else {
        let mut i = 1;
        loop {
            let re = c
                .get("https://palworld.caimogu.cc/breed")
                .header("x-requested-with", "XMLHttpRequest")
                .query(&[("seed1", 0), ("seed2", 0), ("result", 0), ("page", i)])
                .send()
                .unwrap();
            i = i + 1;
            let resp: Resp = re.json().unwrap();
            breeds.extend(resp.data.list);
            if resp.data.has_more != 1 {
                break;
            }
        }
        // write json to path
        let mut f = std::fs::File::create("breeds.json").unwrap();
        f.write_all(serde_json::to_string_pretty(&breeds).unwrap().as_bytes())
            .unwrap();
    }

    // build namemap
    let namemap = breeds
        .iter()
        .map(|b| {
            vec![
                (b.s1_name.clone(), b.s1),
                (b.s2_name.clone(), b.s2),
                (b.ret_name.clone(), b.ret),
            ]
        })
        .flatten()
        .collect::<HashMap<_, _>>();

    // build graph
    let graph = breeds
        .iter()
        .map(|b| vec![(b.s1, (b.s2, b.ret)), (b.s2, (b.s1, b.ret))])
        .flatten()
        .fold(HashMap::new(), |mut acc, (k, v)| {
            acc.entry(k).or_insert(vec![]).push(v);
            acc
        });

    let parent1 = namemap.get(&args.parent).expect("parent not found");

    let target = namemap.get(&args.target).expect("target not found");

    // djstra
    // the graph's value is (another parent, child)
    let mut queue = vec![*parent1];
    let mut visited = HashSet::new();
    let mut path = HashMap::new();
    while let Some(node) = queue.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);
        if node == *target {
            break;
        }
        if let Some(children) = graph.get(&node) {
            for (parent, child) in children {
                if !visited.contains(child) {
                    queue.push(*child);
                    if path.contains_key(child) {
                        continue;
                    }
                    path.insert(*child, (node, *parent));
                    if *child == *target {
                        queue.clear();
                        break;
                    }
                }
            }
        }
    }

    let mut child = *target;
    let mut res = vec![];
    while let Some((p1, p2)) = path.get(&child) {
        res.push((child, *p2, *p1));
        child = *p1;
    }
    if res.is_empty() {
        eprintln!("该配种不可能完成");
    }
    res.reverse();
    let reverse_name_map = namemap
        .iter()
        .map(|(k, v)| (v, k))
        .collect::<HashMap<_, _>>();
    // print names
    for (c, p1, p2) in res {
        println!(
            "{} + {} = {}",
            reverse_name_map.get(&p1).unwrap(),
            reverse_name_map.get(&p2).unwrap(),
            reverse_name_map.get(&c).unwrap()
        );
    }
}

#[derive(Serialize, Deserialize, Default)]
struct PalBreed {
    ret: u32,
    ret_name: String,
    s1: u32,
    s2: u32,
    s1_name: String,
    s2_name: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Resp {
    data: Data,
    info: String,
    status: i32,
}
#[derive(Serialize, Deserialize, Default)]
struct Data {
    #[serde(rename = "hasMore")]
    has_more: i32,
    list: Vec<PalBreed>,
}
