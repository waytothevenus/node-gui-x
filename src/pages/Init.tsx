// Copyright (c) 2024 RBB S.r.l
// opensource@mintlayer.org
// SPDX-License-Identifier: MIT
// Licensed under the MIT License;
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/mintlayer/node-gui-x/blob/master/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Menu } from "@tauri-apps/api/menu";
import MintlayerIcon from '../assets/mintlayer_icon.png'
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { ChainInfoType } from "../types/Types";
import { notify } from "../utils/util";

const Init = () => {
  const navigate = useNavigate();
  const [netMode, setNetMod] = useState("");
  const [walletMode, setWalletMode] = useState("");
  const [isInitialized, setIsInitialized] = useState(() => {
    const saved = localStorage.getItem("isInitialized");
    return saved ? saved === "true" : false;
  });

  useEffect(() => {
    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = ""; // This line is necessary for some browsers to show the confirmation dialog
    };
    window.addEventListener("beforeunload", handleBeforeUnload);
  }, []);
  useEffect(() => {
    const init_node = async () => {
      try {
        if (netMode !== "" && walletMode !== "") {
          const result: ChainInfoType = await invoke("initialize_node", {
            network: netMode,
            mode: walletMode,
          });
          if (result) {
            setIsInitialized(true);
            notify("Node initialized", "info");
            navigate("/home", {
              state: {
                initChainInfo: result,
                netMode: netMode,
                walletMode: walletMode,
              },
            });
          }
        }
      } catch (err) {
        console.error("Error initializing node: ", err);
        notify("Error occurred while initializing node", "error");
      }
    };
    window.addEventListener("contextmenu", async (event) => {
      event.preventDefault();
      (await contextMenu).popup(
        new LogicalPosition(event.clientX, event.clientY)
      );
    });
    !isInitialized && init_node();
  }, [netMode, walletMode]);

  useEffect(() => {
    localStorage.setItem("isInitialized", isInitialized.toString());
  }, [isInitialized]);

  const contextMenu = Menu.new({
    items: [],
  });
  const InitNetwork = {
    Mainnet: "Mainnet",
    Testnet: "Testnet",
  };

  const WalletMode = {
    Hot: "Hot",
    Cold: "Cold",
  };

  return (
    <div className="container page mt-1 pt-1 w-full">
      {(!netMode || !walletMode) && (
        <div className="banner py-6 ">
          <div className="container w-[100vw]">
            <img
              src={MintlayerIcon}
              alt="mintlayer"
              className="w-40 self-center mb-8 mt-8"
            />

            <p className="text-2xl font-bold">
              Please choose the{" "}
              {!netMode ? "Network" : !walletMode ? "Wallet Mode" : ""} you want
              to use.
            </p>
          </div>
        </div>
      )}
      {!netMode ? (
        <div className="flex flex-col items-center space-y-2">
          <button
            onClick={() => setNetMod(InitNetwork.Mainnet)}
            className="py-2 px-4 rounded w-24 bg-[#69EE96] hover:bg-black text-[#000000] font-bold hover:text-[#69EE96]"
          >
            Mainnet
          </button>
          <button
            onClick={() => setNetMod(InitNetwork.Testnet)}
            className="py-2 px-4 rounded w-24 bg-[#69EE96] hover:bg-black text-[#000000] font-bold hover:text-[#69EE96]"
          >
            Testnet
          </button>
        </div>
      ) : !walletMode ? (
        <div className="flex justify-center space-x-16 items-center">
          <div className="bg-white space-y-4 w-[40vw] py-16 px-8 shadow rounded rounded-2 justify-center items-center">
            <button
              className="py-1 px-4 rounded w-48 bg-[#69EE96] text-[#000000] font-bold hover:text-[#69EE96] hover:bg-black text-xl"
              onClick={() => setWalletMode(WalletMode.Hot)}
            >
              Hot
            </button>
            <p className="text-start">
              Hot mode is the standard operating mode with all wallet functions
              enabled. The wallet will be connected to the internet, allowing
              you to stake and perform all operations. If you are unsure which
              options to choose, select 'Hot'.
            </p>
          </div>
          <div className="bg-white w-[40vw] space-y-2 py-16 px-8 shadow rounded rounded-2 justify-center items-center">
            <button
              className="py-1 px-4 rounded w-48 bg-[#C4FCCA] text-[#0D372F] font-bold hover:text-[#69EE96] hover:bg-black text-xl"
              onClick={() => setWalletMode(WalletMode.Cold)}
            >
              Cold
            </button>
            <p className="text-start">
              Cold mode runs a limited version of the node, allowing the wallet
              to function offline for enhanced security. In this mode, the
              wallet cannot sync, check balances, or create transactions, but it
              can sign imported transactions. Staking is also disabled.
            </p>
          </div>
        </div>
      ) : (
        <></>
      )}
    </div>
  );
};

export default Init;
