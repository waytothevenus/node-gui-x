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

macro_rules! tests {
    ($($(#[$meta:meta])* $name:ident,)+) => {
        pub fn tests<T, N>() -> impl Iterator<Item = libtest_mimic::Trial>
        where
            T: networking::test_helpers::TestTransportMaker<Transport = N::Transport>,
            N: p2p::net::NetworkingService + std::fmt::Debug + 'static,
            N::ConnectivityHandle: p2p::net::ConnectivityService<N> + std::fmt::Debug,
            N::MessagingHandle: MessagingService + std::fmt::Debug,
            N::SyncingEventReceiver: SyncingEventReceiver + std::fmt::Debug,
        {
            [
                $($(#[$meta])*
                libtest_mimic::Trial::test(
                concat!(module_path!(), "::", stringify!($name)),
                || {
                    tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(async {
                            $name::<T, N>().await;
                        });
                    Ok(())
                }
            ),)*].into_iter()
        }
    }
}
