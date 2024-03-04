use axum::{
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::process;
use std::{fs};
use std::path::Path;
use query::{get_epoch, get_proposals};
use tendermint_rpc::{self, HttpClient};
use tower_http::cors::{CorsLayer, Any};
use crate::query::{check_steward, get_balance, get_delegators_delegation, get_delegators_delegation_at, get_epoch_at_height, get_governance_parameters, get_meta_data, get_proposal_votes, get_tx_events, get_validator_consensus_keys, get_validator_state};

mod query;


#[derive(Clone, Debug, Serialize, Deserialize)]
struct Settings {
    rpc_url: String,
    bind_ip: String,
    port: u16,
}

#[derive(Clone)]
pub struct ServerState {
    client: HttpClient,
    config: Settings
}

#[tokio::main]
async fn main() {

    // Load config
    let settings_path = "config/Settings.toml";
    let config = read_settings_from_file(settings_path).unwrap_or_else(|err| {
        eprintln!("Failed to read settings: {}", err);
        process::exit(1);
    });

    // Connect to RPC
    let client = HttpClient::new(config.rpc_url.as_str()).unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Namada REST API Running" }))
        .route("/proposal_result/:id", get(get_proposals))
        .route("/epoch", get(get_epoch))
        .route("/epoch_at_height/:height", get(get_epoch_at_height))
        .route("/balance/:wallet",get(get_balance))
        .route("/validator_state/:address/:epoch",get(get_validator_state))
        .route("/delegator_delegation/:wallet",get(get_delegators_delegation))
        .route("/delegator_delegation_at/:wallet/:epoch",get(get_delegators_delegation_at))
        .route("/metadata/:address/:epoch",get(get_meta_data))
        .route("/governance", get(get_governance_parameters))
        .route("/proposal_votes/:id", get(get_proposal_votes))
        .route("/is_steward/:wallet",get(check_steward))
        .route("/validator_consensus_keys/:wallet",get(get_validator_consensus_keys))
        .route("/tx_event/:tx_hash",get(get_tx_events))


        .with_state(ServerState { client: client, config: config.clone() })
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_credentials(false),
        );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.bind_ip, config.port)).await.unwrap();
    println!("Server listening {}:{}", config.bind_ip, config.port);

    axum::serve(listener, app).await.unwrap();
}


fn read_settings_from_file<P: AsRef<Path>>(path: P) -> Result<Settings, Box<dyn std::error::Error>> {
    let settings_str = fs::read_to_string(path)?;
    let settings: Settings = toml::from_str(&settings_str)?;
    Ok(settings)
}
