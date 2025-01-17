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

use common::{
    chain::GenBlock,
    primitives::{BlockHeight, Id},
};
use rpc_description::HasValueHint;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, HasValueHint)]
pub struct CreatedBlockInfo {
    pub id: Id<GenBlock>,
    pub height: BlockHeight,
    pub pool_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, HasValueHint)]
pub struct BlockInfo {
    pub id: Id<GenBlock>,
    pub height: BlockHeight,
}

impl BlockInfo {
    pub fn from_tuple((id, height): (Id<GenBlock>, BlockHeight)) -> Self {
        Self { id, height }
    }
}
