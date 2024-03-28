use hashbrown::HashSet;
use num::Integer;
use rand::prelude::*;
use rayon::prelude::*;
use serde::Serialize;

use std::collections::VecDeque;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

fn w_index(s: [i64; 4], n: i64, coprimes: &[i64]) -> i64 {
    for g in coprimes {
        // using par iter here causes the program to DRASTICALLY slow
        // this is because 's' is only four elements long, so spawning a thread takes longer than
        // just calculating the sum
        let sum: i64 = s.iter().map(|i| (i * g) % n).sum(); // this is lines 40 -> 43
        let sum_div = sum / n;
        if sum_div != 2 {
            return 1;
        }
    }

    2
}

fn big_check(n: i64) {
    let half_n = n / 2;
    let coprimes: Vec<i64> = (1..n).into_par_iter().filter(|&i| i.gcd(&n) == 1).collect();

    let coprimes_a: Vec<i64> = (&coprimes)
        .into_par_iter()
        .filter_map(|i| {
            if *i >= 7 && *i < half_n {
                Some(*i)
            } else {
                None
            }
        })
        .collect();

    let coprimes_b: Vec<i64> = (&coprimes)
        .into_par_iter()
        .filter_map(|i| {
            if *i > half_n && *i < n - 2 {
                Some(*i)
            } else {
                None
            }
        })
        .collect();

    let coprime_set: HashSet<&i64> = HashSet::from_iter(coprimes.iter());

    coprimes_a.into_par_iter().for_each(|a| {
        for &b in coprimes_b.iter() {
            if b >= n + 2 - a && b <= n - (3 / 2) - (a / 2) {
                let c = 2 * n - a - b - 1;

                if coprime_set.contains(&c) {
                    let s = [1, a, b, c];
                    let sum: i64 = s.iter().sum();

                    if sum % n == 0 {
                        if w_index(s, n, &*coprimes) != 1 {
                            println!("error at: {} for: {:?}", n, s);
                            return;
                        }
                    }
                }
            }
        }
    })
}

const DIR_PATH: &'static str = "/sciclone/scr-lst/ajpendleton";

fn get_remaining_num() -> Option<i64> {
    let path_str = format!("{DIR_PATH}/remaining.lock");
    let remaining_path = Path::new(&path_str);
    if !remaining_path.exists() {
        return None;
    }

    let _lock_file = match fs::OpenOptions::new().create_new(true).open(&path_str) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                ErrorKind::AlreadyExists => (),
                _ => eprintln!("unexpected file open error {}", e.to_string()),
            };
            return None;
        }
    };

    let mut remaining_file = fs::OpenOptions::new()
        .write(true)
        .append(false)
        .read(true)
        .open(&format!("{DIR_PATH}/remaining"))
        .unwrap();

    let mut content = String::new();
    match remaining_file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return None;
        }
    };

    let mut queue: VecDeque<i64> = ron::from_str(&content).unwrap();
    let num = match queue.pop_front() {
        Some(i) => i,
        None => return None,
    };

    let queue_str = ron::to_string(&queue).unwrap();
    remaining_file.write_all(queue_str.as_bytes()).unwrap();
    fs::remove_file(&path_str).unwrap();

    Some(num)
}

fn write_to_vec(file_name: &str, num: i64) {
    let mut rand = rand::thread_rng();

    loop {
        let path_str = format!("{DIR_PATH}/{file_name}.lock");
        let remaining_path = Path::new(&path_str);
        if !remaining_path.exists() {
            let wait = rand.gen::<f64>() + 0.1;
            thread::sleep(Duration::from_secs_f64(wait));
            continue;
        }

        let _lock_file = match fs::OpenOptions::new().create_new(true).open(&path_str) {
            Ok(f) => f,
            Err(e) => {
                match e.kind() {
                    ErrorKind::AlreadyExists => (),
                    _ => panic!("unexpected file open error {}", e.to_string()),
                };

                let wait = rand.gen::<f64>() + 0.1;
                thread::sleep(Duration::from_secs_f64(wait));
                continue;
            }
        };

        let mut remaining_file = fs::OpenOptions::new()
            .write(true)
            .append(false)
            .read(true)
            .open(&format!("{DIR_PATH}/remaining"))
            .unwrap();

        let mut content = String::new();
        match remaining_file.read_to_string(&mut content) {
            Ok(_) => (),
            Err(e) => {
                panic!("Error reading file: {}", e);
            }
        };

        let mut vec: Vec<i64> = ron::from_str(&content).unwrap();
        vec.push(num);

        let vec_str = ron::to_string(&vec).unwrap();
        remaining_file.write_all(vec_str.as_bytes()).unwrap();
        fs::remove_file(&path_str).unwrap();
        break;
    }
}

fn main() {
    let mut rand = rand::thread_rng();

    loop {
        let wait = (rand.gen::<f64>() + 1.0) * 5.0;
        thread::sleep(Duration::from_secs_f64(wait));

        let i = match get_remaining_num() {
            Some(i) => i,
            None => continue,
        };

        write_to_vec("processing", i);
        println!("processing {i}");
        big_check(i);
        write_to_vec("processed", i);
        println!("checked {i}");
    }
}
