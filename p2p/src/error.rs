// Copyright (c) 2021-2022 RBB S.r.l
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

use thiserror::Error;

use chainstate::{ban_score::BanScore, ChainstateError};
use common::{
    chain::{config::MagicBytes, Block, Transaction},
    primitives::{time::Time, Id},
};
use mempool::error::{Error as MempoolError, MempoolBanScore};
use networking::error::NetworkingError;
use p2p_types::{services::Services, socket_address::SocketAddress, PeerId};
use utils::try_as::TryAsRef;

use crate::{net::types::PeerRole, peer_manager::peerdb_common, protocol::ProtocolVersion};

/// Errors related to invalid data/peer information that results in connection getting closed
/// and the peer getting banned.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("Peer is unresponsive")]
    Unresponsive,
    #[error("Locator size ({0}) exceeds allowed limit ({1})")]
    LocatorSizeExceeded(usize, usize),
    #[error("Requested {0} blocks with limit of {1}")]
    BlocksRequestLimitExceeded(usize, usize),
    #[error("Number of headers in message ({0}) exceeds allowed limit ({1})")]
    HeadersLimitExceeded(usize, usize),
    #[error("A peer requested an unknown block ({0})")]
    UnknownBlockRequested(Id<Block>),
    #[error("A peer tried to download same block ({0})")]
    DuplicatedBlockRequest(Id<Block>),
    #[error("Headers aren't connected")]
    DisconnectedHeaders,
    #[error("Peer sent a message ({0}) that wasn't expected")]
    UnexpectedMessage(String),
    #[error("Peer sent a block ({0}) that wasn't requested")]
    UnsolicitedBlockReceived(Id<Block>),
    #[error("Peer sent block {expected_block_id} while it was expected to send {actual_block_id}")]
    BlocksReceivedInWrongOrder {
        expected_block_id: Id<Block>,
        actual_block_id: Id<Block>,
    },
    #[error("Empty block list requested")]
    ZeroBlocksInRequest,
    #[error("Handshake expected")]
    HandshakeExpected,
    #[error("More than MAX_ADDRESS_COUNT addresses sent")]
    AddressListLimitExceeded,
    #[error("A peer tried to announce the same transaction ({0})")]
    DuplicatedTransactionAnnouncement(Id<Transaction>),
    #[error("Announced too many transactions (limit is {0})")]
    TransactionAnnouncementLimitExceeded(usize),
}

/// Peer state errors (Errors either for an individual peer or for the [`PeerManager`](crate::peer_manager::PeerManager))
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PeerError {
    #[error("Peer doesn't exist")]
    PeerDoesntExist,
    #[error("Peer {0} already exists")]
    PeerAlreadyExists(PeerId),
    #[error(
        "Rejecting {new_peer_role:?} connection to {new_peer_addr:?} \
             because we already have {existing_peer_role:?} connection to {existing_peer_addr:?}"
    )]
    AlreadyConnected {
        existing_peer_addr: SocketAddress,
        existing_peer_role: PeerRole,
        new_peer_addr: SocketAddress,
        new_peer_role: PeerRole,
    },
    #[error("Connection to address {0} already pending")]
    Pending(String),
    /// This error is used by backend to drop the connection after the peer has informed us
    /// about an impending disconnection.
    #[error("The peer is going to disconnect us")]
    PeerWillDisconnect,
}

// TODO: this error type doesn't make much sense since most of its contents have been moved
// to `networking`. Need to find a better place for the remaining enum variants.
/// Errors related to establishing a connection with a remote peer
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DialError {
    #[error("Tried to dial self")]
    AttemptToDialSelf,
    #[error("Connection refused or timed out")]
    ConnectionRefusedOrTimedOut,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConnectionValidationError {
    #[error("Peer has an unsupported network protocol: {peer_protocol_version:?}")]
    UnsupportedProtocol {
        peer_protocol_version: ProtocolVersion,
    },
    #[error("Peer time {remote_time:?} out of the acceptable range {accepted_peer_time:?}")]
    TimeDiff {
        remote_time: Time,
        accepted_peer_time: std::ops::RangeInclusive<Time>,
    },
    #[error(
        "Peer is in different network. Our network is '{our_network}', their network is '{their_network}'"
    )]
    DifferentNetwork {
        our_network: MagicBytes,
        their_network: MagicBytes,
    },
    #[error("Too many peers")]
    TooManyInboundPeersAndThisOneIsDiscouraged,
    #[error("Too many peers")]
    TooManyInboundPeersAndCannotEvictAnyone,
    #[error("Address {address} is banned")]
    AddressBanned { address: String },
    #[error("Address {address} is discouraged")]
    AddressDiscouraged { address: String },
    #[error("No common services")]
    NoCommonServices,
    #[error(
        "Insufficient services, needed: {needed_services:?}, available: {available_services:?}"
    )]
    InsufficientServices {
        needed_services: Services,
        available_services: Services,
    },
    #[error("Networking disabled")]
    NetworkingDisabled,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    #[error("Block data missing when trying to send block {0}")]
    BlockDataMissingInSendBlock(Id<Block>),
    #[error("Block index missing when trying to send block {0}")]
    BlockIndexMissingInSendBlock(Id<Block>),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum P2pError {
    #[error("Networking error: {0}")]
    NetworkingError(#[from] NetworkingError),
    #[error("Protocol violation: {0}")]
    ProtocolError(ProtocolError),
    #[error("Failed to dial peer: {0}")]
    DialError(DialError),
    #[error("Connection to other task lost")]
    ChannelClosed,
    #[error("Peer-related error: {0}")]
    PeerError(PeerError),
    #[error("SubsystemFailure")]
    SubsystemFailure,
    #[error("ConsensusError: {0}")]
    ChainstateError(ChainstateError),
    #[error("DatabaseFailure")]
    StorageFailure(#[from] storage::Error),
    #[error("Noise protocol handshake error")]
    NoiseHandshakeError(String),
    #[error("The configuration value is invalid: {0}")]
    InvalidConfigurationValue(String),
    #[error("The storage state is invalid: {0}")]
    InvalidStorageState(String),
    #[error("Peer db storage version mismatch: expected {expected_version}, got {actual_version}")]
    PeerDbStorageVersionMismatch {
        expected_version: peerdb_common::StorageVersion,
        actual_version: peerdb_common::StorageVersion,
    },
    #[error("Mempool error: {0}")]
    MempoolError(#[from] MempoolError),
    #[error("Connection validation failed: {0}")]
    ConnectionValidationFailed(#[from] ConnectionValidationError),
    #[error("Synchronization error: {0}")]
    SyncError(#[from] SyncError),
}

impl From<DialError> for P2pError {
    fn from(e: DialError) -> P2pError {
        P2pError::DialError(e)
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for P2pError {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> P2pError {
        P2pError::ChannelClosed
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for P2pError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> P2pError {
        P2pError::ChannelClosed
    }
}

impl From<subsystem::error::CallError> for P2pError {
    fn from(_e: subsystem::error::CallError) -> P2pError {
        P2pError::ChannelClosed
    }
}

impl From<ChainstateError> for P2pError {
    fn from(e: ChainstateError) -> P2pError {
        P2pError::ChainstateError(e)
    }
}

impl BanScore for P2pError {
    fn ban_score(&self) -> u32 {
        match self {
            P2pError::NetworkingError(_) => 0,
            P2pError::ProtocolError(err) => err.ban_score(),
            P2pError::DialError(_) => 0,
            P2pError::ChannelClosed => 0,
            P2pError::PeerError(_) => 0,
            P2pError::SubsystemFailure => 0,
            P2pError::ChainstateError(err) => err.ban_score(),
            P2pError::StorageFailure(_) => 0,
            // Could be a noise protocol violation but also a network error, do not ban peer
            P2pError::NoiseHandshakeError(_) => 0,
            P2pError::InvalidConfigurationValue(_) => 0,
            P2pError::InvalidStorageState(_) => 0,
            P2pError::PeerDbStorageVersionMismatch {
                expected_version: _,
                actual_version: _,
            } => 0,
            P2pError::MempoolError(err) => err.mempool_ban_score(),
            P2pError::ConnectionValidationFailed(_) => 0,
            P2pError::SyncError(err) => err.ban_score(),
        }
    }
}

impl BanScore for ProtocolError {
    fn ban_score(&self) -> u32 {
        match self {
            ProtocolError::Unresponsive => 100,
            ProtocolError::LocatorSizeExceeded(_, _) => 20,
            ProtocolError::BlocksRequestLimitExceeded(_, _) => 20,
            ProtocolError::HeadersLimitExceeded(_, _) => 20,
            ProtocolError::UnknownBlockRequested(_) => 20,
            ProtocolError::DuplicatedBlockRequest(_) => 20,
            ProtocolError::DisconnectedHeaders => 20,
            ProtocolError::UnexpectedMessage(_) => 20,
            ProtocolError::UnsolicitedBlockReceived(_) => 20,
            ProtocolError::BlocksReceivedInWrongOrder {
                expected_block_id: _,
                actual_block_id: _,
            } => 20,
            ProtocolError::ZeroBlocksInRequest => 20,
            ProtocolError::HandshakeExpected => 100,
            ProtocolError::AddressListLimitExceeded => 100,
            ProtocolError::DuplicatedTransactionAnnouncement(_) => 20,
            ProtocolError::TransactionAnnouncementLimitExceeded(_) => 20,
        }
    }
}

impl BanScore for SyncError {
    fn ban_score(&self) -> u32 {
        match self {
            SyncError::BlockDataMissingInSendBlock(_) => 0,
            SyncError::BlockIndexMissingInSendBlock(_) => 0,
        }
    }
}

impl TryAsRef<storage::Error> for P2pError {
    fn try_as_ref(&self) -> Option<&storage::Error> {
        match self {
            P2pError::NetworkingError(_)
            | P2pError::ProtocolError(_)
            | P2pError::DialError(_)
            | P2pError::ChannelClosed
            | P2pError::PeerError(_)
            | P2pError::SubsystemFailure
            | P2pError::ChainstateError(_)
            | P2pError::NoiseHandshakeError(_)
            | P2pError::InvalidConfigurationValue(_)
            | P2pError::InvalidStorageState(_)
            | P2pError::PeerDbStorageVersionMismatch {
                expected_version: _,
                actual_version: _,
            }
            | P2pError::MempoolError(_)
            | P2pError::ConnectionValidationFailed(_)
            | P2pError::SyncError(_) => None,
            P2pError::StorageFailure(err) => Some(err),
        }
    }
}
