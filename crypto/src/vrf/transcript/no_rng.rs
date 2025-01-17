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

use super::{traits::SignableTranscript, with_rng::VRFTranscriptWithRng};

#[must_use]
#[derive(Clone)]
pub struct VRFTranscript(merlin::Transcript);

impl VRFTranscript {
    pub fn new(label: &'static [u8]) -> Self {
        Self(merlin::Transcript::new(label))
    }

    pub(crate) fn take(self) -> merlin::Transcript {
        self.0
    }

    pub fn with_rng<'a, R: randomness::RngCore + randomness::CryptoRng + 'a>(
        self,
        rng: R,
    ) -> VRFTranscriptWithRng<'a> {
        VRFTranscriptWithRng::from_no_rng(self, rng)
    }
}

impl SignableTranscript for VRFTranscript {
    fn make_extra_transcript(&self) -> Self {
        VRFTranscript::new(b"VRF")
    }

    fn attach_u64(mut self, label: &'static [u8], value: u64) -> Self {
        self.0.append_u64(label, value);
        self
    }

    fn attach_raw_data<T: AsRef<[u8]>>(mut self, label: &'static [u8], message: T) -> Self {
        self.0.append_message(label, message.as_ref());
        self
    }
}

impl schnorrkel::context::SigningTranscript for VRFTranscript {
    fn commit_bytes(&mut self, label: &'static [u8], bytes: &[u8]) {
        self.0.append_message(label, bytes)
    }

    fn challenge_bytes(&mut self, label: &'static [u8], dest: &mut [u8]) {
        self.0.challenge_bytes(label, dest)
    }

    fn witness_bytes_rng<R>(
        &self,
        label: &'static [u8],
        dest: &mut [u8],
        nonce_seeds: &[&[u8]],
        rng: R,
    ) where
        R: randomness::RngCore + randomness::CryptoRng,
    {
        self.0.witness_bytes_rng(label, dest, nonce_seeds, rng)
    }

    fn witness_bytes(&self, label: &'static [u8], dest: &mut [u8], nonce_seeds: &[&[u8]]) {
        self.witness_bytes_rng(label, dest, nonce_seeds, randomness::make_true_rng())
    }
}

#[cfg(test)]
mod tests {

    use rand_chacha::ChaChaRng;

    use randomness::{Rng, SeedableRng};

    use super::*;

    #[test]
    fn manual_vs_assembled() {
        // build first transcript by manually filling values
        let mut manual_transcript = merlin::Transcript::new(b"initial");
        manual_transcript.append_message(b"abc", b"xyz");
        manual_transcript.append_u64(b"rx42", 424242);

        // build the second transcript using the assembler
        let assembled_transcript = VRFTranscript::new(b"initial")
            .attach_raw_data(b"abc", b"xyz")
            .attach_u64(b"rx42", 424242);

        // build a random number generator using each transcript and ensure they both arrive to the same values
        let mut g1 = manual_transcript.build_rng().finalize(&mut ChaChaRng::from_seed([0u8; 32]));
        let mut g2 = assembled_transcript
            .0
            .build_rng()
            .finalize(&mut ChaChaRng::from_seed([0u8; 32]));

        for _ in 0..100 {
            assert_eq!(g1.gen::<u64>(), g2.gen::<u64>());
        }
    }
}
