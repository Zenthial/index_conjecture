use hashbrown::HashSet;
use num::Integer;
use rayon::prelude::*;

use std::collections::VecDeque;
use std::fs;
use std::io::ErrorKind;
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

// fn domain(n: i64, m: i64) -> Vec<i64> {
//     (n..m)
//         .into_par_iter()
//         .filter(|&i| {
//             if (i.gcd(&6) == 1) & (i % 5 == 0) {
//                 return true;
//             }
//
//             false
//         })
//         .collect()
// }

const DIR_PATH: &'static str = "/sciclone/scr-lst/ajpendleton";

enum Reason {
    NoneLeft,
    Retry,
}

// fn fill_queue() {
//     let curr_int_ron = fs::read_to_string(&format!("{DIR_PATH}/current_min.ron")).unwrap();
//     let curr_int: i64 = ron::from_str(&curr_int_ron).unwrap();
//
//     let new_domain =
// }

fn retrieve_lock(path_str: &str) -> Result<fs::File, Reason> {
    let remaining_path = Path::new(path_str);
    if remaining_path.exists() {
        return Err(Reason::Retry);
    }

    let _lock_file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path_str)
    {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                ErrorKind::AlreadyExists => (),
                _ => eprintln!("unexpected file open error {}", e.to_string()),
            };
            return Err(Reason::Retry);
        }
    };

    Ok(_lock_file)
}

fn get_remaining_num() -> Result<i64, Reason> {
    let path_str = format!("{DIR_PATH}/remaining.ron.lock");
    let _ = retrieve_lock(&path_str)?;

    let remaining_file_name = format!("{DIR_PATH}/remaining.ron");
    let content = fs::read_to_string(&remaining_file_name).unwrap();

    let mut queue: VecDeque<i64> = ron::from_str(&content).unwrap();
    let num = match queue.pop_front() {
        Some(i) => i,
        None => {
            fs::remove_file(&path_str).unwrap();
            return Err(Reason::NoneLeft);
        }
    };

    let queue_str = ron::to_string(&queue).unwrap();
    fs::write(&remaining_file_name, queue_str).unwrap();
    fs::remove_file(&path_str).unwrap();

    Ok(num)
}

fn write_to_vec(file_name: &str, num: i64) {
    loop {
        let path_str = format!("{DIR_PATH}/{file_name}.lock");
        match retrieve_lock(&path_str) {
            Err(e) => match e {
                Reason::Retry => {
                    thread::sleep(Duration::from_secs_f64(0.1));
                    continue;
                }
                _ => unreachable!(),
            },
            Ok(_) => {}
        };

        let vec_file_name = format!("{DIR_PATH}/{file_name}");
        let content = fs::read_to_string(&vec_file_name).unwrap();

        let mut vec: Vec<i64> = ron::from_str(&content).unwrap();
        vec.push(num);

        let vec_str = ron::to_string(&vec).unwrap();
        fs::write(&vec_file_name, vec_str).unwrap();
        // write!(vec_file, "{vec_str}").unwrap();
        fs::remove_file(&path_str).unwrap();
        break;
    }
}

fn remove_from_vec(file_name: &str, num: i64) {
    loop {
        let path_str = format!("{DIR_PATH}/{file_name}.lock");
        match retrieve_lock(&path_str) {
            Err(e) => match e {
                Reason::Retry => {
                    thread::sleep(Duration::from_secs_f64(0.1));
                    continue;
                }
                _ => unreachable!(),
            },
            Ok(_) => {}
        };

        let vec_file_name = format!("{DIR_PATH}/{file_name}");
        let content = fs::read_to_string(&vec_file_name).unwrap();

        let mut vec: Vec<i64> = ron::from_str(&content).unwrap();
        vec = vec.into_par_iter().filter(|&i| i != num).collect();

        let vec_str = ron::to_string(&vec).unwrap();
        fs::write(&vec_file_name, vec_str).unwrap();
        // write!(vec_file, "{vec_str}").unwrap();
        fs::remove_file(&path_str).unwrap();
        break;
    }
}

fn main() {
    loop {
        thread::sleep(Duration::from_secs_f64(0.1));
        let i = match get_remaining_num() {
            Ok(i) => i,
            Err(e) => match e {
                Reason::Retry => continue,
                Reason::NoneLeft => break,
            },
        };

        write_to_vec("processing.ron", i);
        println!("processing {i}");
        big_check(i);
        remove_from_vec("processing.ron", i);
        write_to_vec("processed.ron", i);
        println!("checked {i}");
    }

    println!("no more remaining numbers");
}
