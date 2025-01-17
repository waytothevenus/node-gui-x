// Copyright (c) 2021-2024 RBB S.r.l
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

/// A wrapper trait for a transcript that can be signed
pub trait SignableTranscript: schnorrkel::context::SigningTranscript {
    /// The extra transcript is built during signature as per the specs in Schnorrkel.
    /// This function is necessary to customize the rng object that will be used.
    fn make_extra_transcript(&self) -> Self;
    fn attach_u64(self, label: &'static [u8], value: u64) -> Self;
    fn attach_raw_data<T: AsRef<[u8]>>(self, label: &'static [u8], value: T) -> Self;
}
