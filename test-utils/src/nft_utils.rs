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

use crate::random_ascii_alphanumeric_string;
use common::{
    chain::{
        config::ChainConfig,
        tokens::{
            IsTokenFreezable, Metadata, NftIssuanceV0, TokenCreator, TokenIssuanceV0,
            TokenIssuanceV1, TokenTotalSupply,
        },
        Destination,
    },
    primitives::Amount,
};
use crypto::key::{KeyKind, PrivateKey};
use randomness::{CryptoRng, Rng};
use serialization::extras::non_empty_vec::DataOrNoVec;

pub fn random_creator(rng: &mut (impl Rng + CryptoRng)) -> TokenCreator {
    let (_, public_key) = PrivateKey::new_from_rng(rng, KeyKind::Secp256k1Schnorr);
    TokenCreator::from(public_key)
}

pub fn random_token_issuance(chain_config: &ChainConfig, rng: &mut impl Rng) -> TokenIssuanceV0 {
    let max_ticker_len = chain_config.token_max_ticker_len();
    let max_dec_count = chain_config.token_max_dec_count();
    let max_uri_len = chain_config.token_max_uri_len();

    TokenIssuanceV0 {
        token_ticker: random_ascii_alphanumeric_string(rng, 1..max_ticker_len).as_bytes().to_vec(),
        amount_to_issue: Amount::from_atoms(rng.gen_range(1..u128::MAX)),
        number_of_decimals: rng.gen_range(1..max_dec_count),
        metadata_uri: random_ascii_alphanumeric_string(rng, 1..max_uri_len).as_bytes().to_vec(),
    }
}

pub fn random_token_issuance_v1(
    chain_config: &ChainConfig,
    authority: Destination,
    rng: &mut impl Rng,
) -> TokenIssuanceV1 {
    let max_ticker_len = chain_config.token_max_ticker_len();
    let max_dec_count = chain_config.token_max_dec_count();
    let max_uri_len = chain_config.token_max_uri_len();

    let _fix_code_below_if_enum_changes = |supply: TokenTotalSupply| match supply {
        TokenTotalSupply::Fixed(_) => unreachable!(),
        TokenTotalSupply::Lockable => unreachable!(),
        TokenTotalSupply::Unlimited => unreachable!(),
    };

    let supply = match rng.gen_range(0..3) {
        0 => {
            let supply = Amount::from_atoms(rng.gen_range(1..1_000_000));
            TokenTotalSupply::Fixed(supply)
        }
        1 => TokenTotalSupply::Lockable,
        2 => TokenTotalSupply::Unlimited,
        _ => unreachable!(),
    };

    let is_freezable = if rng.gen::<bool>() {
        IsTokenFreezable::Yes
    } else {
        IsTokenFreezable::No
    };

    TokenIssuanceV1 {
        token_ticker: random_ascii_alphanumeric_string(rng, 1..max_ticker_len).as_bytes().to_vec(),
        number_of_decimals: rng.gen_range(1..max_dec_count),
        metadata_uri: random_ascii_alphanumeric_string(rng, 1..max_uri_len).as_bytes().to_vec(),
        total_supply: supply,
        is_freezable,
        authority,
    }
}

pub fn random_nft_issuance(
    chain_config: &ChainConfig,
    rng: &mut (impl Rng + CryptoRng),
) -> NftIssuanceV0 {
    let max_desc_len = chain_config.token_max_description_len();
    let max_name_len = chain_config.token_max_name_len();
    let max_ticker_len = chain_config.token_max_ticker_len();

    NftIssuanceV0 {
        metadata: Metadata {
            creator: Some(random_creator(rng)),
            name: random_ascii_alphanumeric_string(rng, 1..max_name_len).into_bytes(),
            description: random_ascii_alphanumeric_string(rng, 1..max_desc_len).into_bytes(),
            ticker: random_ascii_alphanumeric_string(rng, 1..max_ticker_len).into_bytes(),
            icon_uri: DataOrNoVec::from(None),
            additional_metadata_uri: DataOrNoVec::from(None),
            media_uri: DataOrNoVec::from(None),
            media_hash: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
        },
    }
}
