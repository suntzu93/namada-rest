mod constants;

use std::collections::{HashMap, HashSet};
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
use namada_sdk::error::{EncodingError};
use namada_sdk::events::Event;
use namada_sdk::governance::parameters::GovernanceParameters;
use namada_sdk::governance::utils::Vote;
use namada_sdk::masp::MaspTokenRewardData;
use namada_sdk::proof_of_stake::{PosParams};
use namada_sdk::proof_of_stake::types::{CommissionPair, ValidatorMetaData, ValidatorState};
use namada_sdk::rpc::{TxEventQuery};
use namada_sdk::state::{BlockHash, BlockHeight, LastBlock};
use namada_sdk::types::address::Address;
use namada_sdk::types::key::common;
use namada_sdk::types::time::DateTimeUtc;
use namada_sdk::types::token;
use namada_sdk::types::uint::Uint;
use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tendermint_rpc::{self, HttpClient};

use crate::ServerState;

pub enum RPCRequestType {
    QueryEpoch,
    QueryEpochAtHeight(BlockHeight),
    QueryProposalResult(u64),
    QueryProposalVotes(u64),
    QueryBalance(Address, Address),
    QueryValidatorState(Address, Option<Epoch>),
    QueryDelegatorDelegation(Address),
    QueryDelegatorDelegationAt(Address, Epoch),
    QueryMetaData(Address, Option<Epoch>),
    QueryGovernanceParameters,
    QueryPosParameters,
    QueryCheckIsSteward(Address),
    QueryValidatorConsensusKeys(Address),
    QueryTxEvents(String),
    QueryNativeToken,
    QueryLatestBlock,
    QueryCheckIsValidator(Address),
    QueryCheckIsDelegator(Address),
    QueryMaspReward,
    QueryTotalStakedTokens(Epoch),
    QueryValidatorStaked(Epoch, Address),
}

pub enum RPCResult {
    Epoch(Epoch),
    EpochAtHeight(Option<Epoch>),
    ProposalResult(Option<ProposalResult>),
    ProposalVotes(Vec<Vote>),
    BalanceResult(token::Amount),
    ValidatorState(Option<ValidatorState>),
    DelegatorDelegation(HashSet<Address>),
    DelegatorDelegationAt(HashMap<Address, token::Amount>),
    MetaData((Option<ValidatorMetaData>, Option<CommissionPair>)),
    GovernanceParameters(GovernanceParameters),
    PosParameters(PosParams),
    IsSteward(bool),
    ValidatorConsensusKeys(Option<common::PublicKey>),
    TxEvents(Event),
    NativeToken(Address),
    LatestBlock(Option<LastBlock>),
    IsValidator(bool),
    IsDelegator(bool),
    MapsReward(Vec<MaspTokenRewardData>),
    TotalStakedTokens(token::Amount),
    ValidatorStake(token::Amount),
}

#[derive(Serialize)]
pub struct CommissionPairWrapper {
    commission_rate: String,
    max_commission_change_per_epoch: String,
}

#[derive(Serialize)]
struct GovernanceParametersWrapper {
    min_proposal_fund: String,
    max_proposal_code_size: String,
    min_proposal_voting_period: String,
    max_proposal_period: String,
    max_proposal_content_size: String,
    min_proposal_grace_epochs: String,
}

#[derive(Serialize)]
pub struct SerializableOwnedPosParams {
    pub max_validator_slots: u64,
    pub pipeline_len: u64,
    pub unbonding_len: u64,
    pub tm_votes_per_token: String,
    pub block_proposer_reward: String,
    pub block_vote_reward: String,
    pub max_inflation_rate: String,
    pub target_staked_ratio: String,
    pub duplicate_vote_min_slash_rate: String,
    pub light_client_attack_min_slash_rate: String,
    pub cubic_slashing_window_length: u64,
    pub validator_stake_threshold: String,
    pub liveness_window_check: u64,
    pub liveness_threshold: String,
    pub rewards_gain_p: String,
    pub rewards_gain_d: String,
}

#[derive(Serialize)]
pub struct SerializablePosParams {
    pub owned: SerializableOwnedPosParams,
    pub max_proposal_period: u64,
}

#[derive(Serialize)]
pub struct VoteWrapper {
    validator: String,
    delegator: String,
    data: String,
}

#[derive(Serialize)]
pub struct EventSerializable {
    event_type: String,
    level: String,
    attributes: HashMap<String, String>,
}

#[derive(Serialize)]
struct SerializableLastBlock {
    height: BlockHeight,
    hash: BlockHash,
    time: DateTimeUtc,
}

#[derive(Serialize)]
struct MaspTokenRewardDataWrapper {
    pub name: String,
    pub address: Address,
    pub max_reward_rate: Dec,
    pub kp_gain: Dec,
    pub kd_gain: Dec,
    pub locked_amount_target: Uint,
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
    let decode = Address::decode(constants::NAAN_ADDRESS);
    match decode {
        Ok(token_address) => get_rpc_data(state.client, RPCRequestType::QueryBalance(token_address, owner)).await,
        Err(_) => Err(MyErrorWrapper(error::Error::Encode(
            EncodingError::Decoding("Error decoding address.".to_string()),
        )))
    }
}

pub async fn get_validator_state(State(state): State<ServerState>,
                                 Path((address, epoch)): Path<(Address, Epoch)>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryValidatorState(address, Some(epoch))).await
}

pub async fn get_delegators_delegation(State(state): State<ServerState>,
                                       Path(delegator): Path<Address>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryDelegatorDelegation(delegator)).await
}

pub async fn get_delegators_delegation_at(State(state): State<ServerState>,
                                          Path((address, epoch)): Path<(Address, Epoch)>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryDelegatorDelegationAt(address, epoch)).await
}

pub async fn get_meta_data(State(state): State<ServerState>,
                           Path((address, epoch)): Path<(Address, Epoch)>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryMetaData(address, Some(epoch))).await
}

pub async fn get_governance_parameters(State(state): State<ServerState>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryGovernanceParameters).await
}

pub async fn get_pos_parameters(State(state): State<ServerState>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryPosParameters).await
}

pub async fn check_steward(State(state): State<ServerState>,
                           Path(address): Path<Address>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryCheckIsSteward(address)).await
}

pub async fn get_proposals(
    State(state): State<ServerState>,
    Path(id): Path<u32>,
) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryProposalResult(id as u64)).await
}

pub async fn get_proposal_votes(
    State(state): State<ServerState>,
    Path(id): Path<u32>,
) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryProposalVotes(id as u64)).await
}

pub async fn get_validator_consensus_keys(State(state): State<ServerState>,
                                          Path(address): Path<Address>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryValidatorConsensusKeys(address)).await
}

pub async fn get_tx_events(State(state): State<ServerState>,
                           Path(tx_hash): Path<String>) -> Result<Json<Value>, MyErrorWrapper> {
    {
        get_rpc_data(state.client, RPCRequestType::QueryTxEvents(tx_hash)).await
    }
}

pub async fn get_native_token(State(state): State<ServerState>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryNativeToken).await
}

pub async fn get_latest_block(State(state): State<ServerState>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryLatestBlock).await
}

pub async fn check_is_validator(State(state): State<ServerState>,
                                Path(address): Path<Address>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryCheckIsValidator(address)).await
}

pub async fn check_is_delegator(State(state): State<ServerState>,
                                Path(address): Path<Address>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryCheckIsDelegator(address)).await
}

pub async fn get_masp_reward(State(state): State<ServerState>) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryMaspReward).await
}

pub async fn get_total_staked_tokens(State(state): State<ServerState>,
                                     Path(epoch): Path<Epoch>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryTotalStakedTokens(epoch)).await
}

pub async fn get_validator_stake(State(state): State<ServerState>,
                                 Path((address, epoch)): Path<(Address, Epoch)>, ) -> Result<Json<Value>, MyErrorWrapper> {
    get_rpc_data(state.client, RPCRequestType::QueryValidatorStaked(epoch, address)).await
}

pub fn serialize<S>(public_key: &Option<common::PublicKey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    match public_key {
        Some(common::PublicKey::Ed25519(pk)) => serializer.serialize_str(&format!("{}", pk)),
        Some(common::PublicKey::Secp256k1(pk)) => serializer.serialize_str(&format!("{}", pk)),
        None => serializer.serialize_none(),
    }
}

fn to_serializable(event: Event) -> EventSerializable {
    EventSerializable {
        event_type: format!("{:?}", event.event_type),
        level: format!("{:?}", event.level),
        attributes: event.attributes,
    }
}

fn block_to_serializable(last_block: &LastBlock) -> SerializableLastBlock {
    SerializableLastBlock {
        height: last_block.height.clone(),
        hash: last_block.hash.clone(),
        time: last_block.time,
    }
}

fn convert_to_serializable_pos(pos_params: PosParams) -> SerializablePosParams {
    let owned_params = pos_params.owned;
    let serializable_owned_pos_params = SerializableOwnedPosParams {
        max_validator_slots: owned_params.max_validator_slots,
        pipeline_len: owned_params.pipeline_len,
        unbonding_len: owned_params.unbonding_len,
        tm_votes_per_token: owned_params.tm_votes_per_token.to_string(),
        block_proposer_reward: owned_params.block_proposer_reward.to_string(),
        block_vote_reward: owned_params.block_vote_reward.to_string(),
        max_inflation_rate: owned_params.max_inflation_rate.to_string(),
        target_staked_ratio: owned_params.target_staked_ratio.to_string(),
        duplicate_vote_min_slash_rate: owned_params.duplicate_vote_min_slash_rate.to_string(),
        light_client_attack_min_slash_rate: owned_params.light_client_attack_min_slash_rate.to_string(),
        cubic_slashing_window_length: owned_params.cubic_slashing_window_length,
        validator_stake_threshold: owned_params.validator_stake_threshold.to_string(),
        liveness_window_check: owned_params.liveness_window_check,
        liveness_threshold: owned_params.liveness_threshold.to_string(),
        rewards_gain_p: owned_params.rewards_gain_p.to_string(),
        rewards_gain_d: owned_params.rewards_gain_d.to_string(),
    };

    SerializablePosParams {
        owned: serializable_owned_pos_params,
        max_proposal_period: pos_params.max_proposal_period,
    }
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
                RPCRequestType::QueryEpochAtHeight(height) => rpc::query_epoch_at_height(&client, height)
                    .await
                    .map(RPCResult::EpochAtHeight),
                RPCRequestType::QueryProposalResult(id) => rpc::query_proposal_result(&client, id)
                    .await
                    .map(RPCResult::ProposalResult),
                RPCRequestType::QueryProposalVotes(id) => rpc::query_proposal_votes(&client, id)
                    .await
                    .map(RPCResult::ProposalVotes),
                RPCRequestType::QueryBalance(token, owner) => rpc::get_token_balance(&client, &token, &owner)
                    .await
                    .map(RPCResult::BalanceResult),
                RPCRequestType::QueryValidatorState(address, epoch) => rpc::get_validator_state(&client, &address, epoch)
                    .await
                    .map(RPCResult::ValidatorState),
                RPCRequestType::QueryDelegatorDelegation(address) => rpc::get_delegators_delegation(&client, &address)
                    .await
                    .map(RPCResult::DelegatorDelegation),
                RPCRequestType::QueryDelegatorDelegationAt(address, epoch) => rpc::get_delegators_delegation_at(&client, &address, epoch)
                    .await
                    .map(RPCResult::DelegatorDelegationAt),
                RPCRequestType::QueryMetaData(address, epoch) => rpc::query_metadata(&client, &address, epoch)
                    .await
                    .map(RPCResult::MetaData),
                RPCRequestType::QueryGovernanceParameters => {
                    let result = rpc::query_governance_parameters(&client).await;
                    Ok(RPCResult::GovernanceParameters(result))
                }
                RPCRequestType::QueryPosParameters => rpc::get_pos_params(&client)
                    .await
                    .map(RPCResult::PosParameters),
                RPCRequestType::QueryCheckIsSteward(address) => {
                    let result = rpc::is_steward(&client, &address).await;
                    Ok(RPCResult::IsSteward(result))
                }
                RPCRequestType::QueryValidatorConsensusKeys(address) => rpc::query_validator_consensus_keys(&client, &address)
                    .await
                    .map(RPCResult::ValidatorConsensusKeys),
                RPCRequestType::QueryTxEvents(tx_hash) => {

                    // In case search event_type Applied return None then we will search with Accepted
                    match rpc::query_tx_events(&client, TxEventQuery::Applied(&tx_hash)).await {
                        Ok(Some(event)) => Ok(RPCResult::TxEvents(event)),
                        Ok(None) => {
                            match rpc::query_tx_events(&client, TxEventQuery::Accepted(&tx_hash)).await {
                                Ok(Some(event)) => Ok(RPCResult::TxEvents(event)),
                                Ok(None) => Err(error::Error::Other("Unable to find tx events for your transaction.".to_string())),
                                Err(err) => Err(error::Error::Other("Error to find tx events for your transaction.".to_string())),
                            }
                        }
                        Err(err) => Err(error::Error::Other("Error find tx events for your transaction".to_string())),
                    }
                }
                RPCRequestType::QueryNativeToken => rpc::query_native_token(&client).await.map(RPCResult::NativeToken),
                RPCRequestType::QueryLatestBlock => rpc::query_block(&client).await.map(RPCResult::LatestBlock),
                RPCRequestType::QueryCheckIsValidator(address) => rpc::is_validator(&client, &address)
                    .await
                    .map(RPCResult::IsValidator),
                RPCRequestType::QueryCheckIsDelegator(address) => rpc::is_delegator(&client, &address)
                    .await
                    .map(RPCResult::IsDelegator),
                RPCRequestType::QueryMaspReward => rpc::query_masp_reward_tokens(&client)
                    .await
                    .map(RPCResult::MapsReward),
                RPCRequestType::QueryTotalStakedTokens(epoch) => rpc::get_total_staked_tokens(&client, epoch)
                    .await
                    .map(RPCResult::TotalStakedTokens),
                RPCRequestType::QueryValidatorStaked(epoch, address) => rpc::get_validator_stake(&client, epoch, &address)
                    .await
                    .map(RPCResult::TotalStakedTokens),
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
                }
                RPCResult::ProposalVotes(votes) => {
                    let wrapped = votes.into_iter().map(|vote| {
                        VoteWrapper {
                            validator: format!("{}", vote.validator),
                            delegator: format!("{}", vote.delegator),
                            data: format!("{}", vote.data),
                        }
                    }).collect::<Vec<_>>();
                    Json(json!({ "data": wrapped }))
                }
                RPCResult::BalanceResult(amount) => Json(json!({ "balance": amount })),
                RPCResult::ValidatorState(maybe_validator_state) => match maybe_validator_state {
                    Some(validator_state) => {
                        match validator_state {
                            ValidatorState::Consensus => { Json(json!({ "state": "Consensus" })) }
                            ValidatorState::BelowCapacity => { Json(json!({ "state": "BelowCapacity" })) }
                            ValidatorState::BelowThreshold => { Json(json!({ "state": "BelowThreshold" })) }
                            ValidatorState::Inactive => { Json(json!({ "state": "Inactive" })) }
                            ValidatorState::Jailed => { Json(json!({ "state": "Jailed" })) }
                        }
                    }
                    None => {
                        Json(json!({ "state": "Your validator is either not a validator, \
                        or an epoch before the current epoch has been queried (and the validator state information is no longer stored)" }))
                    }
                },
                RPCResult::DelegatorDelegation(delegating) => Json(json!({ "data": delegating })),
                RPCResult::DelegatorDelegationAt(delegating) => Json(json!({ "data": delegating })),
                RPCResult::MetaData((meta_data, commission)) => {
                    let meta_data = meta_data.map_or(json!(null), |data| json!(data));
                    let commission = commission.map_or(json!(null), |comm| json!(CommissionPairWrapper {
                        commission_rate: format!("{}", comm.commission_rate),
                        max_commission_change_per_epoch: format!("{}", comm.max_commission_change_per_epoch)
                    }));
                    Json(json!({
                        "metadata": meta_data,
                        "commission": commission
                    }))
                }
                RPCResult::GovernanceParameters(governance) => {
                    let wrapped = GovernanceParametersWrapper {
                        min_proposal_fund: format!("{}", governance.min_proposal_fund),
                        max_proposal_code_size: format!("{}", governance.max_proposal_code_size),
                        min_proposal_voting_period: format!("{}", governance.min_proposal_voting_period),
                        max_proposal_period: format!("{}", governance.max_proposal_period),
                        max_proposal_content_size: format!("{}", governance.max_proposal_content_size),
                        min_proposal_grace_epochs: format!("{}", governance.min_proposal_grace_epochs),
                    };
                    Json(json!({ "data": wrapped }))
                }
                RPCResult::PosParameters(pos) => {
                    let serialized_pos_params = convert_to_serializable_pos(pos);
                    let json_pos_params = serde_json::to_value(&serialized_pos_params).unwrap();
                    Json(json!({ "data": json_pos_params }))
                }
                RPCResult::IsSteward(result) => Json(json!({ "data": result })),
                RPCResult::ValidatorConsensusKeys(result) => {
                    Json(json!({ "data": serialize(&result, serde_json::value::Serializer).unwrap() }))
                }
                RPCResult::TxEvents(event) => {
                    let serializable_event = to_serializable(event);
                    Json(json!({ "data": serializable_event }))
                }
                RPCResult::NativeToken(token) => Json(json!({ "address": token })),
                RPCResult::LatestBlock(last_block) => match last_block {
                    Some(last_block) => {
                        let serializable_block = block_to_serializable(&last_block);
                        let json_last_block = serde_json::to_value(&serializable_block).unwrap();
                        Json(json!({ "data": json_last_block }))
                    }
                    None => {
                        Json(json!({ "data": "Error to query latest block" }))
                    }
                },
                RPCResult::IsValidator(is_validator) => Json(json!({ "data": is_validator })),
                RPCResult::IsDelegator(is_delegator) => Json(json!({ "data": is_delegator })),
                RPCResult::MapsReward(rewards) => {
                    let wrapped = rewards.into_iter().map(|masp_reward| {
                        MaspTokenRewardDataWrapper {
                            name: masp_reward.name,
                            address: masp_reward.address,
                            max_reward_rate: masp_reward.max_reward_rate,
                            kp_gain: masp_reward.kp_gain,
                            kd_gain: masp_reward.kd_gain,
                            locked_amount_target: masp_reward.locked_amount_target,
                        }
                    }).collect::<Vec<_>>();
                    Json(json!({ "data": wrapped }))
                }
                RPCResult::TotalStakedTokens(amount) => Json(json!({ "total": amount })),
                RPCResult::ValidatorStake(amount) => Json(json!({ "total": amount }))
            }
        })
        .map_err(MyErrorWrapper)
}
