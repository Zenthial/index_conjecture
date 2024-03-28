use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use libsql::{de, Connection};
use num::Integer;
use rayon::prelude::*;
use serde::Deserialize;
use tokio::sync::Mutex;

use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Connection>,
    queue: Arc<Mutex<Box<VecDeque<(i32, i64)>>>>,
}

#[derive(Deserialize)]
struct Remaining {
    #[serde(rename = "ID")]
    id: i32,
    num: i64,
}

#[derive(Deserialize)]
struct Process {
    num: i64,
}

async fn domain(n: i64, m: i64) -> Vec<i64> {
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

async fn write_new_remaining(db: Arc<Connection>) {
    let mut stats_rows = db
        .query("SELECT * FROM stats WHERE KEY = 'max';", ())
        .await
        .unwrap();

    let max_row = stats_rows.next().unwrap().unwrap();
    let max = max_row.get::<i64>(1).unwrap();
    let new_max = max + 50_000;

    let new_domain = domain(max, new_max).await;

    let mut many_insert = String::from("INSERT INTO remaining(num) VALUES ");

    for num in new_domain {
        many_insert += &format!("({}), ", num);
    }

    let mut query = String::from(many_insert.strip_suffix(", ").unwrap());
    query += ";";
    db.execute(query.as_str(), ()).await.unwrap();

    db.execute(
        "UPDATE stats SET VALUE = ?1 WHERE KEY = \"max\";",
        [new_max],
    )
    .await
    .unwrap();
    db.execute("UPDATE stats SET VALUE = ?1 WHERE KEY = \"min\";", [max])
        .await
        .unwrap();

    println!("filled remaining table");
}

async fn fill_queue(db: Arc<Connection>, queue: Arc<Mutex<Box<VecDeque<(i32, i64)>>>>) {
    let count = db
        .query("SELECT COUNT(*) FROM remaining;", ())
        .await
        .unwrap()
        .next()
        .unwrap()
        .unwrap();

    if count.get::<i32>(0).unwrap() == 0 {
        write_new_remaining(db.clone()).await;
    }

    let mut remaining_rows = db
        .query("SELECT * FROM remaining LIMIT 100;", ())
        .await
        .unwrap();

    let mut remaining_row = remaining_rows.next().unwrap();

    let mut q = queue.lock().await;

    while let Some(row) = remaining_row {
        let r = de::from_row::<Remaining>(&row).unwrap();
        q.push_back((r.id, r.num));
        remaining_row = remaining_rows.next().unwrap();
    }

    println!("queue filled: {:?}", q);
}

async fn get_num(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.db.clone();

    let mut queue = state.queue.lock().await;
    if queue.len() == 0 {
        drop(queue);

        fill_queue(db.clone(), state.queue.clone()).await;
        queue = state.queue.lock().await;
    }

    let (id, num) = queue.pop_front().unwrap();

    db.execute("INSERT INTO processing(num) VALUES (?1);", [num])
        .await
        .unwrap();

    db.execute("DELETE FROM remaining WHERE ID = ?1;", [id])
        .await
        .unwrap();

    (StatusCode::OK, num.to_string())
}

async fn process_num(
    State(state): State<AppState>,
    Json(body): Json<Process>,
) -> impl IntoResponse {
    let db = state.db;

    db.execute("DELETE FROM processing WHERE num = ?1", [body.num])
        .await
        .unwrap();

    db.execute("INSERT INTO processed VALUES (?1)", [body.num])
        .await
        .unwrap();

    StatusCode::OK
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_turso::Turso(
        addr = "{secrets.DB_URI}",
        local_addr = "{secrets.DB_URI}",
        token = "{secrets.DB_TOKEN}"
    )]
    client: Connection,
) -> shuttle_axum::ShuttleAxum {
    let queue = VecDeque::new();

    let state = AppState {
        db: Arc::new(client),
        queue: Arc::new(Mutex::new(Box::new(queue))),
    };

    let router = Router::new()
        .route("/num", get(get_num))
        .route("/num", post(process_num))
        .with_state(state);

    Ok(router.into())
}
