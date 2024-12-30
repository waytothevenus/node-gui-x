// Copyright (c) 2024 RBB S.r.l
// opensource@mintlayer.org
// SPDX-License-Identifier: MIT
// Licensed under the MIT License;
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/mintlayer/mintlayer-core/blob/master/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc::UnboundedReceiver;

use common::{address::Address, chain::ChainConfig, time_getter::TimeGetter};
use node_gui_backend::{
    error::BackendError,
    messages::{
        BackendEvent, BackendRequest, CreateDelegationRequest, DecommissionPoolRequest,
        DelegateStakingRequest, EncryptionAction, SendDelegateToAddressRequest, SendRequest,
        StakeRequest, WalletId,
    },
};
use node_gui_backend::{ImportOrCreate, InitNetwork, WalletMode};
use wallet_types::wallet_type::WalletType;

use crate::{result::InitializationResult, AppState};

use super::request::{
    ConsoleRequest, DecommissionStakingPoolRequest, DelegationCreateRequest, NewAccountRequest,
    NewAddressRequest, OpenCreateWalletRequest, OpenWalletRequest, SendAmountRequest,
    SendDelegateRequest, StakeAmountRequest, StakingDelegateRequest, SubmitTransactionRequest,
    ToggleStakingRequest, UpdateEncryptionRequest,
};

use super::result::{
    BalanceResult, DelegateStakingResult, DelegationsBalanceResult, StakingBalanceResult,
    TransactionListResult, TransactionResult, WalletBestBlockResult,
};

#[tauri::command]
pub async fn initialize_node(
    state: State<'_, Mutex<AppState>>,
    network: &str,
    mode: &str,
) -> Result<InitializationResult, String> {
    let net_type = match network {
        "Mainnet" => InitNetwork::Mainnet,
        "Testnet" => InitNetwork::Testnet,
        _ => {
            return Err("Invalid network selection".into());
        }
    };
    let wallet_type = match mode {
        "Hot" => WalletMode::Hot,
        "Cold" => WalletMode::Cold,
        _ => {
            return Err("Invalid wallet mode selection".into());
        }
    };

    let backend_controls =
        node_gui_backend::node_initialize(TimeGetter::default(), net_type, wallet_type)
            .await
            .map_err(|e| e.to_string())?;

    let mut app_state = state.lock().expect("Failed to acquire the lock on the state");
    app_state.backend_sender = Some(backend_controls.backend_sender);
<<<<<<< HEAD
    app_state.chain_config = Some(backend_controls.initialized_node.chain_config.clone());

    // TODO: reconsider if the task should be joined
    tokio::spawn(listen_backend_events(
        app_state.app_handle.clone(),
        backend_controls.initialized_node.chain_config.clone(),
=======

    tokio::spawn(listen_backend_events(
        app_state.app_handle.clone(),
        backend_controls.initialized_node.chain_config.clone(),
        backend_controls.backend_receiver,
        backend_controls.low_priority_backend_receiver,
    ));

    Ok(InitializationResult::new(
        backend_controls.initialized_node.chain_info.clone(),
        backend_controls
            .initialized_node
            .chain_config
            .staking_pool_spend_maturity_block_count(
                backend_controls.initialized_node.chain_info.best_block_height,
            ),
    ))
}

<<<<<<< HEAD
async fn listen_backend_events(
=======
pub async fn listen_backend_events(
>>>>>>> 981c87b (fix(backend): apply patch result)
    app_handle: AppHandle,
    chain_config: Arc<ChainConfig>,
    mut backend_receiver: UnboundedReceiver<BackendEvent>,
    mut low_priority_backend_receiver: UnboundedReceiver<BackendEvent>,
) {
    loop {
        tokio::select! {
            biased;

            Some(event) = backend_receiver.recv() => {
                process_event(&app_handle, event, &chain_config);
            }
            Some(event) = low_priority_backend_receiver.recv() => {
                process_event(&app_handle, event, &chain_config);
            }
            else => {
                // Node is stopped
                return;
            }
        }
    }
}

<<<<<<< HEAD
#[tauri::command]
pub async fn get_stake_pool_maturity_distance(
    state: tauri::State<'_, Mutex<AppState>>,
    best_block_height: BlockHeight,
) -> Result<BlockCount, String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    Ok(state
        .chain_config
        .as_ref()
        .expect("must be initialized")
        .staking_pool_spend_maturity_block_count(best_block_height))
}

fn emit_event_or_error<T>(app_handle: &AppHandle, event_name: &str, r: Result<T, BackendError>)
where
    T: serde::Serialize + Clone + std::fmt::Debug,
{
    match r {
        Ok(data) => {
            app_handle.emit(event_name, data.clone()).expect("Failed to emit backend event");
=======
fn emit_event_or_error<T>(app_handle: &AppHandle, event_name: &str, r: Result<T, BackendError>)
where
    T: serde::Serialize + Clone + std::fmt::Debug,
{
    match r {
        Ok(data) => {
            app_handle.emit(event_name, data.clone()).expect("Failed to emit backend event");
                  }
        Err(e) => {
            app_handle.emit("Error", e.to_string()).expect("Failed to emit backend event");
        }
    }
}

fn process_event(app_handle: &AppHandle, event: BackendEvent, chain_config: &ChainConfig) {
    match event {
        BackendEvent::P2p(msg) => {
            emit_event_or_error(app_handle, "P2p", Ok(msg));
        }
        BackendEvent::ChainInfo(msg) => {
            emit_event_or_error(app_handle, "ChainInfo", Ok(msg));
        }
        BackendEvent::Balance(wallet_id, account_id, balance) => {
            let balance = BalanceResult::new(wallet_id, account_id, balance);
            emit_event_or_error(app_handle, "Balance", Ok(balance));
        }
        BackendEvent::StakingBalance(wallet_id, account_id, staking_balance) => {
            let staking_balance = StakingBalanceResult::new(wallet_id, account_id, staking_balance);
            emit_event_or_error(app_handle, "StakingBalance", Ok(staking_balance));
        }
        BackendEvent::WalletBestBlock(wallet_id, block_info) => {
            let wallet_best_block = WalletBestBlockResult::new(wallet_id, block_info);
            emit_event_or_error(app_handle, "WalletBestBlock", Ok(wallet_best_block));
        }
        BackendEvent::ImportWallet(msg) => {
            emit_event_or_error(app_handle, "ImportWallet", msg);
        }
        BackendEvent::OpenWallet(msg) => {
            emit_event_or_error(app_handle, "OpenWallet", msg);
        }
        BackendEvent::NewAddress(msg) => {
            emit_event_or_error(app_handle, "NewAddress", msg);
        }
        BackendEvent::UpdateEncryption(msg) => {
            emit_event_or_error(app_handle, "UpdateEncryption", msg);
        }
        BackendEvent::CloseWallet(msg) => {
            emit_event_or_error(app_handle, "CloseWallet", Ok(msg));
        }
        BackendEvent::NewAccount(msg) => {
            emit_event_or_error(app_handle, "NewAccount", msg);
        }
        BackendEvent::ToggleStaking(msg) => {
            emit_event_or_error(app_handle, "ToggleStaking", msg);
        }
        BackendEvent::ConsoleResponse(_, _, result) => match result {
            Ok(console_result) => {
                app_handle
                    .emit("ConsoleResponse", console_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                app_handle
                    .emit("ConsoleResponse", e.to_string())
                    .expect("Failed to emit backend event");
            }
        },
        BackendEvent::Broadcast(msg) => {
            emit_event_or_error(app_handle, "Broadcast", msg);
        }
        BackendEvent::SendAmount(msg) => {
            let result = msg.and_then(|transaction_info| {
                TransactionResult::from_transaction_info(chain_config, transaction_info)
            });
            emit_event_or_error(app_handle, "SendAmount", result);
        }
        BackendEvent::StakeAmount(msg) => {
            let result = msg.and_then(|transaction_info| {
                TransactionResult::from_transaction_info(chain_config, transaction_info)
            });
            emit_event_or_error(app_handle, "StakeAmount", result);
        }
        BackendEvent::DecommissionPool(msg) => {
            let result = msg.and_then(|transaction_info| {
                TransactionResult::from_transaction_info(chain_config, transaction_info)
            });
            emit_event_or_error(app_handle, "DecommissionPool", result);
        }
        BackendEvent::CreateDelegation(msg) => {
            let result = msg.and_then(|transaction_info| {
                TransactionResult::from_transaction_info(chain_config, transaction_info)
            });
            emit_event_or_error(app_handle, "CreateDelegation", result);
        }
        BackendEvent::DelegateStaking(msg) => {
            let result = msg.and_then(|transaction_info| {
                transaction_info.0.tx.to_json(chain_config).map(|serialized_info| {
                    DelegateStakingResult::new(
                        transaction_info.0,
                        serialized_info,
                        transaction_info.1,
                    )
                })
            });
            emit_event_or_error(app_handle, "DelegateStaking", result);
        }
        BackendEvent::SendDelegationToAddress(msg) => {
            let result = msg.and_then(|transaction_info| {
                TransactionResult::from_transaction_info(chain_config, transaction_info)
            });
            emit_event_or_error(app_handle, "SendDelegationToAddress", result);
        }
        BackendEvent::DelegationsBalance(wallet_id, account_id, delegations_balance) => {
            let delegations_balance = delegations_balance
                .into_iter()
                .map(|(delegation_id, (pool_id, balance))| {
                    let delegation_address =
                        Address::new(chain_config, delegation_id).expect("can't fail").to_string();
                    let pool_address =
                        Address::new(chain_config, pool_id).expect("can't fail").to_string();
                    (delegation_address, (pool_address, balance))
                })
                .collect();

            let delegations_balance =
                DelegationsBalanceResult::new(wallet_id, account_id, delegations_balance);

            emit_event_or_error(app_handle, "DelegationBalance", Ok(delegations_balance));
        }
        BackendEvent::TransactionList(wallet_id, account_id, msg) => {
            let result = msg.map(|transaction_list| {
                TransactionListResult::new(wallet_id, account_id, transaction_list)
            });
            emit_event_or_error(app_handle, "TransactionList", result);
        }
    }
}

#[tauri::command]
pub async fn add_create_wallet_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: OpenCreateWalletRequest,
) -> Result<(), String> {
    let mnemonic = wallet_controller::mnemonic::Mnemonic::parse(request.mnemonic)
        .map_err(|e| e.to_string())?;

    let file_path = PathBuf::from(request.file_path);

    let wallet_type = match request.wallet_type.as_str() {
        "Hot" => WalletType::Hot,
        "Cold" => WalletType::Cold,
        _ => return Err("Invalid wallet type".to_owned()),
    };

    let import = match request.import {
        true => ImportOrCreate::Import,
        false => ImportOrCreate::Create,
    };

    let state = state.lock().expect("Failed to acquire the lock on the state");

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::RecoverWallet {
            file_path,
            wallet_type,
            mnemonic,
            import,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn add_open_wallet_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: OpenWalletRequest,
) -> Result<(), String> {
    let file_path = PathBuf::from(request.file_path);

    let wallet_type = match request.wallet_type.as_str() {
        "Hot" => WalletType::Hot,
        "Cold" => WalletType::Cold,
        _ => return Err("Invalid wallet type".to_owned()),
    };

    let state = state.lock().expect("Failed to acquire the lock on the state");

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::OpenWallet {
            file_path,
            wallet_type,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn send_amount_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: SendAmountRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    let request = SendRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        amount: request.amount,
        address: request.address,
    };

    state
        .backend_sender
        .as_ref()
        .expect("Backend sender must be initialized")
        .send(BackendRequest::SendAmount(request));

    Ok(())
}

#[tauri::command]
pub async fn new_address_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: NewAddressRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::NewAddress(request.wallet_id, request.account_id),
    );

    Ok(())
}

#[tauri::command]
pub async fn update_encryption_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: UpdateEncryptionRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    let update_encryption_action = match request.action.to_lowercase().as_str() {
        "set_password" => {
            if let Some(pass) = request.password {
                EncryptionAction::SetPassword(pass.to_string())
            } else {
                return Err("Password cannot be empty".to_string());
            }
        }
        "remove_password" => EncryptionAction::RemovePassword,
        "unlock" => {
            if let Some(pass) = request.password {
                EncryptionAction::Unlock(pass.to_string())
            } else {
                return Err("Password cannot be empty".to_string());
            }
        }
        "lock" => EncryptionAction::Lock,
        _ => return Err("Invalid encryption action".to_owned()),
    };

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::UpdateEncryption {
            wallet_id: request.wallet_id,
            action: update_encryption_action,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn close_wallet_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    wallet_id: WalletId,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    state
        .backend_sender
        .as_ref()
        .expect("Backend sender must be initialized")
        .send(BackendRequest::CloseWallet(wallet_id));

    Ok(())
}

#[tauri::command]
pub async fn stake_amount_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: StakeAmountRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    let stake_request = StakeRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pledge_amount: request.pledge_amount,
        mpt: request.mpt,
        cost_per_block: request.cost_per_block,
        decommission_address: request.decommission_address,
    };

    state
        .backend_sender
        .as_ref()
        .expect("Backend sender must be initialized")
        .send(BackendRequest::StakeAmount(stake_request));

    Ok(())
}

#[tauri::command]
pub async fn decommission_pool_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: DecommissionStakingPoolRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    let decommission_request = DecommissionPoolRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pool_id: request.pool_id,
        output_address: request.output_address,
    };

    state
        .backend_sender
        .as_ref()
        .expect("Backend sender must be initialized")
        .send(BackendRequest::DecommissionPool(decommission_request));

    Ok(())
}

#[tauri::command]
pub async fn create_delegation_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: DelegationCreateRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    let delegation_request = CreateDelegationRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pool_id: request.pool_id,
        delegation_address: request.delegation_address,
    };

    state
        .backend_sender
        .as_ref()
        .expect("Backend sender must be initialized")
        .send(BackendRequest::CreateDelegation(delegation_request));

    Ok(())
}

#[tauri::command]
pub async fn delegate_staking_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: StakingDelegateRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    let delegation_request = DelegateStakingRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        delegation_id: request.delegation_id,
        delegation_amount: request.delegation_amount,
    };

    state
        .backend_sender
        .as_ref()
        .expect("Backend sender must be initialized")
        .send(BackendRequest::DelegateStaking(delegation_request));

    Ok(())
}

#[tauri::command]
pub async fn send_delegation_to_address_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: SendDelegateRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    let send_delegation_request = SendDelegateToAddressRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        address: request.address,
        amount: request.amount,
        delegation_id: request.delegation_id,
    };

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::SendDelegationToAddress(send_delegation_request),
    );

    Ok(())
}

#[tauri::command]
pub async fn new_account_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: NewAccountRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::NewAccount {
            wallet_id: request.wallet_id,
            name: request.name,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn toggle_staking_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: ToggleStakingRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::ToggleStaking(request.wallet_id, request.account_id, request.enabled),
    );
    Ok(())
}

#[tauri::command]
pub async fn handle_console_command_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: ConsoleRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");
    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::ConsoleCommand {
            wallet_id: request.wallet_id,
            account_id: request.account_id,
            command: request.command,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn submit_transaction_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: SubmitTransactionRequest,
) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    state.backend_sender.as_ref().expect("Backend sender must be initialized").send(
        BackendRequest::SubmitTx {
            wallet_id: request.wallet_id,
            tx: request.tx.tx,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn shutdown_wrapper(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = state.lock().expect("Failed to acquire the lock on the state");

    state
        .backend_sender
        .as_ref()
        .expect("Backend sender must be initialized")
        .send(BackendRequest::Shutdown);

    Ok(())
}
