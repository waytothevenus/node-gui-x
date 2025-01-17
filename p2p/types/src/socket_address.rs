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

use std::{
    fmt::Display,
    net::{AddrParseError, IpAddr, SocketAddr},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{bannable_address::BannableAddress, peer_address::PeerAddress};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
    rpc_description::HasValueHint,
)]
pub struct SocketAddress(SocketAddr);

impl SocketAddress {
    pub fn new(addr: SocketAddr) -> Self {
        Self(addr)
    }

    pub fn socket_addr(&self) -> SocketAddr {
        self.0
    }

    pub fn ip_addr(&self) -> IpAddr {
        self.0.ip()
    }

    pub fn as_bannable(&self) -> BannableAddress {
        BannableAddress::new(self.0.ip())
    }

    pub fn as_peer_address(&self) -> PeerAddress {
        self.0.into()
    }
}

impl Display for SocketAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for SocketAddress {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SocketAddr::from_str(s).map(SocketAddress)
    }
}

impl From<SocketAddr> for SocketAddress {
    fn from(value: SocketAddr) -> Self {
        SocketAddress::new(value)
    }
}
