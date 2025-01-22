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

import { useEffect, useState, MouseEvent, useRef } from "react";
import { useLocation } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Menu } from "@tauri-apps/api/menu";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { save, open } from "@tauri-apps/plugin-dialog";
import * as bip39 from "@scure/bip39";
import { wordlist } from "@scure/bip39/wordlists/english";
import { RiInformation2Line } from "react-icons/ri";
import { PiShareNetworkBold } from "react-icons/pi";
import { ToastContainer } from "react-toastify";
import _ from "lodash";
import "react-toastify/dist/ReactToastify.css";
import { toast, Id } from "react-toastify";

import { IoCloseSharp } from "react-icons/io5";
import TransactionIcon from "../assets/transaction_icon.png";
import AddressIcon from "../assets/address_icon.png";
import SendIcon from "../assets/send_icon.png";
import StakingIcon from "../assets/staking_icon.png";
import DelegationIcon from "../assets/delegation_icon.png";
import ConsoleIcon from "../assets/console_icon.png";
import WalletIcon from "../assets/wallet_icon.png";
import AccountIcon from "../assets/account_icon.png";
import MintlayerIcon from "../assets/mintlayer_icon.png";

import { notify } from "../utils/util";
import SummaryTab from "../components/Summary";
import NetworkingTab from "../components/Networking";
import {
  AccountType,
  AmountType,
  BalanceType,
  ChainInfoType,
  DelegationBalancesType,
  P2p,
  PeerConnected,
  PoolInfoType,
  StakingBalancesType,
  TransactionType,
  WalletInfo,
} from "../types/Types";
import WalletActions from "../components/WalletActions";
import { LogicalPosition } from "@tauri-apps/api/dpi";

function Home() {
  const location = useLocation();
  const appWindow = getCurrentWindow();
  const initChainInfo = location.state.initChainInfo as ChainInfoType;
  const netMode = location.state.netMode as string;
  const walletMode = location.state.walletMode as string;
  const toastId = useRef<Id | null>(null);
  const [walletsInfo, setWalletsInfo] = useState<WalletInfo[]>(() => {
    const saved = localStorage.getItem("walletsInfo");
    if (saved && saved !== "undefined") {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error("Error parsing currentWallet from localStorage", e);

        return [];
      }
    }
    return [];
  });
  const [currentWallet, setCurrentWallet] = useState<WalletInfo | undefined>(
    () => {
      const saved = localStorage.getItem("currentWallet");
      if (saved && saved !== "undefined") {
        try {
          return JSON.parse(saved);
        } catch (e) {
          console.error("Error parsing currentWallet from localStorage", e);
          return undefined;
        }
      }
      return undefined;
    }
  );
  const [currentAccount, setCurrentAccount] = useState<AccountType | undefined>(
    () => {
      const saved = localStorage.getItem("currentAccount");
      if (saved && saved !== "undefined") {
        try {
          return JSON.parse(saved);
        } catch (e) {
          console.error("Error parsing currentWallet from localStorage", e);
          return undefined;
        }
      }
      return undefined;
    }
  );
  const [currentAccountId, setCurrentAccountId] = useState<number>(() => {
    const saved = localStorage.getItem("currentAccountId");
    if (saved && saved !== "undefined") {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error("Error parsing currentWallet from localStorage", e);
        return 0;
      }
    }
    return 0;
  });
  const [currentWalletId, setCurrentWalletId] = useState<number>(() => {
    const saved = localStorage.getItem("currentWalletId");
    if (saved && saved !== "undefined") {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error("Error parsing currentWallet from localStorage", e);
        return 0;
      }
    }
    return 0;
  });
  const [chainInfo, setChainInfo] = useState<ChainInfoType | undefined>(
    initChainInfo as ChainInfoType
  );
  const [p2pInfo, setP2pInfo] = useState<PeerConnected["PeerConnected"][]>([]);
  const [currentTab, setCurrentTab] = useState("summary");
  const [activeTab, setActiveTab] = useState("transactions");
  const [mnemonic, setMnemonic] = useState("");
  const [accountName, setAccountName] = useState("");
  const [stakingBalances, setStakingBalances] = useState<StakingBalancesType[]>(
    () => {
      const saved = localStorage.getItem("stakingBalances");
      return saved ? JSON.parse(saved) : [];
    }
  );
  const [delegationBalances, setDelegationBalances] = useState<
    DelegationBalancesType[]
  >(() => {
    const saved = localStorage.getItem("delegationBalances");
    return saved ? JSON.parse(saved) : [];
  });

  const [maturityPeriod, setMaturityPeriod] = useState(0);

  const [showMnemonicModal, setShowMnemonicModal] = useState(false);
  const [showRecoverWalletModal, setShowRecoverWalletModal] = useState(false);
  const [showNewAccountModal, setShowNewAccountModal] = useState(false);
  const [loading, setLoading] = useState(false);
  const [loadingMessage, setLoadingMessage] = useState("");
  const [showProgressBar, setShowProgressBar] = useState(false);
  const [progress, setProgress] = useState(0);
  const errorListenerInitialized = useRef(false);
  const unsubscribeErrorListenerRef = useRef<UnlistenFn | undefined>(undefined);
  const balanceEventListenerInitialized = useRef(false);
  const unsubscribeBalanceListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );

  const unsubscribeWalletBestBlockListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );
  const walletBestBlockEventListenerInitialized = useRef(false);
  const stakingBalanceListenerInitialized = useRef(false);
  const unsubscribeStakingBalanceListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );
  const delegationBalanceListenerInitialized = useRef(false);
  const unsubscribeDelegationBalanceListenerRef = useRef<
    UnlistenFn | undefined
  >(undefined);
  const chainInfoEventListenerInitialized = useRef(false);
  const unsubscribeChainInfoListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );
  const P2pEventListenerInitialized = useRef(false);
  const unsubscribeP2pEventListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );
  const transactionListEventListenerInitialized = useRef(false);
  const unsubscribeTransactionListListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );
  const setupErrorListener = async () => {
    if (!errorListenerInitialized.current) {
      unsubscribeErrorListenerRef.current = await errorListener();
      errorListenerInitialized.current = true;
    }
  };
  const setupBalanceEventListener = async () => {
    if (!balanceEventListenerInitialized.current) {
      unsubscribeBalanceListenerRef.current = await balanceEventListener();
      balanceEventListenerInitialized.current = true;
    }
  };

  const setupStakingBalanceEventListener = async () => {
    if (!stakingBalanceListenerInitialized.current) {
      unsubscribeStakingBalanceListenerRef.current =
        await stakingBalanceEventListener();
      stakingBalanceListenerInitialized.current = true;
    }
  };

  const setupDelegationBalanceEventListener = async () => {
    if (!delegationBalanceListenerInitialized.current) {
      unsubscribeDelegationBalanceListenerRef.current =
        await delegationBalanceEventListener();
      delegationBalanceListenerInitialized.current = true;
    }
  };

  const setupTransactionListEventListener = async () => {
    if (!transactionListEventListenerInitialized.current) {
      unsubscribeTransactionListListenerRef.current =
        await transactionListEventListener();
      transactionListEventListenerInitialized.current = true;
    }
  };
  const setupChainInfoEventListener = async () => {
    if (!chainInfoEventListenerInitialized.current) {
      unsubscribeChainInfoListenerRef.current = await chainStateEventListener();
      chainInfoEventListenerInitialized.current = true;
    }
  };

  const setupWalletBestBlockEventListener = async () => {
    if (!walletBestBlockEventListenerInitialized.current) {
      unsubscribeWalletBestBlockListenerRef.current =
        await walletBestBlockEventListener();
      walletBestBlockEventListenerInitialized.current = true;
    }
  };
  const setupP2pEventListener = async () => {
    if (!P2pEventListenerInitialized.current) {
      unsubscribeP2pEventListenerRef.current = await p2pEventListener();
      P2pEventListenerInitialized.current = true;
    }
  };
  useEffect(() => {
    window.addEventListener("contextmenu", async (event) => {
      event.preventDefault();
      (await contextMenu).popup(
        new LogicalPosition(event.clientX, event.clientY)
      );
    });

    const handleCloseRequested = async () => {
      localStorage.clear();
      await appWindow.close();
    };

    const unlisten = listen("tauri://close-requested", handleCloseRequested);
    setupErrorListener();
    setupBalanceEventListener();
    setupStakingBalanceEventListener();
    setupDelegationBalanceEventListener();
    setupTransactionListEventListener();
    setupChainInfoEventListener();
    setupP2pEventListener();
    setupWalletBestBlockEventListener();

    return () => {
      if (unsubscribeErrorListenerRef.current) {
        unsubscribeErrorListenerRef.current();
      }
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (!currentWallet && walletsInfo.length > 0) {
      setCurrentWallet(walletsInfo[0]);
    } else if (currentWalletId >= 0 && currentWalletId < walletsInfo.length) {
      setCurrentWallet(walletsInfo[currentWalletId]);
    }
    localStorage.setItem("walletsInfo", JSON.stringify(walletsInfo));
  }, [walletsInfo]);

  useEffect(() => {
    if (currentWallet) {
      const accounts = Object.values(currentWallet.accounts || {});
      const updatedAccount =
        accounts[
          currentAccountId >= 0 && currentAccountId < accounts.length
            ? currentAccountId
            : 0
        ];

      if (!_.isEqual(updatedAccount, currentAccount) && updatedAccount) {
        setCurrentAccount(updatedAccount);
      }

      setWalletsInfo((prevWallets) => {
        const updatedWallets = [...prevWallets];
        if (!_.isEqual(updatedWallets[currentWalletId], currentWallet)) {
          updatedWallets[currentWalletId] = currentWallet;
        }
        return updatedWallets;
      });
    } else {
      setCurrentTab("summary");
    }

    localStorage.setItem("currentWallet", JSON.stringify(currentWallet));
  }, [currentWallet]);

  useEffect(() => {
    localStorage.setItem("currentWalletId", currentWalletId.toString());
  }, [currentWalletId]);

  useEffect(() => {
    if (currentAccount) {
      setCurrentWallet((prevWallet) => {
        if (
          !_.isEqual(prevWallet?.accounts?.[currentAccountId], currentAccount)
        ) {
          const updatedWallet = {
            ...prevWallet,
            accounts: {
              ...prevWallet?.accounts,
              [currentAccountId]: currentAccount,
            },
          } as WalletInfo;
          return updatedWallet;
        } else {
          return prevWallet;
        }
      });
    }
    localStorage.setItem("currentAccount", JSON.stringify(currentAccount));
  }, [currentAccount]);

  useEffect(() => {
    localStorage.setItem("currentAccountId", currentAccountId.toString());
  }, [currentAccountId]);

  useEffect(() => {
    localStorage.setItem("stakingBalances", JSON.stringify(stakingBalances));
  }, [stakingBalances]);

  useEffect(() => {
    localStorage.setItem(
      "delegationBalances",
      JSON.stringify(delegationBalances)
    );
  }, [delegationBalances]);

  useEffect(() => {
    const fetchMaturityPeriod = async () => {
      if (chainInfo) {
        try {
          const response = await invoke("get_stake_pool_maturity_distance", {
            bestBlockHeight: chainInfo.best_block_height,
          });
          if (response) {
            setMaturityPeriod(response as number);
          }
        } catch (error) {
          notify("Error fetching maturity period", "error");
        }
      }
    };
    fetchMaturityPeriod();
  }, [chainInfo]);

  const contextMenu = Menu.new({
    items: [],
  });

  const p2pEventListener = async () => {
    try {
      const unsubscribe = await listen("P2p", (event) => {
        const newP2pInfo = event.payload as P2p;
        if ("PeerConnected" in newP2pInfo) {
          const peerInfo = newP2pInfo.PeerConnected;
          setP2pInfo((prevP2pInfo) => {
            const exists = prevP2pInfo.some(
              (peer) => peer.address === peerInfo.address
            );

            if (!exists) {
              return [...prevP2pInfo, peerInfo];
            } else {
              return prevP2pInfo;
            }
          });
        } else if ("PeerDisconnected" in newP2pInfo) {
          const peerId = newP2pInfo.PeerDisconnected as number;
          setP2pInfo((prevP2pInfo) =>
            prevP2pInfo.filter((peer) => peer.id !== peerId)
          );
        }
      });
      return unsubscribe;
    } catch (error) {
      notify("Error setting up p2p event listener", "error");
    }
  };

  const errorListener = async () => {
    try {
      const unsubscribe = await listen("Error", (event) => {
        const regex = /Wallet error: (.+)/;
        const errorMessage = new String(event.payload).match(regex);
        if (errorMessage) {
          notify(errorMessage[1], "error");
        }
        setLoading(false);
      });
      return unsubscribe;
    } catch (error) {
      notify("Error setting up  error listener", "error");
      setLoading(false);
    }
  };

  const chainStateEventListener = async () => {
    try {
      const unsubscribe = await listen("ChainInfo", (event) => {
        const newChainInfo = event.payload as ChainInfoType;
        setChainInfo(newChainInfo);
      });
      return unsubscribe;
    } catch (error) {
      notify("Error setting up chain state listener", "error");
    }
  };
  const balanceEventListener = async () => {
    try {
      const unsubscribe = await listen("Balance", (event) => {
        const newBalances = event.payload as {
          wallet_id: number;
          account_id: number;
          balance: BalanceType;
        };

        if (newBalances && newBalances.wallet_id && newBalances.account_id) {
          setWalletsInfo((currentWalletsInfo) => {
            if (!currentWalletsInfo) {
              return [];
            }

            return currentWalletsInfo.map((wallet) => {
              if (wallet.wallet_id === newBalances.wallet_id) {
                const accounts = { ...wallet.accounts };
                const account = accounts[newBalances.account_id];
                if (
                  account &&
                  !_.isEqual(account.balance, newBalances.balance)
                ) {
                  accounts[newBalances.account_id] = {
                    ...account,
                    balance: newBalances.balance,
                  };
                }

                return {
                  ...wallet,
                  accounts: accounts,
                };
              }
              return wallet;
            });
          });
        }
      });
      return unsubscribe;
    } catch (error) {
      notify("Error setting up balance listener", "error");
    }
  };
  const stakingBalanceEventListener = async () => {
    try {
      const unsubscribe = await listen("StakingBalance", (event) => {
        const newStakingBalances = event.payload as {
          wallet_id: number;
          account_id: number;
          staking_balance: Record<string, PoolInfoType>;
        };

        if (newStakingBalances) {
          setStakingBalances((currentStakingBalance) => {
            const index = currentStakingBalance.findIndex(
              (balance) =>
                balance.wallet_id === newStakingBalances.wallet_id &&
                balance.account_id === newStakingBalances.account_id
            );

            if (index !== -1) {
              const updateBalances = [...currentStakingBalance];
              updateBalances[index] = newStakingBalances;
              return updateBalances;
            } else {
              return [...currentStakingBalance, newStakingBalances];
            }
          });
        }
      });
      return unsubscribe;
    } catch (error) {
      notify("Error setting up  staking balance listener", "error");
    }
  };

  const walletBestBlockEventListener = async () => {
    try {
      const unsubscribe = await listen("WalletBestBlock", (event) => {
        console.log("WalletBestBlockInfo: ", event.payload);
        const newWalletBestBlock = event.payload as {
          wallet_id: number;
          block_info: [account_id: number, best_block: number];
        };

        if (
          chainInfo &&
          newWalletBestBlock.block_info[1] < chainInfo.best_block_height
        ) {
          setShowProgressBar(true);
          setProgress(newWalletBestBlock.block_info[1]);

          const progressPercentage =
            (newWalletBestBlock.block_info[1] / chainInfo.best_block_height) *
            100;

          if (toastId.current === null) {
            toastId.current = toast(
              `Wallet syncing: ${progressPercentage.toFixed(2)}%`
            );
          } else {
            toast.update(toastId.current, {
              render: `Wallet syncing: ${progressPercentage.toFixed(2)}%`,
            });
          }
        } else {
          setShowProgressBar(false);
          if (toastId.current !== null) {
            toast.done(toastId.current);
            toastId.current = null;
          }
        }
      });
      return unsubscribe;
    } catch (error) {
      notify("Error setting up wallet best block listener", "error");
    }
  };
  const transactionListEventListener = async () => {
    try {
      const unsubscribe = await listen("TransactionList", (event) => {
        const newTransactionList = event.payload as {
          wallet_id: number;
          account_id: number;
          transaction_list: TransactionType;
        };

        if (newTransactionList.transaction_list) {
          setWalletsInfo((currentWalletsInfo) => {
            if (currentWalletsInfo) {
              const accounts = {
                ...currentWalletsInfo[newTransactionList.wallet_id]?.accounts,
              };
              const account = accounts[newTransactionList.account_id];
              if (
                account &&
                !_.isEqual(
                  account.transaction_list,
                  newTransactionList.transaction_list
                )
              ) {
                return currentWalletsInfo.map((wallet, index) => {
                  if (index === newTransactionList.wallet_id) {
                    return {
                      ...wallet,
                      accounts: {
                        ...accounts,
                        [newTransactionList.account_id]: {
                          ...account,
                          transaction_list: newTransactionList.transaction_list,
                        },
                      },
                    };
                  }
                  return wallet;
                });
              }
            }
            return currentWalletsInfo;
          });
        }
      });
      return unsubscribe;
    } catch (error) {
      console.error("Error setting up transaction list listener:", error);
    }
  };
  const delegationBalanceEventListener = async () => {
    try {
      const unsubscribe = await listen("DelegationBalance", (event) => {
        const newDelegationBalance = event.payload as {
          wallet_id: number;
          account_id: number;
          delegations_balance: Record<
            string,
            [pool_id: string, amount: AmountType]
          >;
        };

        if (newDelegationBalance) {
          setDelegationBalances((currentBalances) => {
            const index = currentBalances.findIndex(
              (balance) =>
                balance.wallet_id === newDelegationBalance.wallet_id &&
                balance.account_id === newDelegationBalance.account_id
            );
            if (index !== -1) {
              const updateBalances = [...currentBalances];
              updateBalances[index] = newDelegationBalance;
              return updateBalances;
            } else {
              return [...currentBalances, newDelegationBalance];
            }
          });
        }
      });
      return unsubscribe;
    } catch (error) {
      console.error("Error setting up delegation balance listener:", error);
    }
  };
  const createNewWallet = () => {
    try {
      const newMnemonic = bip39.generateMnemonic(wordlist, 256);
      setMnemonic(newMnemonic);
      setShowMnemonicModal(true);
    } catch (error) {
      console.error(error);
      setMnemonic("Error generating mnemonic. Please try again");
    }
  };

  const handleCreateNewWallet = async (e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    try {
      const path = await save({
        defaultPath: "key.dat",
        filters: [{ name: "Key files", extensions: [] }],
      });

      if (path) {
        try {
          setLoadingMessage("Creating wallet. Please wait");
          setLoading(true);
          const unsubscribe = await listen("ImportWallet", (event) => {
            const walletInfo = event.payload as WalletInfo;
            if (walletInfo) {
              setWalletsInfo([...walletsInfo, walletInfo]);
              notify("Wallet created successfully!", "success");
            }
            setLoading(false);
            unsubscribe();
          });
          await invoke("add_create_wallet_wrapper", {
            request: {
              file_path: path,
              mnemonic: mnemonic,
              import: true,
              wallet_type: walletMode,
            },
          });
        } catch (invokeError) {
          notify("Error in creating wallet!", "error");
          setLoading(false);
          console.error(
            "Error during invoke:",
            invokeError instanceof Error ? invokeError.message : invokeError
          );
        }
        setMnemonic("");

        setShowMnemonicModal(false);
      } else {
        console.error("No file selected");
      }
    } catch (err) {
      console.error(
        "Error while selecting file:",
        err instanceof Error ? err.message : err
      );
    }
  };

  const handleRecoverWallet = async () => {
    setLoadingMessage("Recovering wallet. Please wait.");
    try {
      const path = await save({
        defaultPath: "key.dat",
        filters: [{ name: "Key files", extensions: [] }],
      });

      if (path) {
        setLoading(true);

        try {
          const unsubscribe = await listen("ImportWallet", (event) => {
            const walletInfo: WalletInfo = event.payload as WalletInfo;

            if (walletInfo) {
              setWalletsInfo((prevWallets) => [...prevWallets, walletInfo]);
              notify("Wallet recovered successfully", "success");
            }
            setLoading(false);
            unsubscribe();
          });
          await invoke("add_create_wallet_wrapper", {
            request: {
              file_path: path,
              mnemonic: mnemonic,
              import: false,
              wallet_type: walletMode,
            },
          });
        } catch (invokeError) {
          notify("Error in recovering wallet!", "error");
          console.error(
            "Error during invoke:",
            invokeError instanceof Error ? invokeError.message : invokeError
          );
        }
        setMnemonic("");
        setShowRecoverWalletModal(false);
      } else {
        console.error("No file selected");
      }
    } catch (err) {
      console.error(
        "Error while selecting file:",
        err instanceof Error ? err.message : err
      );
    }
  };

  const recoverWallet = () => {
    setShowRecoverWalletModal(true);
  };

  const handleOpenWallet = async () => {
    try {
      const filePath = await open({
        filters: [
          {
            name: "Key file",
            extensions: [],
          },
        ],
      });

      if (filePath) {
        const unsubscribe = await listen("OpenWallet", (event) => {
          const walletInfo: WalletInfo = event.payload as WalletInfo;
          console.log("Wallet info: ", walletInfo);

          if (walletInfo) {
            setWalletsInfo((prevWallets) => [...prevWallets, walletInfo]);
            notify("Wallet opened successfully", "success");
          }

          unsubscribe();
        });
        await invoke("add_open_wallet_wrapper", {
          request: {
            file_path: filePath,
            wallet_type: walletMode,
          },
        });
      }
    } catch (error) {
      console.error("Error opening wallet:", error);
    }
  };

  const handleExit = async () => {
    try {
      await invoke("shutdown_wrapper");
    } catch (error) {
      console.error("Error shutting down node", error);
    }
    await appWindow.close();
  };

  const handleUpdateCurrentAccountAddresses = (
    wallet_id: number,
    account_id: number,
    index: string,
    address: string
  ) => {
    if (
      wallet_id === currentWallet?.wallet_id &&
      account_id === currentAccountId
    ) {
      const updatedAccount: AccountType = {
        addresses: {
          ...currentAccount?.addresses,
          [index]: address,
        },
        name: currentAccount?.name,
        staking_enabled: currentAccount?.staking_enabled,
        balance: currentAccount?.balance,
        staking_balance: currentAccount?.staking_balance,
        delegations_balance: currentAccount?.delegations_balance,
        transaction_list: currentAccount?.transaction_list,
      } as AccountType;
      if (updatedAccount) {
        setCurrentAccount(updatedAccount);
      }
    }
  };

  const handleUpdateCurrentWalletEncryptionState = (
    wallet_id: number,
    encrypted: string
  ) => {
    const updatedWallet: WalletInfo = {
      accounts: currentWallet?.accounts ? currentWallet.accounts : {},
      wallet_id: wallet_id,
      path: currentWallet?.path ? currentWallet.path : "",
      encryption: encrypted,
      best_block: currentWallet?.best_block
        ? currentWallet.best_block
        : ["", 0],
      wallet_type: currentWallet?.wallet_type ? currentWallet.wallet_type : "",
    };

    setCurrentWallet(updatedWallet);
  };

  const handleRemoveWallet = (wallet_id: number) => {
    setWalletsInfo((wallets) =>
      wallets.filter((wallet) => wallet.wallet_id !== wallet_id)
    );
    setCurrentWallet(undefined);
    setCurrentWalletId(0);
    setCurrentAccount(undefined);
    setCurrentAccountId(0);
  };

  const handleUpdateStakingState = (enabled: boolean) => {
    setCurrentAccount((currentAccount) => {
      if (currentAccount) {
        return {
          ...currentAccount,
          staking_enabled: enabled,
        };
      }
    });
  };

  const addAccount = (accountId: number, accountInfo: AccountType) => {
    setCurrentWallet(
      (prevWallet) =>
        ({
          ...prevWallet,
          accounts: {
            ...prevWallet?.accounts,
            [accountId]: accountInfo,
          },
        } as WalletInfo)
    );
  };

  const handleCreateNewAccount = async () => {
    setLoading(true);
    setLoadingMessage("Creating new account. Please wait.");
    try {
      const unsubscribe = await listen("NewAccount", (event) => {
        const [wallet_id, account_id, account_info] = event.payload as [
          wallet_id: number,
          account_id: number,
          account_info: AccountType
        ];
        if (account_info && wallet_id === currentWalletId) {
          addAccount(account_id, account_info);
          notify("Account created successfully!", "success");
        }
        unsubscribe();
      });
      await invoke("new_account_wrapper", {
        request: {
          name: accountName,
          wallet_id: currentWalletId,
        },
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
    }
    setLoading(false);
    setShowNewAccountModal(false);
  };

  return (
    <div className="home-page">
      <ToastContainer />
      {!showMnemonicModal && !showRecoverWalletModal && loading && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div className="bg-opacity-50 z-10 p-6 max-w-lg mx-auto relative space-y-4">
            <div className="loader px-10">{loadingMessage}</div>
          </div>
        </div>
      )}
      {showNewAccountModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-lg mx-auto relative space-y-4">
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowNewAccountModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-lg font-bold mb-4">New Account</h2>
            <input
              placeholder="Enter a name"
              type="text"
              className="w-full rounded-lg"
              value={accountName}
              onChange={(e) => setAccountName(e.target.value)}
            />

            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={() => handleCreateNewAccount()}
            >
              Create
            </button>
          </div>
        </div>
      )}
      {showMnemonicModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          {loading ? (
            <div className="bg-opacity-50 z-10 p-6 max-w-lg mx-auto relative space-y-4">
              <div className="loader px-10">Creating wallet. Please wait.</div>
            </div>
          ) : (
            <div className="bg-white rounded-lg shadow-lg z-10 p-6 max-w-lg mx-auto relative space-y-4">
              <>
                <button
                  className="absolute top-2 right-2 text-gray-600 "
                  onClick={() => setShowMnemonicModal(false)}
                >
                  <IoCloseSharp />
                </button>
                <h2 className="text-lg font-bold mb-4">
                  Create New {walletMode} Wallet
                </h2>
                <p className="mb-4">Your Wallet Mnemonic</p>
                <textarea
                  defaultValue={mnemonic}
                  rows={3}
                  contentEditable={false}
                  className="w-full shadow-[1px] resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 border border-gray-300 rounded-lg"
                />
                <button
                  className="bg-green-400 text-black w-full px-4 py-2 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
                  onClick={(e) => handleCreateNewWallet(e)}
                >
                  Create
                </button>
              </>
            </div>
          )}
        </div>
      )}
      {showRecoverWalletModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          {loading ? (
            <div className="bg-opacity-50 z-10 p-6 max-w-lg mx-auto relative space-y-4">
              <div className="loader px-10">
                Recovering wallet. Please wait.
              </div>
            </div>
          ) : (
            <div className="bg-white rounded-lg shadow-lg z-10 p-6 max-w-lg mx-auto relative space-y-4">
              <>
                <button
                  className="absolute top-2 right-2 text-gray-600 "
                  onClick={() => setShowRecoverWalletModal(false)}
                >
                  <IoCloseSharp />
                </button>
                <h2 className="text-lg font-bold mb-4">
                  Recover New {walletMode} Wallet
                </h2>
                <p className="mb-4">Your Wallet Mnemonic</p>
                <textarea
                  defaultValue={mnemonic}
                  rows={3}
                  onChange={(e) => setMnemonic(e.target.value)}
                  className="w-full shadow-[1px] resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 border border-gray-300 rounded-lg"
                />
                <button
                  className="bg-green-400 text-black w-full px-4 py-2 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
                  onClick={handleRecoverWallet}
                >
                  Recover
                </button>
              </>
            </div>
          )}
        </div>
      )}
      <div className="w-full pt-1">
        <div className="grid grid-cols-12">
          <div className="col-span-3">
            <div className="flex flex-col  space-y-4 p-4  rounded w-full overflow-y-auto">
              <div className="flex justify-center items-center w-[20vw] ">
                <img src={MintlayerIcon} alt="sidebar_icon" />
              </div>

              <>
                <button
                  onClick={() => createNewWallet()}
                  className="w-full text-[#000000] rounded  transition border-none shadow-none text-left py-2 px-1"
                >
                  Create New {walletMode} Wallet
                </button>
                <button
                  onClick={() => recoverWallet()}
                  className="w-full text-[#000000] rounded  transition border-none shadow-none text-left py-2 px-1"
                >
                  Recover {walletMode} Wallet
                </button>
                <button
                  onClick={() => handleOpenWallet()}
                  className="w-full text-[#000000] rounded  transition border-none shadow-none text-left py-2 px-1"
                >
                  Open {walletMode} Wallet
                </button>
                <button
                  onClick={() => handleExit()}
                  className="w-full text-[#ff0000] rounded transition border-none py-2 px-1 shadow-none text-left"
                >
                  Exit
                </button>
                <hr className="my-12 h-[2px] bg-gradient-to-r from-transparent via-neutral-500 to-transparent opacity-25 dark:via-neutral-800" />
              </>
              {walletsInfo.length !== 0 && (
                <>
                  <div className="relative flex items-center justify-center space-x-2">
                    <img src={WalletIcon} alt="wallet_ico" />
                    <select
                      value={
                        currentWallet?.wallet_id ? currentWallet.wallet_id : ""
                      }
                      onChange={(e) => {
                        setCurrentWallet(
                          walletsInfo.find(
                            (wallet) =>
                              wallet.wallet_id == parseInt(e?.target.value)
                          )
                        );
                        setCurrentWalletId(parseInt(e.target.value));
                      }}
                      className="block w-[16vw] bg-white px-2 border-gray-300 text-gray-700 py-2  rounded-lg shadow-sm focus:outline-none  "
                    >
                      {walletsInfo.map((wallet) => (
                        <option key={wallet.wallet_id} value={wallet.wallet_id}>
                          {wallet.path?.substring(
                            wallet.path?.lastIndexOf("\\") + 1
                          )}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="relative pl-4 flex items-center justify-center space-x-2">
                    <button
                      className="bg-transparent border-noe shadow-none outline-none hover:border-none focused: border-none"
                      onClick={() => setShowNewAccountModal(true)}
                    >
                      <img src={AccountIcon} alt="wallet_ico" />
                    </button>
                    <select
                      onChange={(e) => {
                        setCurrentAccountId(parseInt(e.target.value));
                        setCurrentAccount(
                          Object.values(
                            currentWallet?.accounts
                              ? currentWallet.accounts
                              : {}
                          )[parseInt(e.target.value)]
                        );
                      }}
                      value={currentAccountId}
                      className="block w-[16vw] bg-white px-2 border-gray-300 text-gray-700 py-2  rounded-lg shadow-sm focus:outline-none  "
                    >
                      {Object.entries(
                        (currentWallet
                          ? currentWallet
                          : walletsInfo[currentWalletId]
                        )?.accounts
                          ? (currentWallet
                              ? currentWallet
                              : walletsInfo[currentWalletId]
                            ).accounts
                          : {}
                      ).map(([index, account]) => (
                        <option key={index} value={index}>
                          {account?.name ? account?.name : "Account " + index}
                        </option>
                      ))}
                    </select>
                  </div>
                  {walletMode === "Hot" && (
                    <button
                      onClick={() => {
                        setCurrentTab("transactions");
                        setActiveTab("transactions");
                      }}
                      className="mb-4 py-2 px-2 text-[#000000] rounded text-left items-center flex justify-left translation shadow-none border-none w-full"
                    >
                      <img src={TransactionIcon} className="pr-2" />
                      Transactions
                    </button>
                  )}
                  <button
                    onClick={() => {
                      setCurrentTab("transactions");
                      setActiveTab("addresses");
                    }}
                    className="mb-4 py-2 px-2 text-[#000000] rounded text-left items-center flex justify-left translation shadow-none border-none w-full"
                  >
                    <img src={AddressIcon} className="pr-2" />
                    Addresses
                  </button>
                  {walletMode === "Hot" && (
                    <button
                      onClick={() => {
                        setCurrentTab("transactions");
                        setActiveTab("send");
                      }}
                      className="mb-4 py-2 px-2 text-[#000000] rounded text-left items-center flex justify-left translation shadow-none border-none w-full"
                    >
                      <img src={SendIcon} className="pr-2" />
                      Send
                    </button>
                  )}
                  {walletMode === "Hot" && (
                    <button
                      onClick={() => {
                        setCurrentTab("transactions");
                        setActiveTab("staking");
                      }}
                      className="mb-4 py-2 px-2 text-[#000000] rounded text-left items-center flex justify-left translation shadow-none border-none w-full"
                    >
                      <img src={StakingIcon} className="pr-2" />
                      Staking
                    </button>
                  )}
                  {walletMode === "Hot" && (
                    <button
                      onClick={() => {
                        setCurrentTab("transactions");
                        setActiveTab("delegation");
                      }}
                      className="mb-4 py-2 px-2 text-[#000000] rounded text-left items-center flex justify-left translation shadow-none border-none w-full"
                    >
                      <img src={DelegationIcon} className="pr-2" />
                      Delegation
                    </button>
                  )}
                  <button
                    onClick={() => {
                      setCurrentTab("transactions");
                      setActiveTab("console");
                    }}
                    className="mb-4 py-2 px-2 text-[#000000] rounded text-left items-center flex justify-left translation shadow-none border-none w-full"
                  >
                    <img src={ConsoleIcon} className="pr-2" />
                    Console
                  </button>
                  {showProgressBar && chainInfo && (
                    <div className="w-full bg-neutral-200 dark:bg-neutral-600">
                      <div
                        className={`bg-primary p-0.5 text-center text-xs font-medium leading-none text-primary-100 width: ${
                          progress / chainInfo?.best_block_height
                        }`}
                      >
                        {progress.toString() +
                          " / " +
                          chainInfo.best_block_height.toString()}
                      </div>
                    </div>
                  )}
                </>
              )}
            </div>
          </div>
          <div className="col-span-9 ">
            <div className="bg-[#F3F4F6] h-full">
              <div className="flex p-8">
                <button
                  onClick={() => setCurrentTab("summary")}
                  className={`flex items-center justify-center w-full pl-2  transition-colors duration-300 rounded-tr-[0] rounded-br-[0] ${
                    currentTab === "summary"
                      ? "bg-gray-200 text-gray-800"
                      : "bg-white-500 text-black"
                  }`}
                >
                  <RiInformation2Line />
                  <span className="pl-2">Summary</span>
                </button>
                <button
                  onClick={() => setCurrentTab("network")}
                  className={`flex items-center justify-center w-full p-2 transition-colors duration-300 rounded-tl-[0] rounded-bl-[0] ${
                    currentTab === "network"
                      ? "bg-gray-200 text-gray-800"
                      : "bg-white-500 text-black"
                  }`}
                >
                  <PiShareNetworkBold />
                  <span className="pl-2">Networking</span>
                </button>
              </div>
              {currentTab === "summary" && (
                <SummaryTab network={netMode} chainInfo={chainInfo} />
              )}
              {currentTab === "network" && <NetworkingTab peerInfo={p2pInfo} />}
              {currentTab === "transactions" && (
                <WalletActions
                  netMode={netMode}
                  isLoading={loading}
                  setIsLoading={setLoading}
                  loadingMessage={loadingMessage}
                  setLoadingMessage={setLoadingMessage}
                  currentWallet={currentWallet}
                  currentAccount={currentAccount}
                  stakingBalances={stakingBalances}
                  delegationBalances={delegationBalances}
                  currentAccountId={currentAccountId}
                  chainInfo={chainInfo}
                  activeTab={activeTab}
                  maturityPeriod={maturityPeriod}
                  handleUpdateCurrentAccount={
                    handleUpdateCurrentAccountAddresses
                  }
                  handleUpdateCurrentWalletEncryptionState={
                    handleUpdateCurrentWalletEncryptionState
                  }
                  handleUpdateStakingState={handleUpdateStakingState}
                  handleRemoveWallet={handleRemoveWallet}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Home;
