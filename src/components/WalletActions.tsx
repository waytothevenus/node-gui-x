import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import Addresses from "./Addresses";
import Console from "./Console";
import Delegation from "./Delegation";
import Send from "./Send";
import Staking from "./Staking";
import Transactions from "./Transactions";
import { IoCloseSharp } from "react-icons/io5";
import { invoke } from "@tauri-apps/api/core";
import { notify } from "../utils/util";
import {
  AccountType,
  ChainInfoType,
  DelegationBalancesType,
  StakingBalancesType,
  WalletInfo,
} from "../types/Types";
const WalletActions = (props: {
  netMode: string;
  isLoading: boolean;
  setIsLoading: (isLoading: boolean) => void;
  loadingMessage: string;
  setLoadingMessage: (loadingMessage: string) => void;
  currentWallet: WalletInfo | undefined;
  currentAccount: AccountType | undefined;
  stakingBalances: StakingBalancesType[];
  delegationBalances: DelegationBalancesType[];
  activeTab: string;
  currentAccountId: number;
  chainInfo: ChainInfoType | undefined;
  handleUpdateCurrentAccount: (
    wallet_id: number,
    account_id: number,
    index: string,
    address: string
  ) => void;
  handleUpdateCurrentWalletEncryptionState: (
    wallet_id: number,
    encrypted: string
  ) => void;
  handleUpdateStakingState: (enabled: boolean) => void;
  handleRemoveWallet: (wallet_id: number) => void;
}) => {
  const [showEncryptWalletModal, setShowEncryptWalletModal] = useState(false);
  const [showUnlockModal, setShowUnlockModal] = useState(false);
  const [walletState, setWalletState] = useState(
    props.currentWallet?.encryption
  );
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [unLockPassword, setUnLockPassword] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    setWalletState(props.currentWallet?.encryption);
  }, [props.currentWallet]);

  useEffect(() => {
    console.log("props.currentaccount is: ", props.currentAccount);
  }, [props.currentAccount]);

  const handleConfirmPasswordChange = (confirmPassword: string) => {
    setConfirmPassword(confirmPassword);

    if (confirmPassword !== password) {
      setError("Passwords do not match.");
    } else if (confirmPassword.length < 8 || confirmPassword.length > 64) {
      setError("Password must be between 8 and 64 characters.");
    } else {
      setError("");
    }
  };

  const handleEncryptWallet = async () => {
    try {
      await invoke("update_encryption_wrapper", {
        request: {
          wallet_id: props.currentWallet?.wallet_id
            ? props.currentWallet.wallet_id
            : 0,
          action: "set_password",
          password: password,
        },
      });
      const unsubscribe = await listen("UpdateEncryption", (event) => {
        const encryptionResult = event.payload as {
          wallet_id: string;
          encryptionState: string;
        };
        if (encryptionResult) {
          setWalletState("EnabledUnLocked");
          props.handleUpdateCurrentWalletEncryptionState(
            props.currentWallet?.wallet_id ? props.currentWallet.wallet_id : 0,
            "EnabledUnlocked"
          );
          setShowEncryptWalletModal(false);
          notify("Wallet encrypted successfully.", "success");
        }
        unsubscribe();
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
    }
    setShowEncryptWalletModal(false);
    setPassword("");
    setConfirmPassword("");
  };

  const handleUpdateWalletEncryption = async () => {
    if (walletState === "EnabledUnLocked") {
      try {
        await invoke("update_encryption_wrapper", {
          request: {
            wallet_id: props.currentWallet?.wallet_id
              ? props.currentWallet.wallet_id
              : 0,
            action: "remove_password",
          },
        });
        const unsubscribe = await listen("UpdateEncryption", (event) => {
          const encryptionResult = event.payload as {
            wallet_id: string;
            encryptionState: string;
          };
          if (encryptionResult) {
            setWalletState("Disabled");
            props.handleUpdateCurrentWalletEncryptionState(
              props.currentWallet?.wallet_id
                ? props.currentWallet.wallet_id
                : 0,
              "Disabled"
            );
            setShowEncryptWalletModal(false);
            notify("Wallet encryption disabled successfully.", "success");
          }
          unsubscribe();
        });
      } catch (error) {
        notify(new String(error).toString(), "error");
      }
    } else {
      setShowEncryptWalletModal(true);
    }
  };

  const handleLockWallet = async () => {
    try {
      await invoke("update_encryption_wrapper", {
        request: {
          wallet_id: props.currentWallet?.wallet_id
            ? props.currentWallet.wallet_id
            : 0,
          action: "lock",
        },
      });

      const unsubscribe = await listen("UpdateEncryption", (event) => {
        const encryptionResult = event.payload as {
          wallet_id: string;
          encryptionState: string;
        };
        if (encryptionResult) {
          setWalletState("EnabledLocked");
          props.handleUpdateCurrentWalletEncryptionState(
            props.currentWallet?.wallet_id ? props.currentWallet.wallet_id : 0,
            "EnabledLocked"
          );
          setShowUnlockModal(false);
          notify("Wallet locked successfully.", "success");
        }
        unsubscribe();
      });
    } catch (err) {
      notify(new String(err).toString(), "error");
    }
  };

  const handleUnlock = async () => {
    try {
      await invoke("update_encryption_wrapper", {
        request: {
          wallet_id: props.currentWallet?.wallet_id
            ? props.currentWallet.wallet_id
            : 0,
          action: "unlock",
          password: unLockPassword,
        },
      });
      const unsubscribe = await listen("UpdateEncryption", (event) => {
        const encryptionResult = event.payload as {
          wallet_id: string;
          encryptionState: string;
        };
        if (encryptionResult) {
          setWalletState("EnabledUnlocked");
          props.handleUpdateCurrentWalletEncryptionState(
            props.currentWallet?.wallet_id ? props.currentWallet.wallet_id : 0,
            "EnabledUnlocked"
          );
          setShowUnlockModal(false);
          notify("Wallet unlocked successfully.", "success");
        }
        unsubscribe();
      });
    } catch (err) {
      notify(new String(err).toString(), "error");
    }
    setShowUnlockModal(false);
    setUnLockPassword("");
  };

  const handleCloseWallet = async (wallet_id: number) => {
    try {
      props.setIsLoading(true);
      props.setLoadingMessage("Closing wallet. Please wait.");
      await invoke("close_wallet_wrapper", {
        walletId: wallet_id,
      });
      const unsubscribe = await listen("CloseWallet", (event) => {
        const closeWalletResult = event.payload as number;
        console.log("Close wallet result: ", closeWalletResult);
        if (closeWalletResult !== undefined) {
          props.handleRemoveWallet(closeWalletResult);
          notify("Wallet closed successfully.", "success");
        }
        unsubscribe();
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
    }
  };

  return (
    <div className="bg-white border border-gray-200 shadowoverflow-y-auto mt-8 p-8 m-8 rounded-lg shadow">
      {showEncryptWalletModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-lg mx-auto relative space-y-4">
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowEncryptWalletModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-lg font-bold mb-4">Encrypt Wallet</h2>
            <input
              placeholder="Enter password"
              type="password"
              className="w-full rounded-lg"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <input
              placeholder="Repeat password"
              type="password"
              className="w-full rounded-lg"
              value={confirmPassword}
              onChange={(e) => handleConfirmPasswordChange(e.target.value)}
            />
            {error && <p className="text-red-500 text-sm mt-1">{error}</p>}
            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={() => handleEncryptWallet()}
            >
              Encrypt Wallet
            </button>
          </div>
        </div>
      )}
      {showUnlockModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-lg mx-auto relative space-y-4">
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowUnlockModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-lg font-bold mb-4">Unlock</h2>
            <input
              placeholder="Enter a password"
              type="password"
              className="w-full rounded-lg"
              value={unLockPassword}
              onChange={(e) => setUnLockPassword(e.target.value)}
            />

            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={handleUnlock}
            >
              Unlock
            </button>
          </div>
        </div>
      )}
      <div className="row flex items-center justify-between pl-8 pr-8 pb-0">
        <div>
          <span className="flex space-x-2">
            <div className="font-thin">My balance: </div>
            <div className="font-bold">
              {props.currentAccount?.balance?.coins?.decimal}{" "}
              {props.netMode === "Mainnet" ? "ML" : "TML"}
            </div>
          </span>
        </div>
        <div className="space-x-2">
          {walletState === "EnabledLocked" && (
            <button
              className="py-1 px-2 rounded-lg bg-[#69EE96] text-[#000000] hover:text-[#69EE96] hover:bg-black "
              onClick={() => {
                setShowUnlockModal(true);
              }}
            >
              UnLock
            </button>
          )}
          {walletState === "EnabledUnLocked" && (
            <button
              className="py-1 px-2 rounded-lg bg-[#69EE96] text-[#000000] hover:text-[#69EE96] hover:bg-black "
              onClick={handleLockWallet}
            >
              Lock
            </button>
          )}
          {walletState !== "EnabledLocked" && (
            <button
              className={`py-1 px-2 ${
                !props.currentWallet
                  ? "bg-gray-400 cursor-not-allowed"
                  : "bg-[#69EE96] hover:text-[#69EE96] hover:bg-black"
              } rounded-lg bg-[#69EE96] text-[#000000] hover:text-[#69EE96] hover:bg-black`}
              onClick={handleUpdateWalletEncryption}
            >
              {walletState === "EnabledUnLocked"
                ? "Disable Wallet Encryption"
                : "Encrypt Wallet"}
            </button>
          )}
          <button
            className={`py-1 px-4 ${
              !props.currentWallet
                ? "bg-gray-400 cursor-not-allowed"
                : "bg-white hover:outline-none hover:bg-[#E02424] hover:text-white hover:border-[#E02424]"
            } mt-8 mb-8 border text-[#E02424] border-[#E02424]  rounded-lg transition-all duration-200 `}
            onClick={() =>
              handleCloseWallet(
                props.currentWallet?.wallet_id
                  ? props.currentWallet.wallet_id
                  : 0
              )
            }
          >
            Close Wallet
          </button>
        </div>
      </div>
      {props.activeTab === "transactions" && (
        <Transactions transactions={props.currentAccount?.transaction_list} />
      )}
      {props.activeTab === "addresses" && (
        <Addresses
          isLoading={props.isLoading}
          setIsLoading={props.setIsLoading}
          loadingMessage={props.loadingMessage}
          setLoadingMessage={props.setLoadingMessage}
          addresses={
            props.currentAccount?.addresses
              ? props.currentAccount.addresses
              : {}
          }
          walletId={
            props.currentWallet?.wallet_id ? props.currentWallet.wallet_id : 0
          }
          accountId={props.currentAccountId}
          handleUpdateCurrentAccount={props.handleUpdateCurrentAccount}
        />
      )}
      {props.activeTab === "send" && (
        <Send
          isLoading={props.isLoading}
          setIsLoading={props.setIsLoading}
          loadingMessage={props.loadingMessage}
          setLoadingMessage={props.setLoadingMessage}
          currentAccount={props.currentAccount}
          walletId={
            props.currentWallet?.wallet_id ? props.currentWallet.wallet_id : 0
          }
          accountId={props.currentAccountId}
        />
      )}
      {props.activeTab === "staking" && (
        <Staking
          isLoading={props.isLoading}
          setIsLoading={props.setIsLoading}
          loadingMessage={props.loadingMessage}
          setLoadingMessage={props.setLoadingMessage}
          chainInfo={props.chainInfo}
          currentAccount={props.currentAccount}
          currentWallet={props.currentWallet}
          stakingBalances={props.stakingBalances}
          currentAccountId={props.currentAccountId}
          currentWalletId={props.currentWallet?.wallet_id}
          handleUpdateStakingState={props.handleUpdateStakingState}
        />
      )}
      {props.activeTab === "delegation" && (
        <Delegation
          isLoading={props.isLoading}
          setIsLoading={props.setIsLoading}
          loadingMessage={props.loadingMessage}
          setLoadingMessage={props.setLoadingMessage}
          currentAccount={props.currentAccount}
          currentAccountId={props.currentAccountId}
          delegationBalances={props.delegationBalances}
          currentWallet={props.currentWallet}
          empty_consensus_reward_maturity_block_count={
            props.chainInfo?.empty_consensus_reward_maturity_block_count
              ? props.chainInfo?.empty_consensus_reward_maturity_block_count
              : 0
          }
        />
      )}
      {props.activeTab === "console" && (
        <Console
          currentAccount={props.currentAccount}
          currentWallet={props.currentWallet}
          currentAccountId={props.currentAccountId}
        />
      )}
    </div>
  );
};
export default WalletActions;
