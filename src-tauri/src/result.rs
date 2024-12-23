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
// limitations under the License.#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;

use common::{
    chain::{DelegationId, GenBlock, PoolId},
    primitives::{Amount, BlockHeight, Id},
};
use node_gui_backend::{
    messages::{TransactionInfo, WalletId},
    AccountId,
};
use serde::Serialize;
use serde_json::Value;
use wallet::account::transaction_list::TransactionList;
use wallet_rpc_lib::types::{Balances, PoolInfo};

#[derive(Debug, Clone, Serialize)]
pub struct TransactionResult {
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
pub struct DelegateStakingResult {
    transaction_info: TransactionInfo,
    serialized_tx: Value,
    delegation_id: DelegationId,
}

impl DelegateStakingResult {
    pub fn new(
        transaction_info: TransactionInfo,
        serialized_tx: Value,
        delegation_id: DelegationId,
    ) -> Self {
        DelegateStakingResult {
            transaction_info,
            serialized_tx,
            delegation_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StakingBalanceResult {
    wallet_id: WalletId,
    account_id: AccountId,
    staking_balance: BTreeMap<PoolId, PoolInfo>,
}

impl StakingBalanceResult {
    pub fn new(
        wallet_id: WalletId,
        account_id: AccountId,
        staking_balance: BTreeMap<PoolId, PoolInfo>,
    ) -> Self {
        StakingBalanceResult {
            wallet_id,
            account_id,
            staking_balance,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BalanceResult {
    wallet_id: WalletId,
    account_id: AccountId,
    balance: Balances,
}

impl BalanceResult {
    pub fn new(wallet_id: WalletId, account_id: AccountId, balance: Balances) -> Self {
        BalanceResult {
            wallet_id,
            account_id,
            balance,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DelegationsBalanceResult {
    wallet_id: WalletId,
    account_id: AccountId,
    delegations_balance: BTreeMap<String, (String, Amount)>,
}

impl DelegationsBalanceResult {
    pub fn new(
        wallet_id: WalletId,
        account_id: AccountId,
        delegations_balance: BTreeMap<String, (String, Amount)>,
    ) -> Self {
        DelegationsBalanceResult {
            wallet_id,
            account_id,
            delegations_balance,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionListResult {
    wallet_id: WalletId,
    account_id: AccountId,
    transaction_list: TransactionList,
}

impl TransactionListResult {
    pub fn new(
        wallet_id: WalletId,
        account_id: AccountId,
        transaction_list: TransactionList,
    ) -> Self {
        TransactionListResult {
            wallet_id,
            account_id,
            transaction_list,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletBestBlockResult {
    wallet_id: WalletId,
    block_info: (Id<GenBlock>, BlockHeight),
}

impl WalletBestBlockResult {
    pub fn new(wallet_id: WalletId, block_info: (Id<GenBlock>, BlockHeight)) -> Self {
        WalletBestBlockResult {
            wallet_id,
            block_info,
        }
    }
}
