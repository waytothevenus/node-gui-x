// Copyright (c) 2022 RBB S.r.l
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

use serde::{Deserialize, Serialize};

use common::{
    chain::{block::timestamp::BlockTimestamp, GenBlock},
    primitives::{BlockHeight, Id},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, rpc::description::HasValueHint)]
pub struct ChainInfo {
    pub best_block_height: BlockHeight,
    pub best_block_id: Id<GenBlock>,
    pub best_block_timestamp: BlockTimestamp,
    pub median_time: BlockTimestamp,
    pub is_initial_block_download: bool,
}
