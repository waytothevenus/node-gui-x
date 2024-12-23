import { useEffect, useState, MouseEvent, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { exit } from "@tauri-apps/plugin-process";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { save, open } from "@tauri-apps/plugin-dialog";
import * as bip39 from "@scure/bip39";
import { wordlist } from "@scure/bip39/wordlists/english";
import { RiInformation2Line } from "react-icons/ri";
import { PiShareNetworkBold } from "react-icons/pi";
import { ToastContainer } from "react-toastify";
import _ from "lodash";
import "react-toastify/dist/ReactToastify.css";

import { IoCloseSharp } from "react-icons/io5";
import MintlayerIcon from "../assets/mintlayer_icon.png";
import TransactionIcon from "../assets/transaction_icon.png";
import AddressIcon from "../assets/address_icon.png";
import SendIcon from "../assets/send_icon.png";
import StakingIcon from "../assets/staking_icon.png";
import DelegationIcon from "../assets/delegation_icon.png";
import ConsoleIcon from "../assets/console_icon.png";
import WalletIcon from "../assets/wallet_icon.png";
import AccountIcom from "../assets/account_icon.png";
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

function Home() {
  const InitNetwork = {
    Mainnet: "Mainnet",
    Testnet: "Testnet",
  };

  const WalletMode = {
    Hot: "Hot",
    Cold: "Cold",
  };

  const [walletsInfo, setWalletsInfo] = useState<WalletInfo[]>([]);
  const [netMode, setNetMod] = useState("");
  const [walletMode, setWalletMode] = useState("");
  const [currentWallet, setCurrentWallet] = useState<WalletInfo | undefined>(
    walletsInfo?.[0]
  );
  const [chainInfo, setChainInfo] = useState<ChainInfoType | undefined>();
  const [p2pInfo, setP2pInfo] = useState<PeerConnected["PeerConnected"][]>([]);
  const [currentTab, setCurrentTab] = useState("summary");
  const [activeTab, setActiveTab] = useState("transactions");
  const [currentAccount, setCurrentAccount] = useState<AccountType>();
  const [mnemonic, setMnemonic] = useState("");
  const [currentAccountId, setCurrentAccountId] = useState(0);
  const [currentWalletId, setCurrentWalletId] = useState(0);
  const [accountName, setAccountName] = useState("");
  const [stakingBalances, setStakingBalances] = useState<StakingBalancesType[]>(
    []
  );
  const [delegationBalances, setDelegationBalances] = useState<
    DelegationBalancesType[]
  >([]);

  const [showMnemonicModal, setShowMnemonicModal] = useState(false);
  const [showRecoverWalletModal, setShowRecoverWalletModal] = useState(false);
  const [showNewAccountModal, setShowNewAccountModal] = useState(false);
  const [loading, setLoading] = useState(false);
  const [loadingMessage, setLoadingMessage] = useState("");
  const [isInitialized, setIsInitialized] = useState(false);
  const errorListenerInitialized = useRef(false);
  const unsubscribeErrorListenerRef = useRef<UnlistenFn | undefined>(undefined);
  const balanceEventListenerInitialized = useRef(false);
  const unsubscribeBalanceListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );
  const stakingBalanceListenerInitialized = useRef(false);
  const unsubscribeStakingBalanceListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );
  const delegationBalanceListenerInitialized = useRef(false);
  const unsubscribeDelegationBalanceListenerRef = useRef<
    UnlistenFn | undefined
  >(undefined);
  const transactionListEventListenerInitialized = useRef(false);
  const unsubscribeTransactionListListenerRef = useRef<UnlistenFn | undefined>(
    undefined
  );

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
            setChainInfo(result);
            notify("Node initialized", "info");
          }
        }
      } catch (err) {
        console.error("Error initializing node: ", err);
        notify("Error occurred while initializing node", "error");
      }
    };
    !isInitialized && init_node();
    chainStateEventListener();
    p2pEventListener();

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

    setupErrorListener();
    setupBalanceEventListener();
    setupStakingBalanceEventListener();
    setupDelegationBalanceEventListener();
    setupTransactionListEventListener();
    return () => {
      if (unsubscribeErrorListenerRef.current) {
        unsubscribeErrorListenerRef.current();
      }
    };
  }, [netMode, walletMode]);

  useEffect(() => {
    if (!currentWallet) {
      setCurrentWallet(walletsInfo[0]);
    }
  }, [walletsInfo]);

  useEffect(() => {
    if (currentWallet) {
      const firstAccount = Object.values(currentWallet.accounts || {})[0];
      if (!_.isEqual(firstAccount, currentAccount)) {
        setCurrentAccount(firstAccount);
      }

      setWalletsInfo((prevWallets) => {
        const updatedWallets = [...prevWallets];
        updatedWallets[currentWalletId] = currentWallet;
        return updatedWallets;
      });
    }
  }, [currentWallet, currentWalletId]);

  useEffect(() => {
    if (currentAccount) {
      setCurrentWallet((prevWallet) => {
        if (
          !_.isEqual(prevWallet?.accounts?.[currentAccountId], currentAccount)
        ) {
          return {
            ...prevWallet,
            accounts: {
              ...prevWallet?.accounts,
              [currentAccountId]: currentAccount,
            },
          } as WalletInfo;
        }
        return prevWallet;
      });
    }
  }, [currentAccount, currentAccountId]);

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
        return unsubscribe;
      });
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
        return unsubscribe;
      });
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
        console.log("current balance is :", newBalances);
        if (newBalances.balance) {
          setCurrentAccount((currentAccount) => {
            if (currentAccount) {
              return {
                ...currentAccount,
                balance: newBalances.balance,
              };
            }
          });
        }
      });
      return unsubscribe;
    } catch (error) {
      notify("Error setting up  balance listener", "error");
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

        console.log("Staking balance is: ", newStakingBalances);

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
  const transactionListEventListener = async () => {
    try {
      const unsubscribe = await listen("TransactionList", (event) => {
        const newTransactionList = event.payload as {
          wallet_id: number;
          account_id: number;
          transaction_list: TransactionType;
        };

        console.log("transaction list is:", newTransactionList);

        if (newTransactionList.transaction_list) {
          setCurrentAccount((currentAccount) => {
            if (
              currentAccount &&
              newTransactionList.wallet_id === currentWalletId &&
              newTransactionList.account_id === currentAccountId
            ) {
              return {
                ...currentAccount,
                transaction_list: newTransactionList.transaction_list,
              };
            }
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

        console.log("delegation balance: ", newDelegationBalance);

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
      setMnemonic("Error generating mnemoic. Please try again");
    }
  };

  const handleCreateNewWallet = async (e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    setLoadingMessage("Creating wallet. Please wait.");
    try {
      const path = await save({
        defaultPath: "key.dat",
        filters: [{ name: "Key files", extensions: ["dat"] }],
      });

      if (path) {
        setLoading(true);

        try {
          await invoke("add_create_wallet_wrapper", {
            request: {
              file_path: path,
              mnemonic: mnemonic,
              import: true,
              wallet_type: walletMode,
            },
          });

          const unsubscribe = await listen("ImportWallet", (event) => {
            const walletInfo = event.payload as WalletInfo;
            if (walletInfo) {
              setWalletsInfo([...walletsInfo, walletInfo]);
              setLoading(false);
              notify("Wallet created successfully!", "success");
            }
            setLoading(false);
            unsubscribe();
          });
        } catch (invokeError) {
          notify("Error occured while creating wallet!", "error");
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
        filters: [{ name: "Key files", extensions: ["dat"] }],
      });

      if (path) {
        setLoading(true);

        try {
          await invoke("add_create_wallet_wrapper", {
            request: {
              file_path: path,
              mnemonic: mnemonic,
              import: false,
              wallet_type: walletMode,
            },
          });

          const unsubscribe = await listen("ImportWallet", (event) => {
            const walletInfo: WalletInfo = event.payload as WalletInfo;

            if (walletInfo) {
              setWalletsInfo((prevWallets) => [...prevWallets, walletInfo]);
              notify("Wallet recovered successfully", "success");
            }
            setLoading(false);
            unsubscribe();
          });
        } catch (invokeError) {
          notify("Error occured while recovering wallet!", "error");
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
    setLoadingMessage("Opening wallet. Please wait.");
    try {
      const filePath = await open({
        filters: [
          {
            name: "Key file",
            extensions: ["*"],
          },
        ],
      });

      if (filePath) {
        setLoading(true);
        await invoke("add_open_wallet_wrapper", {
          request: {
            file_path: filePath,
            wallet_type: walletMode,
          },
        });

        const unsubscribe = await listen("OpenWallet", (event) => {
          const walletInfo: WalletInfo = event.payload as WalletInfo;

          if (walletInfo) {
            setWalletsInfo((prevWallets) => [...prevWallets, walletInfo]);
            notify("Wallet opened successfully", "success");
          }

          setLoading(false);
          unsubscribe();
        });
      }
    } catch (error) {
      console.error("Error opening wallet:", error);
      setLoading(false);
    }
  };

  const handleExit = async () => {
    try {
      await invoke("shutdown_wrapper");
    } catch (error) {
      console.error("Error shutting down node", error);
    }
    await exit();
  };

  const handleUpdateCurrentAccountAddresses = (
    index: string,
    address: string
  ) => {
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
    setCurrentAccount(updatedAccount);
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

  const addAccount = (accountId: string, accountInfo: AccountType) => {
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
      await invoke("new_account_wrapper", {
        request: {
          name: accountName,
          wallet_id: currentWalletId,
        },
      });
      const unsubscribe = await listen("NewAccount", (event) => {
        const newAccount: {
          wallet_id: string;
          account_id: string;
          account_info: AccountType;
        } = event.payload as {
          wallet_id: string;
          account_id: string;
          account_info: AccountType;
        };
        if (newAccount) {
          addAccount(newAccount.account_id, newAccount.account_info);
          notify("Account created successfully!", "success");
        }
        unsubscribe();
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
      <div className="container page mt-1 pt-1 w-full">
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
                Hot mode is the standard operating mode with all wallet
                functions enabled. The wallet will be connected to the internet,
                allowing you to stake and perform all operations. If you are
                unsure which options to choose, select 'Hot'.
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
                Cold mode runs a limited version of the node, allowing the
                wallet to function offline for enhanced security. In this mode,
                the wallet cannot sync, check balances, or create transactions,
                but it can sign imported transactions. Staking is also disabled.
              </p>
            </div>
          </div>
        ) : (
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
                            currentWallet?.wallet_id
                              ? currentWallet.wallet_id
                              : ""
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
                            <option
                              key={wallet.wallet_id}
                              value={wallet.wallet_id}
                            >
                              {wallet.path?.substring(
                                wallet.path?.lastIndexOf("\\") + 1
                              )}
                            </option>
                          ))}
                        </select>
                      </div>
                      <div className="relative pl-4 flex items-center justify-center space-x-2">
                        <button
                          className="bg-transparent border-noe shadow-none outine-none hover: outline-none hover:border-none focused: border-none"
                          onClick={() => setShowNewAccountModal(true)}
                        >
                          <img src={AccountIcom} alt="wallet_ico" />
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
                            (currentWallet ? currentWallet : walletsInfo[0])
                              ?.accounts
                              ? (currentWallet ? currentWallet : walletsInfo[0])
                                  .accounts
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
                  {currentTab === "network" && (
                    <NetworkingTab peerInfo={p2pInfo} />
                  )}
                  {currentTab === "transactions" && (
                    <WalletActions
                      netMode={netMode}
                      isLoading={loading}
                      setIsLoading={setLoading}
                      currentWallet={currentWallet}
                      currentAccount={currentAccount}
                      stakingBalances={stakingBalances}
                      delegationBalances={delegationBalances}
                      currentAccountId={currentAccountId}
                      chainInfo={chainInfo}
                      activeTab={activeTab}
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
        )}
      </div>
    </div>
  );
}

export default Home;
