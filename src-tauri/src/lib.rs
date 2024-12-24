// Copyright (c) 2023 RBB S.r.l
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

pub mod request;
pub mod result;
use chainstate::ChainInfo;
use common::address::Address;
use common::chain::ChainConfig;
use common::time_getter::TimeGetter;
use node_gui_backend::messages::{
    BackendEvent, BackendRequest, CreateDelegationRequest, DecommissionPoolRequest,
    DelegateStakingRequest, EncryptionAction, SendDelegateToAddressRequest, SendRequest,
    StakeRequest, WalletId,
};
use node_gui_backend::{BackendSender, ImportOrCreate, InitNetwork, WalletMode};
use request::{
    ConsoleRequest, DecommissionStakingPoolRequest, DelegationCreateRequest, NewAccountRequest,
    NewAddressRequest, OpenCreateWalletRequest, OpenWalletRequest, SendAmountRequest,
    SendDelegateRequest, StakeAmountRequest, StakingDelegateRequest, SubmitTransactionRequest,
    ToggleStakingRequest, UpdateEncryptionRequest,
};
use result::{
    BalanceResult, DelegateStakingResult, DelegationsBalanceResult, StakingBalanceResult,
    TransactionListResult, TransactionResult, WalletBestBlockResult,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc::UnboundedReceiver;
use wallet_types::wallet_type::WalletType;

#[derive(Default)]
struct AppState {
    backend_sender: Option<BackendSender>,
    app_handle: Option<AppHandle>,
}

#[tauri::command]
async fn initialize_node(
    state: State<'_, Mutex<AppState>>, // Make state mutable
    network: &str,
    mode: &str,
) -> Result<ChainInfo, String> {
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

    // Now you can modify the state
    let mut app_state = state.lock().expect("Failed to acquire the lock on the state");
    app_state.backend_sender = Some(backend_controls.backend_sender);
    let node = Some(backend_controls.initialized_node);
    let backend_receiver = backend_controls.backend_receiver;
    let low_priority_backend_receiver = backend_controls.low_priority_backend_receiver;

    if let Some(node) = node {
        let node_arc = Arc::new(node);
        if let Some(app_handle) = &app_state.app_handle.clone() {
            tokio::spawn(listen_backend_events(
                app_handle.clone(),
                Arc::clone(&node_arc.chain_config),
                backend_receiver,
                low_priority_backend_receiver,
            ));
        }
        Ok(node_arc.chain_info.clone())
    } else {
        Err("backend is not initialized".into())
    }
}

fn handle_event<T>(app_handle: AppHandle, event_name: &str, msg: Result<T, impl std::fmt::Display>)
where
    T: serde::Serialize + Clone,
{
    // Assuming T can be serialized to JSON
    match msg {
        Ok(data) => {
            app_handle.emit(event_name, data).expect("Failed to emit backend event");
        }
        Err(e) => {
            let error_message = e.to_string();
            app_handle.emit("Error", error_message).expect("Failed to emit backend event");
        }
    }
}

fn process_message(app_handle: AppHandle, message: BackendEvent, chain_config: ChainConfig) {
    match message {
        BackendEvent::P2p(msg) => {
            app_handle.emit("P2p", msg).expect("Failed to emit backend event");
        }
        BackendEvent::ChainInfo(msg) => {
            app_handle.emit("ChainInfo", msg).expect("Failed to emit backend event");
        }
        BackendEvent::Balance(wallet_id, account_id, balance) => {
            let balance = BalanceResult::new(wallet_id, account_id, balance);
            app_handle.emit("Balance", balance).expect("Failed to emit backend event");
        }
        BackendEvent::StakingBalance(wallet_id, account_id, staking_balance) => {
            let staking_balance = StakingBalanceResult::new(wallet_id, account_id, staking_balance);
            app_handle
                .emit("StakingBalance", staking_balance)
                .expect("Failed to emit backend event");
        }
        BackendEvent::WalletBestBlock(wallet_id, block_info) => {
            let wallet_best_block = WalletBestBlockResult::new(wallet_id, block_info);
            app_handle
                .emit("WalletBestBlock", wallet_best_block)
                .expect("Failed to emit backend event");
        }

        BackendEvent::ImportWallet(msg) => {
            handle_event(app_handle, "ImportWallet", msg);
        }
        BackendEvent::OpenWallet(msg) => {
            handle_event(app_handle, "OpenWallet", msg);
        }
        BackendEvent::NewAddress(msg) => {
            handle_event(app_handle, "NewAddress", msg);
        }
        BackendEvent::UpdateEncryption(msg) => {
            handle_event(app_handle, "UpdateEncryption", msg);
        }
        BackendEvent::CloseWallet(msg) => {
            app_handle.emit("CloseWallet", msg).expect("Failed to emit backend event");
        }
        BackendEvent::NewAccount(msg) => {
            handle_event(app_handle, "NewAccount", msg);
        }

        BackendEvent::ToggleStaking(msg) => {
            handle_event(app_handle, "ToggleStaking", msg);
        }

        BackendEvent::ConsoleResponse(_, _, result) => {
            handle_event(app_handle, "ConsoleResponse", result)
        }

        BackendEvent::Broadcast(msg) => {
            handle_event(app_handle, "Broadcast", msg);
        }

        BackendEvent::SendAmount(msg) => match msg {
            Ok(transaction_info) => {
                let chain_config_ref = chain_config.clone();
                let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                    Ok(json) => json,
                    Err(e) => e.to_string().into(),
                };
                let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                app_handle
                    .emit("SendAmount", transaction_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                let error_message = e.to_string();
                app_handle.emit("Error", error_message).expect("Failed to emit backend event");
            }
        },

        BackendEvent::StakeAmount(msg) => match msg {
            Ok(transaction_info) => {
                let chain_config_ref = chain_config.clone();
                let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                    Ok(json) => json,
                    Err(e) => e.to_string().into(),
                };
                let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                app_handle
                    .emit("StakeAmount", transaction_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                let error_message = e.to_string();
                app_handle.emit("Error", error_message).expect("Failed to emit backend event");
            }
        },

        BackendEvent::DecommissionPool(msg) => match msg {
            Ok(transaction_info) => {
                let chain_config_ref = chain_config.clone();
                let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                    Ok(json) => json,
                    Err(e) => e.to_string().into(),
                };
                let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                app_handle
                    .emit("DecommissionPool", transaction_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                let error_message = e.to_string();
                app_handle.emit("Error", error_message).expect("Failed to emit backend event");
            }
        },

        BackendEvent::CreateDelegation(msg) => match msg {
            Ok(transaction_info) => {
                let chain_config_ref = chain_config.clone();
                let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                    Ok(json) => json,
                    Err(e) => e.to_string().into(),
                };
                let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                app_handle
                    .emit("CreateDelegation", transaction_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                let error_message = e.to_string();
                app_handle.emit("Error", error_message).expect("Failed to emit backend event");
            }
        },

        BackendEvent::DelegateStaking(msg) => match msg {
            Ok(transaction_info) => {
                let chain_config_ref = chain_config.clone();
                let serialized_info = match transaction_info.0.tx.to_json(&chain_config_ref) {
                    Ok(json) => json,
                    Err(e) => e.to_string().into(),
                };
                let transaction_result = DelegateStakingResult::new(
                    transaction_info.0,
                    serialized_info,
                    transaction_info.1,
                );
                app_handle
                    .emit("DelegateStaking", transaction_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                let error_message = e.to_string();
                app_handle.emit("Error", error_message).expect("Failed to emit backend event");
            }
        },

        BackendEvent::SendDelegationToAddress(msg) => match msg {
            Ok(transaction_info) => {
                let chain_config_ref = chain_config.clone();
                let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                    Ok(json) => json,
                    Err(e) => e.to_string().into(),
                };
                let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                app_handle
                    .emit("SendDelegationToAddress", transaction_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                let error_message = e.to_string();
                app_handle.emit("Error", error_message).expect("Failed to emit backend event");
            }
        },

        BackendEvent::DelegationsBalance(wallet_id, account_id, delegations_balance) => {
            let chain_config_ref = chain_config.clone();
            let mut delegation_balances = BTreeMap::new();
            for (delegation_id, (pool_id, balance)) in delegations_balance {
                let delegation_address = match Address::new(&chain_config_ref, delegation_id) {
                    Ok(addr) => addr.as_str().to_string(),
                    Err(e) => e.to_string(),
                };
                let pool_address = match Address::new(&chain_config_ref, pool_id) {
                    Ok(addr) => addr.as_str().to_string(),
                    Err(e) => e.to_string(),
                };
                delegation_balances.insert(delegation_address, (pool_address, balance));
            }
            let delegations_balance =
                DelegationsBalanceResult::new(wallet_id, account_id, delegation_balances);
            app_handle
                .emit("DelegationBalance", delegations_balance)
                .expect("Failed to emit backend event");
        }
        BackendEvent::TransactionList(wallet_id, account_id, msg) => match msg {
            Ok(transaction_list) => {
                let transaction_list_result =
                    TransactionListResult::new(wallet_id, account_id, transaction_list);
                app_handle
                    .emit("TransactionList", transaction_list_result)
                    .expect("Failed to emit backend event");
            }
            Err(e) => {
                let error_message = e.to_string();
                app_handle.emit("Error", error_message).expect("Failed to emit backend event");
            }
        },
    }
}

async fn listen_backend_events(
    app_handle: AppHandle,
    chain_config: Arc<ChainConfig>,
    mut backend_receiver: UnboundedReceiver<BackendEvent>,
    mut low_priority_backend_receiver: UnboundedReceiver<BackendEvent>,
) -> Result<(), String> {
    let chain_config = Arc::as_ref(&chain_config);
    loop {
        // Acquire a read lock only when receiving messages
        tokio::select! {
        biased;
                msg_opt = backend_receiver.recv() =>{
                    match msg_opt {
                                Some(msg_opt)=>{
                                    process_message(app_handle.clone(), msg_opt, chain_config.clone());
                                }

                                None => {
                                    println!("No message received from backend");
                                }

                            }
                }

                msg_opt = low_priority_backend_receiver.recv() =>{
                    match msg_opt {

                                Some(msg_opt)=>{
                                    process_message(app_handle.clone(), msg_opt, chain_config.clone());
                                }
                                None => {
                                    println!("No message received from backend");
                                }
                            }
                }
            }
    }
}

#[tauri::command]
async fn add_create_wallet_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: OpenCreateWalletRequest,
) -> Result<(), String> {
    let mnemonic = wallet_controller::mnemonic::Mnemonic::parse(request.mnemonic)
        .map_err(|e| e.to_string())?;

    let file_path = PathBuf::from(request.file_path);

    let wallet_type = match request.wallet_type.as_str() {
        "Hot" => WalletType::Hot,
        "Cold" => WalletType::Cold,
        _ => WalletType::Cold,
    };

    let import = match request.import {
        true => ImportOrCreate::Import,
        false => ImportOrCreate::Create,
    };
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::RecoverWallet {
            file_path,
            wallet_type,
            mnemonic,
            import,
        });
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn add_open_wallet_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: OpenWalletRequest,
) -> Result<(), String> {
    let file_path = PathBuf::from(request.file_path);

    let wallet_type = match request.wallet_type.as_str() {
        "Hot" => WalletType::Hot,
        "Cold" => WalletType::Cold,
        &_ => WalletType::Cold,
    };
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");
    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::OpenWallet {
            file_path,
            wallet_type,
        });
    } else {
        return Err("Backend sender is not initialized".into());
    };
    Ok(())
}

#[tauri::command]
async fn send_amount_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: SendAmountRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        let request = SendRequest {
            wallet_id: request.wallet_id,
            account_id: request.account_id,
            amount: request.amount,
            address: request.address,
        };

        backend_sender.send(BackendRequest::SendAmount(request));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn new_address_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: NewAddressRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::NewAddress(
            request.wallet_id,
            request.account_id,
        ));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn update_encryption_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: UpdateEncryptionRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

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
        &_ => EncryptionAction::Lock, // Invalid action
    };

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::UpdateEncryption {
            wallet_id: request.wallet_id,
            action: update_encryption_action,
        });
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn close_wallet_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    wallet_id: WalletId,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::CloseWallet(wallet_id));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn stake_amount_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: StakeAmountRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    let stake_request = StakeRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pledge_amount: request.pledge_amount,
        mpt: request.mpt,
        cost_per_block: request.cost_per_block,
        decommission_address: request.decommission_address,
    };

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::StakeAmount(stake_request));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn decommission_pool_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: DecommissionStakingPoolRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    let decommission_request = DecommissionPoolRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pool_id: request.pool_id,
        output_address: request.output_address,
    };
    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::DecommissionPool(decommission_request));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn create_delegation_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: DelegationCreateRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    let delegation_request = CreateDelegationRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pool_id: request.pool_id,
        delegation_address: request.delegation_address,
    };

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::CreateDelegation(delegation_request));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn delegate_staking_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: StakingDelegateRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    let delegation_request = DelegateStakingRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        delegation_id: request.delegation_id,
        delegation_amount: request.delegation_amount,
    };

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::DelegateStaking(delegation_request));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn send_delegation_to_address_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: SendDelegateRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    let send_delegation_request = SendDelegateToAddressRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        address: request.address,
        amount: request.amount,
        delegation_id: request.delegation_id,
    };

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::SendDelegationToAddress(
            send_delegation_request,
        ));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn new_account_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: NewAccountRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::NewAccount {
            wallet_id: request.wallet_id,
            name: request.name,
        });
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn toggle_stakig_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: ToggleStakingRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::ToggleStaking(
            request.wallet_id,
            request.account_id,
            request.enabled,
        ));
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn handle_console_command_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: ConsoleRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::ConsoleCommand {
            wallet_id: request.wallet_id,
            account_id: request.account_id,
            command: request.command,
        });
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn submit_transaction_wrapper(
    state: tauri::State<'_, Mutex<AppState>>,
    request: SubmitTransactionRequest,
) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::SubmitTx {
            wallet_id: request.wallet_id,
            tx: request.tx.tx,
        });
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

#[tauri::command]
async fn shutdown_wrapper(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state: std::sync::MutexGuard<'_, AppState> =
        state.lock().expect("Failed to acquire the lock on the state");

    if let Some(backend_sender) = &state.backend_sender {
        backend_sender.send(BackendRequest::Shutdown);
    } else {
        return Err("Backend sender is not initialized".into());
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(AppState::default())) // Wrap AppState in Mutex
        .invoke_handler(tauri::generate_handler![
            initialize_node,
            add_create_wallet_wrapper,
            add_open_wallet_wrapper,
            send_amount_wrapper,
            new_address_wrapper,
            update_encryption_wrapper,
            close_wallet_wrapper,
            stake_amount_wrapper,
            decommission_pool_wrapper,
            create_delegation_wrapper,
            delegate_staking_wrapper,
            send_delegation_to_address_wrapper,
            new_account_wrapper,
            toggle_stakig_wrapper,
            handle_console_command_wrapper,
            submit_transaction_wrapper,
            shutdown_wrapper
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Ready = event {
                {
                    let state = app_handle.state::<Mutex<AppState>>();
                    let mut state = state.lock().expect("Failed to lock AppState");
                    state.app_handle = Some(app_handle.clone());
                }
            }
        });
}
