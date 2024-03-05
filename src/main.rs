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
        .route("/epoch_at_height/:height", get(query::get_epoch_at_height))
        .route("/balance/:wallet",get(query::get_balance))
        .route("/validator_state/:address/:epoch",get(query::get_validator_state))
        .route("/delegator_delegation/:wallet",get(query::get_delegators_delegation))
        .route("/delegator_delegation_at/:wallet/:epoch",get(query::get_delegators_delegation_at))
        .route("/metadata/:address/:epoch",get(query::get_meta_data))
        .route("/governance", get(query::get_governance_parameters))
        .route("/pos_params", get(query::get_pos_parameters))
        .route("/proposal_votes/:id", get(query::get_proposal_votes))
        .route("/is_steward/:wallet",get(query::check_steward))
        .route("/validator_consensus_keys/:wallet",get(query::get_validator_consensus_keys))
        .route("/tx_event/:tx_hash",get(query::get_tx_events))
        .route("/native_token",get(query::get_native_token))
        .route("/query_block",get(query::get_latest_block))
        .route("/is_validator/:wallet",get(query::check_is_validator))
        .route("/is_delegator/:wallet",get(query::check_is_delegator))
        .route("/masp_reward",get(query::get_masp_reward))
        .route("/total_staked/:epoch",get(query::get_total_staked_tokens))
        .route("/validator_stake/:address/:epoch",get(query::get_validator_stake))
        .with_state(ServerState { client, config: config.clone() })
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
