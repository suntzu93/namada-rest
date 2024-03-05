# Namada REST API

Welcome, Earthling! 🌍👽 You've just docked at the most stellar REST API in the Milky Way: The **Namada REST API**. Serving data at the speed of light across the cosmos, our API is the go-to source for all your interstellar governance and epoch inquiries. Strap in as we guide you through setting up your own galactic data station.

## Prerequisites

Before launching, make sure you have the following onboard:

- Rust: The metal of the cosmos. Ensure you're equipped with the latest stable version to avoid any unexpected asteroid fields.
- Cargo: Your cargo hold for Rust crates. It comes bundled with Rust, so no need for extra spacewalks.

## Setup

Clone this repository to your local star system:

```bash
git clone https://github.com/suntzu93/namada-rest.git
cd namada-rest-api
```

Configure your spacecraft with the necessary settings by editing `config/Settings.toml`. Don't worry; it's not rocket science! Just specify your `rpc_url`, `bind_ip`, and `port`.

## Launching

To ignite the engines and start serving requests across the galaxy, run:

```bash
cargo run
```

Your console will light up with the message: "Server listening [bind_ip]:[port]", indicating that you're now broadcasting to the universe.

## Making Contact

Communicate with the API using your preferred space communication tools (like `curl` or Postman). Here are some examples:

 API                              | #Description                                                                                        | #Output                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
----------------------------------|-----------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
 /epoch                           | Retrieves the current epoch data.                                                                   | ```{"epoch":23}```                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
 /epoch_at_height/{height}        | Query the epoch of the given block height.                                                          | ```{"epoch":23}```                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
 /proposal_result/{id}            | Dives into the proposal results for a given ID.                                                     | ```{"result":"rejected","thresh_frac":"0.333333333333","threshold":"97251865402272",<br>"total_abstain_power":"1900000000","total_nay_power":"29048868607602",<br>"total_voting_power":"291755596207107","total_yay_power":"64272922117602"}                                                                                                                                                                                                                                                                              ```|
 /proposal_votes/{id}            | Get all the votes of a proposal                                                                     | ```{"data":[{"data":"nay","delegator":"tnam1qqlp6rdaxcwyp8m4pf8tsdywtruzwlw5tq639jnk","validator":"tnam1q823fn35rwten24a8j6afpygp7shttmvuupfg5t3"},{"data":"yay","delegator":"tnam1q8243m5rnsls5jn4yvv0ycjp46a0emppzvzs7j3n","validator":"tnam1q8243m5rnsls5jn4yvv0ycjp46a0emppzvzs7j3n"},{"data":"nay","delegator":"tnam1qzdfys6q5nngrcvlw9kf7ykk90ds62ap3yuhcusx","validator":"tnam1q8243m5rnsls5jn4yvv0ycjp46a0emppzvzs7j3n"}...]```                                                                               |
 /balance/{wallet}                | Query token amount of owner.                                                                        | ```{"balance":"5426772897"}```                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
 /validator_state/{address}/{epoch} | Get the given validator's stake at the given epoch                                                  | ```{"state":"Consensus"}``` or ```{"state":"BelowCapacity"}``` or ```{"state":"BelowThreshold"}``` or ```{"state":"Inactive"}``` or ```{"state":"Jailed"}```                                                                                                                                                                                                                                                                                                                                                          |
 /delegator_delegation/{wallet} | Get the delegator's delegation                                                                      | ```{"data":["tnam1q88rs4me28lm3z5mzf0u0k6z4jrfmy3yzvt85kf8","tnam1q9l24sylqfwzjzjclnaj0kv8ythmdgw8mvde5nrf"]}```                                                                                                                                                                                                                                                                                                                                                                                                      |
 /delegator_delegation_at/{wallet}/{epoch} | Get the delegator's delegation include amount at some epoch                                         | ```{"data":{"tnam1q88rs4me28lm3z5mzf0u0k6z4jrfmy3yzvt85kf8":"27959000000","tnam1q9l24sylqfwzjzjclnaj0kv8ythmdgw8mvde5nrf":"0"}}```                                                                                                                                                                                                                                                                                                                                                                                    |
 /metadata/:address/{epoch} | Query and return validator's metadata, including the commission rate and max commission rate change | ```{"commission":{"commission_rate":"0.11","max_commission_change_per_epoch":"1"},"metadata":{"avatar":null,"description":null,"discord_handle":null,"email":"suntzu@gmail.com","website":null}}```                                                                                                                                                                                                                                                                                                                   |
 /governance | Get the governance parameters                                                                       | ```{"data":{"max_proposal_code_size":"600000","max_proposal_content_size":"10000","max_proposal_period":"6","min_proposal_fund":"5000000000","min_proposal_grace_epochs":"2","min_proposal_voting_period":"2"}}```                                                                                                                                                                                                                                                                                                    |
 /pos_params | Get the PoS parameters                                                                              | ```{"data":{"max_proposal_period":6,"owned":{"block_proposer_reward":"0.125","block_vote_reward":"0.1","cubic_slashing_window_length":1,"duplicate_vote_min_slash_rate":"0.001","light_client_attack_min_slash_rate":"0.001","liveness_threshold":"0.9","liveness_window_check":8640,"max_inflation_rate":"0","max_validator_slots":257,"pipeline_len":2,"rewards_gain_d":"0","rewards_gain_p":"0","target_staked_ratio":"0","tm_votes_per_token":"1","unbonding_len":4,"validator_stake_threshold":"1000000000"}}}``` |
 /is_steward/{wallet} | Check if the given address is a pgf steward.                                                        | ```{"data":false}``` |
 /validator_consensus_keys/{wallet} | Query the consensus key by validator address                                                        | ```{"data":"3f18a7eca7bd771bde7b656d2f7ae226793ee2f28237d0d037dc91afe6816007"}``` |
 /tx_event/{tx_hash} | Fetch the current status of a transaction.                                                          | ```{"data":{"attributes":{"code":"0","gas_used":"7263","hash":"3E0702526E372BEF45E0F21B1A0759826322AE26AF67D9F337CCE177ADE230E4","height":"90044","info":"Check inner_tx for result.","inner_tx":"{\"gas_used\":{\"sub\":72622035},\"changed_keys\":[{\"segments\":[{\"AddressSeg\":\"tnam1q5qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqrw33g6\"},{\"StringSeg\":\"proposal\"},{\"StringSeg\":\"247\"},{\"StringSeg\":\"vote\"},{\"AddressSeg\":\"tnam1q8c5j8gjaw7yz2fllzputv2cfn9jjktlzgyezv44\"},{\"AddressSeg\":\"tnam1qpg3k3rfe2qjr74l53j4c0naa8x34cza4gl78htw\"}]}],\"vps_result\":{\"accepted_vps\":[\"tnam1q5qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqrw33g6\",\"tnam1q8c5j8gjaw7yz2fllzputv2cfn9jjktlzgyezv44\",\"tnam1qpg3k3rfe2qjr74l53j4c0naa8x34cza4gl78htw\"],\"rejected_vps\":[],\"gas_used\":{\"max\":{\"sub\":39187674},\"rest\":[{\"sub\":0},{\"sub\":0},{\"sub\":297288},{\"sub\":0},{\"sub\":0},{\"sub\":38403877},{\"sub\":0},{\"sub\":0}]},\"errors\":[],\"invalid_sig\":false},\"initialized_accounts\":[],\"ibc_events\":[],\"eth_bridge_events\":[]}","log":""},"event_type":"Applied","level":"Tx"}}``` |
 /native_token | Query the address of the native token                                                               | ```{"address":"tnam1qxvg64psvhwumv3mwrrjfcz0h3t3274hwggyzcee"}``` |
 /query_block | Query the last committed block, if any.                                                             | ```{"data":{"hash":[148,251,113,202,153,15,202,36,63,217,228,59,187,247,170,4,164,246,144,101,187,116,206,234,101,16,193,70,120,136,41,165],"height":90044,"time":"2024-02-28T20:17:58.704534371+00:00"}}``` |
 /is_validator{address} | Check if the given address is a known validator.                                                    | ```{"data":false}``` |
 /is_delegator{address} | Check if the given address is a known delegator.                                                    | ```{"data":false}``` |
 /masp_reward | Query to read the tokens that earn masp rewards.                                                    | ```{"data":[{"address":"tnam1qxvg64psvhwumv3mwrrjfcz0h3t3274hwggyzcee","kd_gain":"0","kp_gain":"0","locked_amount_target":"0","max_reward_rate":"0","name":"naan"}]}``` |
 /total_staked/{epoch} | Get the total staked tokens in the given epoch.                                                     | ```{"total":"240903728697679"}``` |
 /validator_stake/{address}/{epoch} | Get the given validator's stake at the given epoch.                                                 | ```{"total":"28647000000"}``` |


Remember, with great power comes great responsibility. Use this API wisely to maintain peace and prosperity across the galaxies.

## Contributing

Found a wormhole to a new feature or spotted an asteroid of a bug? Open a pull request or issue. Contributions are more welcome than a water planet in a desert solar system!

## License

This project is under the galaxy's most powerful and freedom-respecting license: the GNU General Public License (GPL). For more details, warp to the `LICENSE` file.

## Acknowledgments

- The Galactic Council of Rustaceans for their guidance and wisdom.
- Earthlings and extraterrestrial beings who contributed to this project.

## Farewell

May your queries be swift and your data vast. Happy exploring, space coder! 🚀✨
