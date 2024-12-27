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
// limitations under the License.#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use common::chain::DelegationId;
use node_gui_backend::{
    messages::{TransactionInfo, WalletId},
    AccountId,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenCreateWalletRequest {
    pub mnemonic: String,
    pub file_path: String,
    pub import: bool,
    pub wallet_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenWalletRequest {
    pub file_path: String,
    pub wallet_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendAmountRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub amount: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StakeAmountRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub pledge_amount: String,
    pub mpt: String,
    pub cost_per_block: String,
    pub decommission_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecommissionStakingPoolRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub pool_id: String,
    pub output_address: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DelegationCreateRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub pool_id: String,
    pub delegation_address: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StakingDelegateRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub delegation_id: DelegationId,
    pub delegation_amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAddressRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEncryptionRequest {
    pub wallet_id: WalletId,
    pub action: String,
    pub password: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SendDelegateRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub address: String,
    pub amount: String,
    pub delegation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAccountRequest {
    pub wallet_id: WalletId,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToggleStakingRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleRequest {
    pub wallet_id: WalletId,
    pub account_id: AccountId,
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    pub tx: TransactionInfo,
    pub wallet_id: WalletId,
}
