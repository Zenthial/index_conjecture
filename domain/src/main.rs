use num::Integer;
use rayon::prelude::*;
use ron;

use std::collections::HashSet;
use std::env::args;
use std::fs;

fn domain(n: i64, m: i64) -> Vec<i64> {
    (n..m)
        .into_par_iter()
        .filter(|&i| {
            if (i.gcd(&6) == 1) & (i % 5 == 0) {
                return true;
            }

            false
        })
        .collect()
}

fn main() {
    let args: Vec<String> = args().collect();
    match args[1].as_str() {
        "generate" => {
            let min: i64 = args[2].parse().unwrap();
            let max: i64 = args[3].parse().unwrap();

            let mut dom = domain(min, max);
            dom.sort_unstable();

            let ron_string = ron::to_string(&dom).unwrap();
            match fs::write("remaining", ron_string) {
                Ok(_) => println!("{} -> {} generated, written to ./remaining", min, max),
                Err(e) => eprintln!("failed to write with error {}", e.to_string()),
            }
        }
        "merge" => {
            let file_one = fs::read_to_string(&args[2]).unwrap();
            let file_two = fs::read_to_string(&args[3]).unwrap();

            let mut vec_one: Vec<i64> = ron::from_str(file_one.as_str()).unwrap();
            let mut vec_two: Vec<i64> = ron::from_str(file_two.as_str()).unwrap();
            vec_one.append(&mut vec_two);

            let set: HashSet<i64> = HashSet::from_iter(vec_one.into_iter());
            let mut vec: Vec<i64> = Vec::from_iter(set.into_iter());
            vec.sort_unstable();

            let ron_string = ron::to_string(&vec).unwrap();
            match fs::write("merge", ron_string) {
                Ok(_) => println!("merge written to ./merge"),
                Err(e) => eprintln!("failed to write with error {}", e.to_string()),
            }
        }
        _ => eprintln!("argument 1 must be 'generate' or 'merge'"),
    }
}
