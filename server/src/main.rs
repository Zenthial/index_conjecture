use axum::{routing::get, Router};
use libsql_client::client::Client;

#[derive(Clone)]
struct AppState {
    db: Arc<Client>
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_turso::Turso(
        addr = "{secrets.DB_URI}",
        local_addr = "{secrets.DB_URI}",
        token = "{secrets.DB_TOKEN}"
    )]
    client: Client,
    #[shuttle_secrets::Secrets] _secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let state = AppState {
        db: Arc::new(client)
    };

    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
