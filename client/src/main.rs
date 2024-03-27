use hashbrown::HashSet;
use num::Integer;
use rayon::prelude::*;
use reqwest::blocking::Client;
use serde::Serialize;

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

const URL: &'static str = "https://idx-conj.shuttleapp.rs/num";

#[derive(Serialize)]
struct Processed {
    num: i64,
}

fn main() {
    let blocking_client = Client::new();
    loop {
        let response = blocking_client.get(URL).send().unwrap();
        let text = response.text().unwrap();

        match text.parse() {
            Ok(i) => {
                println!("processing {i}");
                big_check(i);
                println!("checked {i}");
                let _ = blocking_client.post(URL).json(&Processed { num: i }).send();
            }
            Err(_) => break,
        }
    }
}
