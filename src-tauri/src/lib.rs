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

use std::sync::Mutex;

use node_gui_backend::BackendSender;
use tauri::Manager;

mod commands;
mod request;
mod result;

struct AppState {
    backend_sender: Option<BackendSender>,
    app_handle: tauri::AppHandle,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let r = app.manage(Mutex::new(AppState {
                backend_sender: None,
                app_handle: app.handle().clone(),
            }));
            assert!(r);
            Ok(())
        })
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::initialize_node,
            commands::add_create_wallet_wrapper,
            commands::add_open_wallet_wrapper,
            commands::send_amount_wrapper,
            commands::new_address_wrapper,
            commands::update_encryption_wrapper,
            commands::close_wallet_wrapper,
            commands::stake_amount_wrapper,
            commands::decommission_pool_wrapper,
            commands::create_delegation_wrapper,
            commands::delegate_staking_wrapper,
            commands::send_delegation_to_address_wrapper,
            commands::new_account_wrapper,
            commands::toggle_staking_wrapper,
            commands::handle_console_command_wrapper,
            commands::submit_transaction_wrapper,
            commands::shutdown_wrapper
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // TODO: backend event task should be joined here to make sure it's properly finished
            }
        });
}