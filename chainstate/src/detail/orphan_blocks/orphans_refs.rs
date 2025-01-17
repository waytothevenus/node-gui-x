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

use super::OrphanAddError;
use common::{
    chain::{Block, GenBlock},
    primitives::{id::WithId, Id},
};

pub trait OrphanBlocksRef {
    fn len(&self) -> usize;
    fn is_already_an_orphan(&self, block_id: &Id<Block>) -> bool;
}

pub trait OrphanBlocksMut: OrphanBlocksRef {
    #[allow(dead_code)]
    fn clear(&mut self);
    fn add_block(&mut self, block: WithId<Block>) -> Result<(), Box<OrphanAddError>>;
    fn take_all_children_of(&mut self, block_id: &Id<GenBlock>) -> Vec<WithId<Block>>;
}
