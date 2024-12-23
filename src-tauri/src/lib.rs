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
    BackendEvent,
    BackendRequest,
    CreateDelegationRequest,
    DecommissionPoolRequest,
    DelegateStakingRequest,
    EncryptionAction,
    SendDelegateToAddressRequest,
    SendRequest,
    StakeRequest,
    WalletId,
};
use node_gui_backend::{ BackendSender, ImportOrCreate, InitNetwork, WalletMode };
use once_cell::sync::OnceCell;
use request::{
    ConsoleRequest,
    DecommissionStakingPoolRequest,
    DelegationCreateRequest,
    NewAccountRequest,
    NewAddressRequest,
    OpenCreateWalletRequest,
    OpenWalletRequest,
    SendAmountRequest,
    SendDelegateRequest,
    StakeAmountRequest,
    StakingDelegateRequest,
    SubmitTransactionRequest,
    ToggleStakingRequest,
    UpdateEncryptionRequest,
};
use result::{
    BalanceResult,
    DelegateStakingResult,
    DelegationsBalanceResult,
    StakingBalanceResult,
    TransactionListResult,
    TransactionResult,
    WalletBestBlockResult,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::RwLock;
use tauri::{ AppHandle, Emitter };
use tokio::sync::mpsc::UnboundedReceiver;
use wallet_types::wallet_type::WalletType;

struct AppState {
    backend_sender: RwLock<Option<BackendSender>>,
}

static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

impl Default for AppState {
    fn default() -> Self {
        AppState {
            backend_sender: RwLock::new(None),
        }
    }
}

#[tauri::command]
async fn initialize_node(
    state: tauri::State<'_, AppState>,
    network: &str,
    mode: &str
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
    let backend_controls = node_gui_backend
        ::node_initialize(TimeGetter::default(), net_type, wallet_type).await
        .map_err(|e| e.to_string())?;

    let mut backend_sender_guard = state.backend_sender.write().await;
    *backend_sender_guard = Some(backend_controls.backend_sender);
    let node = Some(backend_controls.initialized_node);
    let backend_receiver = backend_controls.backend_receiver;
    let low_priority_backend_receiver = backend_controls.low_priority_backend_receiver;

    if let Some(node) = node {
        let node_arc = Arc::new(node);
        tokio::spawn(
            listen_backend_events(
                Arc::clone(&node_arc.chain_config),
                backend_receiver,
                low_priority_backend_receiver
            )
        );
        Ok(node_arc.chain_info.clone())
    } else {
        Err("backend is not initialized".into())
    }
}

fn process_message(message: BackendEvent, chain_config: ChainConfig) {
    match message {
        BackendEvent::P2p(msg) => {
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                app_handle.emit("P2p", msg).unwrap();
            }
        }
        BackendEvent::ChainInfo(msg) => {
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                app_handle.emit("ChainInfo", msg).unwrap();
            }
        }

        BackendEvent::ImportWallet(msg) => {
            match msg {
                Ok(wallet_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("ImportWallet", wallet_info).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }
        BackendEvent::OpenWallet(msg) => {
            match msg {
                Ok(wallet_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("OpenWallet", wallet_info).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }
        BackendEvent::SendAmount(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        let chain_config_ref = chain_config.clone();
                        let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                            Ok(json) => json,
                            Err(e) => { e.to_string().into() }
                        };
                        let transaction_result = TransactionResult::new(
                            transaction_info,
                            serialized_info
                        );
                        app_handle.emit("SendAmount", transaction_result).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }
        BackendEvent::NewAddress(msg) => {
            match msg {
                Ok(address_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("NewAddress", address_info).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }
        BackendEvent::UpdateEncryption(msg) => {
            match msg {
                Ok(encryption_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("UpdateEncryption", encryption_info).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }
        BackendEvent::CloseWallet(msg) => {
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                app_handle.emit("CloseWallet", msg).unwrap();
            }
        }
        BackendEvent::StakeAmount(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        let chain_config_ref = chain_config.clone();
                        let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                            Ok(json) => json,
                            Err(e) => { e.to_string().into() }
                        };
                        let transaction_result = TransactionResult::new(
                            transaction_info,
                            serialized_info
                        );
                        app_handle.emit("StakeAmount", transaction_result).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::DecommissionPool(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        let chain_config_ref = chain_config.clone();
                        let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                            Ok(json) => json,
                            Err(e) => { e.to_string().into() }
                        };
                        let transaction_result = TransactionResult::new(
                            transaction_info,
                            serialized_info
                        );
                        app_handle.emit("DecommissionPool", transaction_result).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::CreateDelegation(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        let chain_config_ref = chain_config.clone();
                        let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                            Ok(json) => json,
                            Err(e) => { e.to_string().into() }
                        };
                        let transaction_result = TransactionResult::new(
                            transaction_info,
                            serialized_info
                        );
                        app_handle.emit("CreateDelegation", transaction_result).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::DelegateStaking(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        let chain_config_ref = chain_config.clone();
                        let serialized_info = match
                            transaction_info.0.tx.to_json(&chain_config_ref)
                        {
                            Ok(json) => json,
                            Err(e) => { e.to_string().into() }
                        };
                        let transaction_result = DelegateStakingResult::new(
                            transaction_info.0,
                            serialized_info,
                            transaction_info.1
                        );
                        app_handle.emit("DelegateStaking", transaction_result).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::SendDelegationToAddress(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        let chain_config_ref = chain_config.clone();
                        let serialized_info = match transaction_info.tx.to_json(&chain_config_ref) {
                            Ok(json) => json,
                            Err(e) => { e.to_string().into() }
                        };
                        let transaction_result = TransactionResult::new(
                            transaction_info,
                            serialized_info
                        );
                        app_handle.emit("SendDelegationToAddress", transaction_result).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::NewAccount(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("NewAccount", transaction_info).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::ToggleStaking(msg) => {
            match msg {
                Ok(transaction_info) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("ToggleStaking", transaction_info).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::ConsoleResponse(_, _, result) => {
            match result {
                Ok(console_command) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("ConsoleResponse", console_command).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("ConsoleResponse", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::Broadcast(msg) => {
            match msg {
                Ok(wallet_id) => {
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Broadcast", wallet_id).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }
        BackendEvent::Balance(wallet_id, account_id, balance) => {
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                let balance = BalanceResult::new(wallet_id, account_id, balance);
                app_handle.emit("Balance", balance).unwrap();
            }
        }
        BackendEvent::StakingBalance(wallet_id, account_id, staking_balance) => {
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                let staking_balance = StakingBalanceResult::new(
                    wallet_id,
                    account_id,
                    staking_balance
                );
                app_handle.emit("StakingBalance", staking_balance).unwrap();
            }
        }
        BackendEvent::DelegationsBalance(wallet_id, account_id, delegations_balance) => {
            let chain_config_ref = chain_config.clone();
            let mut delegation_balances = BTreeMap::new();
            for (delegation_id, (pool_id, balance)) in delegations_balance {
                let delegation_address = match Address::new(&chain_config_ref, delegation_id) {
                    Ok(addr) => addr.as_str().to_string(),
                    Err(e) => { e.to_string() }
                };
                let pool_address = match Address::new(&chain_config_ref, pool_id) {
                    Ok(addr) => addr.as_str().to_string(),
                    Err(e) => { e.to_string() }
                };
                delegation_balances.insert(delegation_address, (pool_address, balance));
            }
            let delegations_balance = DelegationsBalanceResult::new(
                wallet_id,
                account_id,
                delegation_balances
            );
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                app_handle.emit("DelegationBalance", delegations_balance).unwrap();
            }
        }
        BackendEvent::TransactionList(wallet_id, account_id, msg) => {
            match msg {
                Ok(transaction_list) => {
                    let transaction_list_result = TransactionListResult::new(
                        wallet_id,
                        account_id,
                        transaction_list
                    );
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("TransactionList", transaction_list_result).unwrap();
                    }
                }
                Err(e) => {
                    let error_message = e.to_string();
                    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                        app_handle.emit("Error", error_message).unwrap();
                    }
                }
            }
        }

        BackendEvent::WalletBestBlock(wallet_id, block_info) => {
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                let wallet_best_block = WalletBestBlockResult::new(wallet_id, block_info);
                app_handle.emit("WalletBestBlock", wallet_best_block).unwrap();
            }
        }
    }
}

async fn listen_backend_events(
    chain_config: Arc<ChainConfig>,
    mut backend_receiver: UnboundedReceiver<BackendEvent>,
    mut low_priority_backend_receiver: UnboundedReceiver<BackendEvent>
) -> Result<(), String> {
    let chain_config = Arc::as_ref(&chain_config);
    loop {
        // Acquire a read lock only when receiving messages
        tokio::select! {
            biased;
                    msg_opt = backend_receiver.recv() =>{
                        match msg_opt {
                                    Some(msg_opt)=>{
                                        process_message(msg_opt, chain_config.clone());
                                    }

                                    None => {
                                        println!("No message received from backend");
                                    }
                                    
                                }
                    }
            
                    msg_opt = low_priority_backend_receiver.recv() =>{
                        match msg_opt {

                                    Some(msg_opt)=>{
                                        process_message(msg_opt, chain_config.clone());
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
    state: tauri::State<'_, AppState>,
    request: OpenCreateWalletRequest
) -> Result<(), String> {
    let mnemonic = wallet_controller::mnemonic::Mnemonic::parse(request.mnemonic).map_err(|e| {
        let error_message = e.to_string();
        error_message
    })?;

    let file_path = PathBuf::from(request.file_path);

    let wallet_type = match request.wallet_type.as_str() {
        "Hot" => WalletType::Hot,
        "Cold" => WalletType::Cold,
        &_ => todo!(),
    };

    let import = match request.import {
        true => ImportOrCreate::Import,
        false => ImportOrCreate::Create,
    };

    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::RecoverWallet {
        file_path: file_path,
        wallet_type: wallet_type,
        mnemonic: mnemonic,
        import: import,
    });
    Ok(())
}

#[tauri::command]
async fn add_open_wallet_wrapper(
    state: tauri::State<'_, AppState>,
    request: OpenWalletRequest
) -> Result<(), String> {
    let file_path = PathBuf::from(request.file_path);

    let wallet_type = match request.wallet_type.as_str() {
        "Hot" => WalletType::Hot,
        "Cold" => WalletType::Cold,
        &_ => todo!(),
    };
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::OpenWallet {
        file_path,
        wallet_type,
    });
    Ok(())
}

#[tauri::command]
async fn send_amount_wrapper(
    state: tauri::State<'_, AppState>,
    request: SendAmountRequest
) -> Result<(), String> {
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;

    let request = SendRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        amount: request.amount,
        address: request.address,
    };

    backend_sender.send(BackendRequest::SendAmount(request));
    Ok(())
}

#[tauri::command]
async fn new_address_wrapper(
    state: tauri::State<'_, AppState>,
    request: NewAddressRequest
) -> Result<(), String> {
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;

    backend_sender.send(BackendRequest::NewAddress(request.wallet_id, request.account_id));
    Ok(())
}

#[tauri::command]
async fn update_encryption_wrapper(
    state: tauri::State<'_, AppState>,
    request: UpdateEncryptionRequest
) -> Result<(), String> {
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
        &_ => todo!(), // Invalid action
    };

    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::UpdateEncryption {
        wallet_id: request.wallet_id,
        action: update_encryption_action,
    });
    Ok(())
}

#[tauri::command]
async fn close_wallet_wrapper(
    state: tauri::State<'_, AppState>,
    wallet_id: WalletId
) -> Result<(), String> {
    let wallet_id = WalletId::from(wallet_id);
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::CloseWallet(wallet_id));
    Ok(())
}

#[tauri::command]
async fn stake_amount_wrapper(
    state: tauri::State<'_, AppState>,
    request: StakeAmountRequest
) -> Result<(), String> {
    let stake_request = StakeRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pledge_amount: request.pledge_amount,
        mpt: request.mpt,
        cost_per_block: request.cost_per_block,
        decommission_address: request.decommission_address,
    };

    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::StakeAmount(stake_request));
    Ok(())
}

#[tauri::command]
async fn decommission_pool_wrapper(
    state: tauri::State<'_, AppState>,
    request: DecommissionStakingPoolRequest
) -> Result<(), String> {
    let decommission_request = DecommissionPoolRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pool_id: request.pool_id,
        output_address: request.output_address,
    };
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::DecommissionPool(decommission_request));
    Ok(())
}

#[tauri::command]
async fn create_delegation_wrapper(
    state: tauri::State<'_, AppState>,
    request: DelegationCreateRequest
) -> Result<(), String> {
    let delegation_request = CreateDelegationRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        pool_id: request.pool_id,
        delegation_address: request.delegation_address,
    };

    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::CreateDelegation(delegation_request));
    Ok(())
}

#[tauri::command]
async fn delegate_staking_wrapper(
    state: tauri::State<'_, AppState>,
    request: StakingDelegateRequest
) -> Result<(), String> {
    let delegation_request = DelegateStakingRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        delegation_id: request.delegation_id,
        delegation_amount: request.delegation_amount,
    };

    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::DelegateStaking(delegation_request));
    Ok(())
}

#[tauri::command]
async fn send_delegation_to_address_wrapper(
    state: tauri::State<'_, AppState>,
    request: SendDelegateRequest
) -> Result<(), String> {
    let send_delegation_request = SendDelegateToAddressRequest {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        address: request.address,
        amount: request.amount,
        delegation_id: request.delegation_id,
    };

    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::SendDelegationToAddress(send_delegation_request));
    Ok(())
}

#[tauri::command]
async fn new_account_wrapper(
    state: tauri::State<'_, AppState>,
    request: NewAccountRequest
) -> Result<(), String> {
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::NewAccount {
        wallet_id: request.wallet_id,
        name: request.name,
    });
    Ok(())
}

#[tauri::command]
async fn toggle_stakig_wrapper(
    state: tauri::State<'_, AppState>,
    request: ToggleStakingRequest
) -> Result<(), String> {
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(
        BackendRequest::ToggleStaking(request.wallet_id, request.account_id, request.enabled)
    );
    Ok(())
}

#[tauri::command]
async fn handle_console_command_wrapper(
    state: tauri::State<'_, AppState>,
    request: ConsoleRequest
) -> Result<(), String> {
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::ConsoleCommand {
        wallet_id: request.wallet_id,
        account_id: request.account_id,
        command: request.command,
    });
    Ok(())
}

#[tauri::command]
async fn submit_transaction_wrapper(
    state: tauri::State<'_, AppState>,
    request: SubmitTransactionRequest
) -> Result<(), String> {
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;

    backend_sender.send(BackendRequest::SubmitTx {
        wallet_id: request.wallet_id,
        tx: request.tx.tx,
    });
    Ok(())
}

#[tauri::command]
async fn shutdown_wrapper(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::Shutdown);
    Ok(())
}

pub fn run() {
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(
            tauri::generate_handler![
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
            ]
        )
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            match event {
                tauri::RunEvent::Ready => {
                    GLOBAL_APP_HANDLE.set(_app_handle.clone()).expect(
                        "Failed to set global app handle"
                    );
                }
                _ => {}
            }
        });
}
