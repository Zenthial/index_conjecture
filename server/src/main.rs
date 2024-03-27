use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use libsql::{de, Connection};
use num::Integer;
use rayon::prelude::*;
use serde::Deserialize;
// use shuttle_runtime::{SecretStore, Secrets};

use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Connection>,
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

async fn write_new_remaining(db: Arc<Connection>) -> i64 {
    let mut stats_rows = db
        .query("SELECT * FROM stats WHERE KEY = 'max';", ())
        .await
        .unwrap();

    let max_row = stats_rows.next().unwrap().unwrap();
    let max = max_row.get::<i64>(1).unwrap();
    let new_max = max * 2;

    let new_domain = domain(max, new_max).await;

    let mut stmt = db.prepare("INSERT INTO remaining VALUES ?1").await.unwrap();

    let mut remaining = Vec::new();
    for num in &new_domain[1..] {
        remaining.push(*num);
    }

    stmt.execute(remaining).await.unwrap();

    db.execute(
        "UPDATE stats SET VALUE = ?1 WHERE KEY = \"max\";",
        [new_max],
    )
    .await
    .unwrap();
    db.execute("UPDATE stats SET VALUE = ?1 WHERE KEY = \"min\";", [max])
        .await
        .unwrap();

    return new_domain[0];
}

async fn get_num(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.db;

    let row_opt = db
        .query(
            "SELECT * FROM remaining WHERE ID = (SELECT MIN(ID) FROM remaining);",
            (),
        )
        .await
        .unwrap()
        .next()
        .unwrap();

    let row = match row_opt {
        Some(r) => r,
        None => {
            let new_num = write_new_remaining(db).await;
            return (StatusCode::OK, new_num.to_string());
        }
    };

    let info = de::from_row::<Remaining>(&row).unwrap();

    db.execute("INSERT INTO processing(num) VALUES (?1);", [info.num])
        .await
        .unwrap();

    db.execute("DELETE FROM remaining WHERE ID = ?1;", [info.id])
        .await
        .unwrap();

    (StatusCode::OK, info.num.to_string())
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
    let state = AppState {
        db: Arc::new(client),
    };

    let router = Router::new()
        .route("/num", get(get_num).post(process_num))
        .with_state(state);

    Ok(router.into())
}
