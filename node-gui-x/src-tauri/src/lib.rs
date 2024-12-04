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

mod backend_impl;
mod chainstate_event_handler;
mod error;
pub mod messages;
mod p2p_event_handler;
mod wallet_events;

use self::error::BackendError;
use self::messages::BackendEvent;
use crate::chainstate_event_handler::ChainstateEventHandler;
use crate::p2p_event_handler::P2pEventHandler;
use chainstate::ChainInfo;
use common::address::{ Address, AddressError };
use common::time_getter::TimeGetter;
use common::chain::SignedTransaction;
use common::chain::{ ChainConfig, DelegationId, Destination };
use common::primitives::{ Amount, BlockHeight };
use crypto::key::hdkd::u31::U31;
use messages::{
    AccountId,
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
use node_lib::{ Command, RunOptions };
use once_cell::sync::OnceCell;
use serde::{ Deserialize, Serialize };
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::RwLock;
use tauri::{ AppHandle, Emitter };
use tokio::sync::mpsc::{ unbounded_channel, UnboundedReceiver, UnboundedSender };
use wallet_types::wallet_type::WalletType;
struct AppState {
    initialized_node: RwLock<Option<InitializedNode>>,
    backend_sender: RwLock<Option<BackendSender>>,
    backend_receiver: RwLock<Option<UnboundedReceiver<BackendEvent>>>,
    low_priority_backend_receiver: RwLock<Option<UnboundedReceiver<BackendEvent>>>,
}

static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

pub struct BackendControls {
    pub initialized_node: InitializedNode,
    pub backend_sender: BackendSender,
    pub backend_receiver: UnboundedReceiver<BackendEvent>,
    pub low_priority_backend_receiver: UnboundedReceiver<BackendEvent>,
}

#[derive(Debug)]
pub struct BackendSender {
    request_tx: UnboundedSender<BackendRequest>,
}

impl BackendSender {
    fn new(request_tx: UnboundedSender<BackendRequest>) -> Self {
        Self { request_tx }
    }

    pub fn send(&self, msg: BackendRequest) {
        let _ = self.request_tx.send(msg);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImportOrCreate {
    Import,
    Create,
}

impl ImportOrCreate {
    pub fn skip_syncing(&self) -> bool {
        match self {
            Self::Create => true,
            Self::Import => false,
        }
    }

    pub fn from_bool(value: bool) -> Self {
        if value { ImportOrCreate::Create } else { ImportOrCreate::Import }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InitNetwork {
    Mainnet,
    Testnet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WalletMode {
    Cold,
    Hot,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenCreateWalletRequest {
    mnemonic: String,
    file_path: String,
    import: bool,
    wallet_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenWalletRequest {
    file_path: String,
    wallet_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendAmountRequest {
    wallet_id: u64,
    account_id: U31,
    amount: String,
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StakeAmountRequest {
    wallet_id: u64,
    account_id: U31,
    pledge_amount: String,
    mpt: String,
    cost_per_block: String,
    decommission_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecommissionStakingPoolRequest {
    pub wallet_id: u64,
    pub account_id: U31,
    pub pool_id: String,
    pub output_address: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DelegationCreateRequest {
    pub wallet_id: u64,
    pub account_id: U31,
    pub pool_id: String,
    pub delegation_address: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StakingDelegateRequest {
    pub wallet_id: u64,
    pub account_id: U31,
    pub delegation_id: DelegationId,
    pub delegation_amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAddressRequest {
    wallet_id: u64,
    account_id: U31,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEncryptionRequest {
    wallet_id: u64,
    action: String,
    password: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SendDelegateRequest {
    pub wallet_id: u64,
    pub account_id: U31,
    pub address: String,
    pub amount: String,
    pub delegation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAccountRequest {
    wallet_id: u64,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToggleStakingRequest {
    wallet_id: u64,
    account_id: U31,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleRequest {
    wallet_id: u64,
    account_id: U31,
    command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    tx: SignedTransaction,
    wallet_id: u64,
    account_id: U31,
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
#[derive(Debug, Clone)]
pub struct InitializedNode {
    pub chain_config: Arc<ChainConfig>,
    pub chain_info: ChainInfo,
}

fn parse_coin_amount(chain_config: &ChainConfig, value: &str) -> Option<Amount> {
    Amount::from_fixedpoint_str(value, chain_config.coin_decimals())
}

fn parse_address(
    chain_config: &ChainConfig,
    address: &str
) -> Result<Address<Destination>, AddressError> {
    Address::from_string(chain_config, address)
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
    let backend_controls = node_initialize(
        TimeGetter::default(),
        net_type,
        wallet_type
    ).await.map_err(|e| e.to_string())?;

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
pub async fn node_initialize(
    _time_getter: TimeGetter,
    network: InitNetwork,
    mode: WalletMode
) -> anyhow::Result<BackendControls> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,wgpu_core=error,hyper=error,jsonrpsee-server=error");
    }

    let mut opts = node_lib::Options::from_args(std::env::args_os());
    opts.command = match network {
        InitNetwork::Mainnet => Some(Command::Mainnet(RunOptions::default())),
        InitNetwork::Testnet => Some(Command::Testnet(RunOptions::default())),
    };

    logging::init_logging();
    logging::log::info!("Command line options: {opts:?}");

    let (request_tx, request_rx) = unbounded_channel();
    let (event_tx, event_rx) = unbounded_channel();
    let (low_priority_event_tx, low_priority_event_rx) = unbounded_channel();
    let (wallet_updated_tx, wallet_updated_rx) = unbounded_channel();

    let (chain_config, chain_info) = match mode {
        WalletMode::Hot => {
            let setup_result = node_lib::setup(opts, true).await?;
            let node = match setup_result {
                node_lib::NodeSetupResult::Node(node) => node,
                node_lib::NodeSetupResult::DataDirCleanedUp => {
                    // TODO: find more friendly way to report the message and shut down GUI
                    anyhow::bail!(
                        "Data directory is now clean. Please restart the node without `--clean-data` flag"
                    );
                }
            };

            let controller = node.controller().clone();

            let manager_join_handle = tokio::spawn(async move { node.main().await });
            // Subscribe to chainstate before getting the current chain_info!
            let chainstate_event_handler = ChainstateEventHandler::new(
                controller.chainstate.clone(),
                event_tx.clone()
            ).await;

            let p2p_event_handler = P2pEventHandler::new(&controller.p2p, event_tx.clone()).await;

            let chain_config = controller.chainstate.call(|this|
                Arc::clone(this.get_chain_config())
            ).await?;
            let chain_info = controller.chainstate.call(|this| this.info()).await??;

            let backend = backend_impl::Backend::new_hot(
                chain_config.clone(),
                event_tx.clone(),
                low_priority_event_tx.clone(),
                wallet_updated_tx.clone(),
                controller.clone(),
                manager_join_handle
            );

            tokio::spawn(async move {
                backend_impl::run(
                    backend,
                    request_rx,
                    wallet_updated_rx,
                    chainstate_event_handler,
                    p2p_event_handler
                ).await;
            });

            (chain_config, chain_info)
        }
        WalletMode::Cold => {
            let chain_config = Arc::new(match network {
                InitNetwork::Mainnet => common::chain::config::create_mainnet(),
                InitNetwork::Testnet => common::chain::config::create_testnet(),
            });
            let chain_info = ChainInfo {
                best_block_id: chain_config.genesis_block_id(),
                best_block_height: BlockHeight::zero(),
                median_time: chain_config.genesis_block().timestamp(),
                best_block_timestamp: chain_config.genesis_block().timestamp(),
                is_initial_block_download: false,
            };

            let manager_join_handle = tokio::spawn(async move {});
            let backend = backend_impl::Backend::new_cold(
                chain_config.clone(),
                event_tx.clone(),
                low_priority_event_tx.clone(),
                wallet_updated_tx.clone(),
                manager_join_handle
            );

            tokio::spawn(async move {
                backend_impl::run_cold(backend, request_rx, wallet_updated_rx).await;
            });


            (chain_config, chain_info)
        }
    };

    let initialized_node = InitializedNode {
        chain_config: Arc::clone(&chain_config),
        chain_info,
    };

    let backend_controls = BackendControls {
        initialized_node,
        backend_sender: BackendSender::new(request_tx),
        backend_receiver: event_rx,
        low_priority_backend_receiver: low_priority_event_rx,
    };

    Ok(backend_controls)
}

#[tauri::command]
async fn listen_events(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let state_clone = state.clone(); // Clone the state to move into the async block
    // tokio::spawn(async move {
    loop {
        // Acquire a read lock only when receiving messages
        let mut backend_receiver_guard = state_clone.backend_receiver.write().await;
        let backend_receiver = backend_receiver_guard
            .as_mut()
            .expect("Backend receiver not initialized");

        // Await the reception of a message outside of the lock
        let msg_opt = backend_receiver.recv().await;

        // Lock is released here
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
                            app_handle.emit("SendAmount", transaction_info).unwrap();
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
                            app_handle.emit("StakeAmount", transaction_info).unwrap();
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
                            app_handle.emit("DecommissionPool", transaction_info).unwrap();
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
                            app_handle.emit("CreateDelegation", transaction_info).unwrap();
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
                            app_handle.emit("DelegateStaking", transaction_info).unwrap();
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
                            app_handle.emit("SendDelegationToAddress", transaction_info).unwrap();
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
                            "Staking toggled successfully: wallet_id:, account_id:, console_command: {:?}, {:?}, {:?}",
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
                            app_handle.emit("Error", error_message).unwrap();
                        }
                    }
                }
            }

            Some(BackendEvent::Broadcast(msg)) => {
                match msg  {
                    Ok(wallet_id) => {
                        println!(
                            "Transaction submitted successfully: {:?}",
                            wallet_id,
                        );
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
    // });
    // Ok(())
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

    let wallet_type = WalletType::from_str(&request.wallet_type).map_err(|e| {
        let error_message = e.to_string();
        println!("Error parsing wallet type: {}", error_message);
        error_message
    })?;

    let import = ImportOrCreate::from_bool(request.import);

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

    let wallet_type = WalletType::from_str(&request.wallet_type).map_err(|e| {
        let error_message = e.to_string();
        println!("Error parsing wallet type: {}", error_message);
        error_message
    })?;
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::OpenWallet { file_path, wallet_type });
    Ok(())
}

#[tauri::command]
async fn send_amount_wrapper(
    state: tauri::State<'_, AppState>,
    request: SendAmountRequest
) -> Result<(), String> {
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;

    let request = SendRequest {
        wallet_id: wallet_id,
        account_id: account_id,
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
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id: AccountId = AccountId::new(request.account_id);

    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;

    backend_sender.send(BackendRequest::NewAddress(wallet_id, account_id));
    Ok(())
}

#[tauri::command]
async fn update_encryption_wrapper(
    state: tauri::State<'_, AppState>,
    request: UpdateEncryptionRequest
) -> Result<(), String> {
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let update_encryption_action = match
        EncryptionAction::from_str(&request.action, request.password.as_deref())
    {
        Some(action) => action,
        None => {
            return Err("Invalid action or missing password".into());
        }
    };
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::UpdateEncryption {
        wallet_id: wallet_id,
        action: update_encryption_action,
    });
    Ok(())
}

#[tauri::command]
async fn close_wallet_wrapper(
    state: tauri::State<'_, AppState>,
    wallet_id: u64
) -> Result<(), String> {
    let wallet_id = WalletId::from_u64(wallet_id);
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
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let stake_request = StakeRequest {
        wallet_id: wallet_id,
        account_id: account_id,
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
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let decommission_request = DecommissionPoolRequest {
        wallet_id: wallet_id,
        account_id: account_id,
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
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let delegation_request = CreateDelegationRequest {
        wallet_id: wallet_id,
        account_id: account_id,
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
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let delegation_request = DelegateStakingRequest {
        wallet_id: wallet_id,
        account_id: account_id,
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
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let send_delegation_request = SendDelegateToAddressRequest {
        wallet_id: wallet_id,
        account_id: account_id,
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
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::NewAccount { wallet_id: wallet_id, name: request.name });
    Ok(())
}

#[tauri::command]
async fn toggle_stakig_wrapper(
    state: tauri::State<'_, AppState>,
    request: ToggleStakingRequest
) -> Result<(), String> {
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::ToggleStaking(wallet_id, account_id, request.enabled));
    Ok(())
}

#[tauri::command]
async fn handle_console_command_wrapper(
    state: tauri::State<'_, AppState>,
    request: ConsoleRequest
) -> Result<(), String> {
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let account_id = AccountId::new(request.account_id);
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::ConsoleCommand {
        wallet_id: wallet_id,
        account_id: account_id,
        command: request.command,
    });
    Ok(())
}

#[tauri::command]
async fn submit_transaction_wrapper(
    state: tauri::State<'_, AppState>,
    request: SubmitTransactionRequest
) -> Result<(), String> {
    let wallet_id = WalletId::from_u64(request.wallet_id);
    let mut backend_sender_guard = state.backend_sender.write().await;
    let backend_sender = backend_sender_guard.as_mut().ok_or("Backend Sender not initialized")?;
    backend_sender.send(BackendRequest::SubmitTx { wallet_id: wallet_id, tx: request.tx });
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
