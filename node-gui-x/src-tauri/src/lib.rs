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

use chainstate::ChainInfo;
use common::chain::{ DelegationId, PoolId };
use common::primitives::Amount;
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
    TransactionInfo,
    WalletId,
};
use node_gui_backend::AccountId;
use node_gui_backend::{ BackendSender, ImportOrCreate, InitNetwork, InitializedNode, WalletMode };
use once_cell::sync::OnceCell;
use serde::{ Deserialize, Serialize };
use serde_json::Value;
use wallet_rpc_lib::types::{ Balances, PoolInfo };
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::RwLock;
use tauri::{ AppHandle, Emitter };
use tokio::sync::mpsc::UnboundedReceiver;
use wallet_types::wallet_type::WalletType;
struct AppState {
    initialized_node: RwLock<Option<InitializedNode>>,
    backend_sender: RwLock<Option<BackendSender>>,
    backend_receiver: RwLock<Option<UnboundedReceiver<BackendEvent>>>,
    low_priority_backend_receiver: RwLock<Option<UnboundedReceiver<BackendEvent>>>,
}

static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize)]
struct OpenCreateWalletRequest {
    mnemonic: String,
    file_path: String,
    import: bool,
    wallet_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenWalletRequest {
    file_path: String,
    wallet_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendAmountRequest {
    wallet_id: WalletId,
    account_id: AccountId,
    amount: String,
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StakeAmountRequest {
    wallet_id: WalletId,
    account_id: AccountId,
    pledge_amount: String,
    mpt: String,
    cost_per_block: String,
    decommission_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DecommissionStakingPoolRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub pool_id: String,
    pub output_address: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct DelegationCreateRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub pool_id: String,
    pub delegation_address: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct StakingDelegateRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub delegation_id: DelegationId,
    pub delegation_amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewAddressRequest {
    wallet_id: WalletId,
    account_id: AccountId,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateEncryptionRequest {
    wallet_id: WalletId,
    action: String,
    password: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct SendDelegateRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub address: String,
    pub amount: String,
    pub delegation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewAccountRequest {
    wallet_id: WalletId,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToggleStakingRequest {
    wallet_id: WalletId,
    account_id: AccountId,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConsoleRequest {
    wallet_id: WalletId,
    account_id: AccountId,
    command: String,
}

#[derive(Debug, Deserialize)]
struct SubmitTransactionRequest {
    tx: TransactionInfo,
    wallet_id: WalletId,
}

#[derive(Debug, Clone, Serialize)]
struct TransactionResult {
    transaction_info: TransactionInfo,
    serialized_tx: Value,
}

impl TransactionResult {
    pub fn new(transaction_info: TransactionInfo, serialized_tx: Value) -> Self {
        TransactionResult {
            transaction_info,
            serialized_tx,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct DelegateStakingResult {
    transaction_info: TransactionInfo,
    serialized_tx: Value,
    delegation_id: DelegationId,
}

impl DelegateStakingResult {
    pub fn new(
        transaction_info: TransactionInfo,
        serialized_tx: Value,
        delegation_id: DelegationId
    ) -> Self {
        DelegateStakingResult {
            transaction_info,
            serialized_tx,
            delegation_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct StakingBalanceResult {
    wallet_id: WalletId,
    account_id: AccountId,
    staking_balance: BTreeMap<PoolId, PoolInfo>,
}

impl StakingBalanceResult {
    fn new(
        wallet_id: WalletId,
        account_id: AccountId,
        staking_balance: BTreeMap<PoolId, PoolInfo>
    ) -> Self {
        StakingBalanceResult {
            wallet_id,
            account_id,
            staking_balance,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct BalanceResult {
    wallet_id: WalletId,
    account_id: AccountId,
    balance: Balances,
}

impl BalanceResult {
    fn new(wallet_id: WalletId, account_id: AccountId, balance: Balances) -> Self {
        BalanceResult {
            wallet_id,
            account_id,
            balance,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct DelegationsBalanceResult {
    wallet_id: WalletId,
    account_id: AccountId,
    delegations_balance: BTreeMap<DelegationId, (PoolId, Amount)>,
}

impl DelegationsBalanceResult {
    fn new(
        wallet_id: WalletId,
        account_id: AccountId,
        delegations_balance: BTreeMap<DelegationId, (PoolId, Amount)>
    ) -> Self {
        DelegationsBalanceResult {
            wallet_id,
            account_id,
            delegations_balance,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            initialized_node: RwLock::new(None),
            backend_receiver: RwLock::new(None),
            backend_sender: RwLock::new(None),
            low_priority_backend_receiver: RwLock::new(None),
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

    let mut guard = state.initialized_node.write().await;
    *guard = Some(backend_controls.initialized_node);

    let mut backend_sender_guard = state.backend_sender.write().await;
    *backend_sender_guard = Some(backend_controls.backend_sender);

    let mut backend_receiver_guard = state.backend_receiver.write().await;
    *backend_receiver_guard = Some(backend_controls.backend_receiver);

    let mut low_priority_backend_receiver_guard = state.low_priority_backend_receiver.write().await;
    *low_priority_backend_receiver_guard = Some(backend_controls.low_priority_backend_receiver);

    if let Some(node) = &*guard {
        Ok(node.chain_info.clone())
    } else {
        Err("backend is not initialized".into())
    }
}

#[tauri::command]
async fn listen_events(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let state_clone = state.clone(); // Clone the state to move into the async block
    loop {
        // Acquire a read lock only when receiving messages
        let mut node_guard = state_clone.initialized_node.write().await;
        let node = node_guard.as_mut().expect("Node not initialized");
        let mut backend_receiver_guard = state_clone.backend_receiver.write().await;
        let backend_receiver = backend_receiver_guard
            .as_mut()
            .expect("Backend receiver not initialized");
        let mut low_priority_backend_receiver_guard =
            state_clone.low_priority_backend_receiver.write().await;
        let low_priority_backend_receiver = low_priority_backend_receiver_guard
            .as_mut()
            .expect("Low priority event receiver not initialized");

        tokio::select! {
                    msg_opt = backend_receiver.recv() =>{
                        println!("Backend event received {:?}", msg_opt.clone());
        match msg_opt {
                    Some(BackendEvent::P2p(msg)) => {
                        println!("P2P event received {:?}", msg);
                        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                            app_handle.emit("P2p", msg).unwrap();
                        }
                    }
                    Some(BackendEvent::ChainInfo(msg)) => {
                        println!("ChainInfo event received {:?}", msg);
                        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                            app_handle.emit("ChainInfo", msg).unwrap();
                        }
                    }
                    
                    Some(BackendEvent::ImportWallet(msg)) => {
                        match msg {
                            Ok(wallet_info) => {
                                println!("Wallet created successfully: {:?}", wallet_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("ImportWallet", wallet_info).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error creating wallet: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }
                    Some(BackendEvent::OpenWallet(msg)) => {
                        match msg {
                            Ok(wallet_info) => {
                                println!("Wallet Opened successfully: {:?}", wallet_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("OpenWallet", wallet_info).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error creating wallet: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }
                    Some(BackendEvent::SendAmount(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("Amount sent successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    let chain_config_ref = Arc::as_ref(&node.chain_config);
                                    let serialized_info = match transaction_info.tx.to_json(chain_config_ref){
                                        Ok(json) => json,
                                        Err(e) => {
                                            e.to_string().into()
                                        }
                                    };
                                    let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                                    app_handle.emit("SendAmount", transaction_result).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error sending amount: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }
                    Some(BackendEvent::NewAddress(msg)) => {
                        match msg {
                            Ok(address_info) => {
                                println!("New address added successfully: {:?}", address_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("NewAddress", address_info).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error generating address wallet: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }
                    Some(BackendEvent::UpdateEncryption(msg)) => {
                        match msg {
                            Ok(encryption_info) => {
                                println!("Encryption updated successfully: {:?}", encryption_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("UpdateEncryption", encryption_info).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error updating encryption: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }
                    Some(BackendEvent::CloseWallet(msg)) => {
                        println!("Wallet closed successfully: {:?}", msg);
                        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                            app_handle.emit("CloseWallet", msg).unwrap();
                        }
                    }
                    Some(BackendEvent::StakeAmount(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("Encryption updated successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    let chain_config_ref = Arc::as_ref(&node.chain_config);
                                    let serialized_info = match transaction_info.tx.to_json(chain_config_ref){
                                        Ok(json) => json,
                                        Err(e) => {
                                            e.to_string().into()
                                        }
                                    };
                                    let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                                    app_handle.emit("StakeAmount", transaction_result).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error staking amount: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::DecommissionPool(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("Pool decommissioned successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    let chain_config_ref = Arc::as_ref(&node.chain_config);
                                    let serialized_info = match transaction_info.tx.to_json(chain_config_ref){
                                        Ok(json) => json,
                                        Err(e) => {
                                            e.to_string().into()
                                        }
                                    };
                                    let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                                    app_handle.emit("DecommissionPool", transaction_result).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error decommissioning pool: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::CreateDelegation(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("Delegation created successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    let chain_config_ref = Arc::as_ref(&node.chain_config);
                                    let serialized_info = match transaction_info.tx.to_json(chain_config_ref){
                                        Ok(json) => json,
                                        Err(e) => {
                                            e.to_string().into()
                                        }
                                    };
                                    let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                                    app_handle.emit("CreateDelegation", transaction_result).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error creating delegation: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::DelegateStaking(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("Staking delegated successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    let chain_config_ref = Arc::as_ref(&node.chain_config);
                                    let serialized_info = match transaction_info.0.tx.to_json(chain_config_ref){
                                        Ok(json) => json,
                                        Err(e) => {
                                            e.to_string().into()
                                        }
                                    };
                                    let transaction_result = DelegateStakingResult::new(transaction_info.0, serialized_info, transaction_info.1);
                                    app_handle.emit("DelegateStaking", transaction_result).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error delegating staking: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::SendDelegationToAddress(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("Sent delegation to address successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    let chain_config_ref = Arc::as_ref(&node.chain_config);
                                    let serialized_info = match transaction_info.tx.to_json(chain_config_ref){
                                        Ok(json) => json,
                                        Err(e) => {
                                            e.to_string().into()
                                        }
                                    };
                                    let transaction_result = TransactionResult::new(transaction_info, serialized_info);
                                    app_handle.emit("SendDelegationToAddress", transaction_result).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error sending delegation to address: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::NewAccount(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("New account created successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("NewAccount", transaction_info).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error creating new account: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::ToggleStaking(msg)) => {
                        match msg {
                            Ok(transaction_info) => {
                                println!("Staking toggled successfully: {:?}", transaction_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("ToggleStaking", transaction_info).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error toggling staking: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::ConsoleResponse(wallet_id, account_id, result)) => {
                        match result {
                            Ok(console_command) => {
                                println!(
                                    "Command executed successfully: wallet_id:, account_id:, console_command: {:?}, {:?}, {:?}",
                                    wallet_id,
                                    account_id,
                                    console_command
                                );
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("ConsoleResponse", console_command).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error toggling staking: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("ConsoleResponse", error_message).unwrap();
                                }
                            }
                        }
                    }

                    Some(BackendEvent::Broadcast(msg)) => {
                        match msg {
                            Ok(wallet_id) => {
                                println!("Transaction submitted successfully: {:?}", wallet_id);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Broadcast", wallet_id).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error toggling staking: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }

                    None => {
                        println!("No message received from backend");
                    }
                    _ => {
                        println!("Received an unhandled backend event");
                    }
                }
                    }
                }
        tokio::select! {
                    msg_opt = low_priority_backend_receiver.recv() =>{
                        println!("Backend event received {:?}", msg_opt.clone());
        match msg_opt {
                   
                    Some(BackendEvent::Balance(wallet_id, account_id, balance))=>{
                        println!("Balance updated {:?}", balance);
                        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                            let balance = BalanceResult::new(wallet_id, account_id, balance);
                            app_handle.emit("Balance", balance).unwrap();
                        }
                    }
                    Some(BackendEvent::StakingBalance(wallet_id, account_id, staking_balance))=>{
                        println!("staking Balance updated {:?}", staking_balance);
                        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                            let staking_balance = StakingBalanceResult::new(wallet_id, account_id, staking_balance);
                            app_handle.emit("StakingBalance", staking_balance).unwrap();
                        }
                    }
                    Some(BackendEvent::DelegationsBalance(wallet_id, account_id, delegations_balance))=>{
                        println!("Delegaion Balance updated {:?}", delegations_balance);
                        let delegations_balance = DelegationsBalanceResult::new(wallet_id, account_id, delegations_balance);
                        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                            app_handle.emit("DelegationBalance", delegations_balance).unwrap();
                        }
                    }
                    Some(BackendEvent::TransactionList(_, _, msg))=>{
                        match msg {
                            Ok(transaction_list) => {
                                println!("Transaction List received: {:?}", transaction_list);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("TransactionList", transaction_list).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error receiving transaction list: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }
                   
                    Some(BackendEvent::NewAddress(msg)) => {
                        match msg {
                            Ok(address_info) => {
                                println!("New address added successfully: {:?}", address_info);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("NewAddress", address_info).unwrap();
                                }
                            }
                            Err(e) => {
                                let error_message = e.to_string();
                                println!("Error generating address wallet: {}", error_message);
                                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                    app_handle.emit("Error", error_message).unwrap();
                                }
                            }
                        }
                    }
                    None => {
                        println!("No message received from backend");
                    }
                    _ => {
                        println!("Received an unhandled backend event");
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
        println!("Error parsing mnemonic: {}", error_message);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
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
                // get_initialized_node,
                initialize_node,
                listen_events,
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
                    println!("Window loaded");
                    GLOBAL_APP_HANDLE.set(_app_handle.clone()).expect(
                        "Failed to set global app handle"
                    );
                }
                _ => {}
            }
        });
}
