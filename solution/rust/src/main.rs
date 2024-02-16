use std::collections::{HashMap, VecDeque};
use std::{env, fs};

use serde::Deserialize;

const MAX_WEIGHT: u32 = 4_000_000;

fn main() {
    let mut args = env::args();
    let file_path = args.nth(1).expect("a file path");
    let txs = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(file_path)
        .unwrap()
        .deserialize()
        .collect::<Result<Vec<Transaction>, csv::Error>>()
        .unwrap()
        .into_iter()
        .fold(HashMap::new(), |mut acc, tx| {
            acc.insert(tx.id.clone(), tx);
            acc
        });

    let mut packages = Vec::new();
    for (id, tx) in &txs {
        let mut total_fee = tx.fee;
        let mut total_weight = tx.weight;
        let mut package_txs = Vec::from([id.clone()]);

        let mut queue = VecDeque::new();
        if let Some(deps) = &tx.deps {
            let deps = deps.split(";").collect::<Vec<_>>();
            queue.extend(deps);
        }

        while let Some(txid) = queue.pop_front() {
            let dep_tx = txs.get(txid).unwrap();
            total_fee += dep_tx.fee;
            total_weight += dep_tx.weight;
            package_txs.push(txid.to_string());

            let Some(deps) = &dep_tx.deps else {
                continue;
            };
            for dep in deps.split(";") {
                if !package_txs.contains(&dep.to_string()) && !queue.contains(&dep) {
                    queue.push_back(dep);
                }
            }
        }

        package_txs.reverse();
        packages.push(Package {
            fee: total_fee,
            weight: total_weight,
            txs: package_txs,
        });
    }

    packages.sort_by(|a, b| b.fee.cmp(&a.fee));

    let mut block = Vec::new();
    let mut total_weight = 0;
    let mut total_fees = 0;
    for package in packages {
        if total_weight + package.weight > MAX_WEIGHT || txs_in_block(&package.txs, &block) {
            continue;
        }
        total_fees += package.fee;
        total_weight += package.weight;
        block.extend(package.txs);
    }

    let contents = block.join("\n");
    fs::write("block.txt", &contents).unwrap();

    println!("total weight: {}", total_weight);
    println!("total fees: {}", total_fees);
}

fn txs_in_block(txs: &Vec<String>, block: &Vec<String>) -> bool {
    for tx in txs {
        if block.contains(tx) {
            return true;
        }
    }
    return false;
}

#[derive(Debug, Deserialize)]
struct Transaction {
    id: String,
    fee: u32,
    weight: u32,
    deps: Option<String>,
}

#[derive(Debug)]
struct Package {
    fee: u32,
    weight: u32,
    txs: Vec<String>,
}
