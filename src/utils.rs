use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use namada_sdk::{
    error::{self, PinnedBalanceError},
    governance::utils::{ProposalResult, TallyType},
    rpc,
    state::Epoch,
    types::dec::Dec,
};
use namada_sdk::error::{EncodingError, QueryError};
use namada_sdk::state::BlockHeight;
use namada_sdk::types::address::Address;
use namada_sdk::types::token;
use serde_json::{json, Value};
use tendermint_rpc::{self, HttpClient};

use crate::ServerState;

pub enum RPCRequestType {
    QueryEpoch,
    QueryEpochAtHeight(BlockHeight),
    QueryProposalResult(u64),
    QueryBalance(Address, Address),
}

pub enum RPCResult {
    Epoch(Epoch),
    EpochAtHeight(Option<Epoch>),
    ProposalResult(Option<ProposalResult>),
    BalanceResult(token::Amount),
}

pub struct MyErrorWrapper(error::Error);

// Implement `IntoResponse` for your new type
impl IntoResponse for MyErrorWrapper {
    fn into_response(self) -> Response {
        // Here you convert your error into an axum response.
        // You can customize this to return a JSON error message, set the status code, etc.
        let error_message = format!("{}", self.0); // Assuming error::Error implements Display
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({ "error": error_message })),
        )
            .into_response()
    }
}

pub async fn get_epoch(State(state): State<ServerState>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryEpoch).await
}

pub async fn get_epoch_at_height(State(state): State<ServerState>,
                                 Path(height): Path<BlockHeight>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryEpochAtHeight(height)).await
}

pub async fn get_balance(State(state): State<ServerState>,
                         Path(owner): Path<Address>, ) -> Result<Json<Value>, MyErrorWrapper> {
    let decode = Address::decode("tnam1qxvg64psvhwumv3mwrrjfcz0h3t3274hwggyzcee");
    match decode {
        Ok(token_address) => get_rpc_data(state.client, RPCRequestType::QueryBalance(token_address, owner)).await,
        Err(_) => Err(MyErrorWrapper(error::Error::Encode(
            EncodingError::Decoding("Error decoding address.".to_string()),
        )))
    }
}

pub async fn get_proposals(
    State(state): State<ServerState>,
    Path(id): Path<u32>,
) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryProposalResult(id as u64)).await
}

// We need to do all this mess only because rpc::query_something is !Send which is a requirment for axum
pub async fn get_rpc_data(
    client: HttpClient,
    req_type: RPCRequestType,
) -> Result<Json<Value>, MyErrorWrapper> {
    let result = tokio::task::spawn_blocking(move || {
        // Execute the blocking operation
        tokio::runtime::Handle::current().block_on(async {
            match req_type {
                RPCRequestType::QueryEpoch => rpc::query_epoch(&client).await.map(RPCResult::Epoch),
                RPCRequestType::QueryEpochAtHeight(height) => rpc::query_epoch_at_height(&client, height).await.map(RPCResult::EpochAtHeight),
                RPCRequestType::QueryProposalResult(id) => rpc::query_proposal_result(&client, id)
                    .await
                    .map(RPCResult::ProposalResult),
                RPCRequestType::QueryBalance(token, owner) => rpc::get_token_balance(&client, &token, &owner).await.map(RPCResult::BalanceResult)
            }
        })
    })
        .await
        .map_err(|e| {
            // Directly handle the conversion from JoinError to MyErrorWrapper
            if e.is_cancelled() {
                MyErrorWrapper(error::Error::Pinned(
                    PinnedBalanceError::NoTransactionPinned,
                ))
            } else {
                // You can adjust this part to better fit your error model
                MyErrorWrapper(error::Error::Pinned(
                    PinnedBalanceError::NoTransactionPinned,
                ))
            }
        })?;

    result
        .map(|rpc_result| {
            match rpc_result {
                RPCResult::Epoch(epoch_data) => Json(json!({ "epoch": epoch_data })),
                RPCResult::EpochAtHeight(maybe_epoch) => match maybe_epoch {
                    Some(epoch_data) => Json(json!({ "epoch": epoch_data })),
                    None => Json(json!({ "epoch": "None" })),
                },
                RPCResult::ProposalResult(proposal_result) => {
                    // We need to reformat proposal result data because it doesn't implement serialize
                    if let Some(proposal_result) = proposal_result {
                        let threshold = match proposal_result.tally_type {
                            TallyType::TwoThirds => {
                                proposal_result.total_voting_power.mul_ceil(Dec::two() / 3)
                            }
                            _ => proposal_result.total_voting_power.mul_ceil(Dec::one() / 3),
                        };

                        let thresh_frac =
                            Dec::from(threshold) / Dec::from(proposal_result.total_voting_power);

                        return Json(json!({
                            "result": format!("{}", proposal_result.result),
                            "total_voting_power": proposal_result.total_voting_power,
                            "total_yay_power": proposal_result.total_yay_power,
                            "total_nay_power": proposal_result.total_nay_power,
                            "total_abstain_power": proposal_result.total_abstain_power,
                            "threshold": threshold,
                            "thresh_frac": thresh_frac
                        }));
                    }

                    return Json(json!({"error": "proposal not found"}));
                },
                RPCResult::BalanceResult(amount) => {
                    Json(json!({ "balance": amount }))
                }
            }
        })
        .map_err(MyErrorWrapper)
}
