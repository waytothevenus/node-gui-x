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

import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { AiOutlineCopy } from "react-icons/ai";
import {
  AccountType,
  TransactionData,
  DelegationBalancesType,
  WalletInfo,
} from "../types/Types";
import { encodeToHash, notify } from "../utils/util";
import { IoCloseSharp } from "react-icons/io5";

const Delegation = (props: {
  isLoading: boolean;
  setIsLoading: (isLoading: boolean) => void;
  loadingMessage: string;
  setLoadingMessage: (loadingMessage: string) => void;
  currentAccount: AccountType | undefined;
  currentAccountId: number;
  maturityPeriod: number;
  delegationBalances: DelegationBalancesType[];
  currentWallet: WalletInfo | undefined;
}) => {
  const [transactionInfo, setTransactionInfo] = useState<TransactionData>();
  const [poolAddress, setPoolAddress] = useState("");
  const [delegationAddress, setDelegationAddress] = useState("");
  const [showDepositModal, setShowDepositModal] = useState(false);
  const [depositAmount, setDepositAmount] = useState(0);
  const [withdrawAmount, setWithdrawAmount] = useState(0);
  const [withdrawAddress, setWithdrawAddress] = useState("");
  const [currentDelegationId, setCurrentDelegationId] = useState("");
  const [showWithdrawModal, setShowWithdrawModal] = useState(false);
  const [showConfirmTransactionModal, setShowConfirmTransactionModal] =
    useState(false);
  const [showSuccessModal, setShowSuccessModal] = useState(false);
  const handleDeposit = async () => {
    setShowDepositModal(false);
    props.setLoadingMessage("Depositing to delegation. Please wait.");
    props.setIsLoading(true);
    try {
      const unsubscribe = await listen("DelegateStaking", (event) => {
        const transactionInfo = event.payload as TransactionData;
        if (transactionInfo) {
          setTransactionInfo(transactionInfo);
          setShowConfirmTransactionModal(true);
        }
        unsubscribe();
      });
      await invoke("delegate_staking_wrapper", {
        request: {
          wallet_id: props.currentWallet?.wallet_id
            ? props.currentWallet.wallet_id
            : 0,
          account_id: props.currentAccountId,
          delegation_id: currentDelegationId,
          delegation_amount: depositAmount.toString(),
        },
      });

      props.setIsLoading(false);
    } catch (error) {
      notify(new String(error).toString(), "error");
    }
    props.setIsLoading(false);
  };
  const handleWithdraw = async () => {
    props.setLoadingMessage("Withdrawing from delegation. Please wait.");
    props.setIsLoading(true);
    setShowWithdrawModal(false);
    try {
      const unsubscribe = await listen("SendDelegationToAddress", (event) => {
        const transactionInfo = event.payload as TransactionData;
        if (transactionInfo) {
          setTransactionInfo(transactionInfo);
          setShowConfirmTransactionModal(true);
        }
        unsubscribe();
      });
      await invoke("send_delegation_to_address_wrapper", {
        request: {
          wallet_id: props.currentWallet?.wallet_id
            ? props.currentWallet.wallet_id
            : 0,
          account_id: props.currentAccountId,
          address: withdrawAddress,
          amount: withdrawAmount.toString(),
          delegation_id: currentDelegationId,
        },
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
    }
    props.setIsLoading(false);
  };

  const handleSelectAllAmount = () => {
    setWithdrawAmount(
      parseInt(
        props.currentAccount?.staking_balance[poolAddress].balance.decimal
          ? props.currentAccount?.staking_balance[poolAddress].balance.decimal
          : "0"
      )
    );
  };

  const handleCreateDelegation = async () => {
    props.setLoadingMessage("Creating Delegation. Please wait");
    props.setIsLoading(true);
    try {
      const unsubscribe = await listen("CreateDelegation", (event) => {
        const transactionInfo = event.payload as TransactionData;
        if (transactionInfo) {
          setTransactionInfo(transactionInfo);
          setShowConfirmTransactionModal(true);
          props.setIsLoading(false);
        }
        unsubscribe();
      });
      await invoke("create_delegation_wrapper", {
        request: {
          wallet_id: props.currentWallet?.wallet_id
            ? props.currentWallet.wallet_id
            : 0,
          account_id: props.currentAccountId,
          pool_id: poolAddress,
          delegation_address: delegationAddress,
        },
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
      props.setIsLoading(false);
    }
  };

  const handleConfirmTransaction = async () => {
    props.setLoadingMessage("Confirming transaction. Please wait.");
    props.setIsLoading(true);
    try {
      const unsubscribe = await listen("Broadcast", (event) => {
        const result = event.payload as number;
        if (result === props.currentWallet?.wallet_id) {
          notify("Transaction submitted successfully!", "success");
          setShowSuccessModal(true);
        }
        unsubscribe();
        setShowConfirmTransactionModal(false);
      });
      await invoke("submit_transaction_wrapper", {
        request: {
          wallet_id: transactionInfo?.transaction_info.wallet_id,
          tx: transactionInfo?.transaction_info,
        },
      });

      props.setIsLoading(false);
    } catch (error) {
      props.setIsLoading(false);
      notify(new String(error).toString(), "error");
    }
  };

  return (
    <div className="container pt-0 p-4 shadow-1">
      {showConfirmTransactionModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div
            className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-lg mx-auto relative space-y-4"
            style={{
              display: "inline-block",
              minWidth: "300px",
              maxWidth: "75%",
              whiteSpace: "wrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowConfirmTransactionModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-lg font-bold mb-4">Confirm Transaction</h2>
            <p className="text-start text-bold">Transaction summary</p>
            <div>
              <p className="text-start text-bold">TRANSACTION ID</p>
              <p className="text-start">
                {encodeToHash(
                  JSON.stringify(
                    transactionInfo?.serialized_tx.V1
                      ? transactionInfo.serialized_tx.V1
                      : {}
                  )
                )}
              </p>
            </div>
            {transactionInfo?.serialized_tx.V1.outputs.find(
              (output) => "CreateDelegationId" in output
            ) && (
              <>
                <div>
                  <p className="text-start text-bold">BEGIN OF INPUTS</p>
                  <p className="text-start">
                    -Transaction({"0x"}
                    {
                      transactionInfo.serialized_tx.V1.inputs.find(
                        (output) => "Utxo" in output
                      )?.Utxo.id.Transaction
                    }
                    )
                  </p>
                </div>
                <div>
                  <p className="text-start  text-bold">END OF INPUTS</p>
                </div>
                <div>
                  <p className="text-start">BEGIN OF OUTPUTS</p>
                  <p className="text-start">
                    -CreateDelegationId(Owner(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "CreateDelegationId" in output
                      )?.CreateDelegationId[0]
                    }
                    ), StakingPool(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "CreateDelegationId" in output
                      )?.CreateDelegationId[1]
                    }
                    ))
                  </p>

                  <p className="text-start">
                    -Transfer(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "Transfer" in output
                      )?.Transfer[1]
                    }
                    ,{" "}
                    {parseInt(
                      new String(
                        transactionInfo?.serialized_tx.V1.outputs.find(
                          (output) => "Transfer" in output
                        )?.Transfer[0].Coin.atoms
                      ).toString()
                    ) /
                      parseInt(
                        props.currentAccount?.balance.coins.decimal
                          ? props.currentAccount?.balance.coins.decimal
                          : "0"
                      )}
                    )
                  </p>
                </div>
              </>
            )}
            {transactionInfo?.serialized_tx.V1.outputs.find(
              (output) => "DelegateStaking" in output
            ) && (
              <>
                <div>
                  <p className="text-start text-bold">BEGIN OF INPUTS</p>
                  <p className="text-start">
                    -Transaction(
                    {
                      transactionInfo.serialized_tx.V1.inputs.find(
                        (output) => "Utxo" in output
                      )?.Utxo.id.Transaction
                    }
                    )
                  </p>
                </div>
                <div>
                  <p className="text-start  text-bold">END OF INPUTS</p>
                </div>
                <div>
                  <p className="text-start">BEGIN OF OUTPUTS</p>
                  <p className="text-start">
                    -DelegateStaking(Amount(
                    {parseInt(
                      new String(
                        transactionInfo?.serialized_tx.V1.outputs.find(
                          (output) => "DelegateStaking" in output
                        )?.DelegateStaking[0]?.atoms
                          ? transactionInfo?.serialized_tx.V1.outputs.find(
                              (output) => "DelegateStaking" in output
                            )?.DelegateStaking[0]?.atoms
                          : "0"
                      ).toString()
                    ) / 100000000000}
                    ), Delegation(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "DelegateStaking" in output
                      )?.DelegateStaking[1]
                    }
                    ))
                  </p>

                  <p className="text-start">
                    -Transfer(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "Transfer" in output
                      )?.Transfer[1]
                    }
                    ,{" "}
                    {parseInt(
                      new String(
                        transactionInfo?.serialized_tx.V1.outputs.find(
                          (output) => "Transfer" in output
                        )?.Transfer[0].Coin.atoms
                      ).toString()
                    ) /
                      parseInt(
                        props.currentAccount?.balance.coins.decimal
                          ? props.currentAccount?.balance.coins.decimal
                          : "0"
                      )}
                    )
                  </p>
                </div>
              </>
            )}
            {transactionInfo?.serialized_tx.V1.outputs.find(
              (output) => "LockThenTransfer" in output
            ) && (
              <>
                <div>
                  <p className="text-start text-bold">BEGIN OF INPUTS</p>
                  <p className="text-start">
                    -AccountOutPoint
                    {`
                      ${
                        "nonce:" +
                        "AccountNonce(" +
                        transactionInfo.serialized_tx.V1.inputs
                          .find((output) => "Account" in output)
                          ?.Account.nonce?.toString() +
                        ")," +
                        " account: DelegationBalance(" +
                        transactionInfo.serialized_tx.V1.inputs.find(
                          (output) => "Account" in output
                        )?.Account.account.DelegationBalance[0] +
                        parseInt(
                          new String(
                            transactionInfo.serialized_tx.V1.inputs.find(
                              (output) => "Account" in output
                            )?.Account.account.DelegationBalance[1].atoms
                          ).toString()
                        ) /
                          100000000000
                      }
                     `}
                  </p>
                </div>
                <div>
                  <p className="text-start  text-bold">END OF INPUTS</p>
                </div>
                <div>
                  <p className="text-start">BEGIN OF OUTPUTS</p>
                  <p className="text-start">
                    -LockThenTransfer(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "LockThenTransfer" in output
                      )?.LockThenTransfer[1]
                    }
                    {parseInt(
                      new String(
                        transactionInfo?.serialized_tx.V1.outputs.find(
                          (output) => "LockThenTransfer" in output
                        )?.LockThenTransfer[0].Coin.atoms
                      ).toString()
                    ) / 100000000000}
                    ), OutputTimeLock::ForBlockCount(
                    {
                      transactionInfo.serialized_tx.V1.outputs.find(
                        (output) => "LockThenTransfer" in output
                      )?.LockThenTransfer[2].content
                    }
                    blocks)
                    {}
                  </p>
                </div>
              </>
            )}
            <div>
              <p className="text-start text-bold">END OF OUTPUTS</p>
            </div>
            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={() => {
                handleConfirmTransaction();
              }}
            >
              Confirm and Broadcast
            </button>
          </div>
        </div>
      )}
      {showSuccessModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-lg mx-auto relative space-y-4">
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowSuccessModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-lg font-bold mb-4">Success</h2>
            <p className="text-start">
              Please wait for your transaction to be included in a block
            </p>

            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={() => setShowSuccessModal(false)}
            >
              Okay
            </button>
          </div>
        </div>
      )}
      {showDepositModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50">Deposit</div>
          <div className="bg-white rounded-lg shadow-lg z-10 p-4 pt-3 max-w-lg mx-auto relative space-y-4">
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowDepositModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-xl font-bold text-start">Deposit</h2>
            <h2 className="text-md font-bold text-start mb-4">
              Enter Deposit Amount
            </h2>
            <input
              placeholder="Enter amount"
              type="number"
              className="w-full rounded-lg"
              value={depositAmount}
              onChange={(e) => setDepositAmount(parseInt(e.target.value))}
            />
            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={handleDeposit}
            >
              Deposit
            </button>
          </div>
        </div>
      )}
      {showWithdrawModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50">Withdraw</div>
          <div className="bg-white rounded-lg shadow-lg z-10 p-6 pt-3 max-w-lg mx-auto relative space-y-4">
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowWithdrawModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-xl font-bold text-start">Withdraw</h2>

            <div>
              <h2 className="text-md font-bold text-start mb-2">
                Withdrawal Address
              </h2>
              <input
                placeholder="Enter address"
                type="text"
                className="w-full rounded-lg"
                value={withdrawAddress}
                onChange={(e) => setWithdrawAddress(e.target.value)}
              />
            </div>
            <div>
              <h2 className="text-md font-bold text-start mb-2">
                Enter Withdrawal Amount
              </h2>
              <div className="flex items-center space-x-3">
                <input
                  placeholder="Enter amount"
                  type="number"
                  className="w-full rounded-lg"
                  value={withdrawAmount}
                  onChange={(e) => setWithdrawAmount(parseInt(e.target.value))}
                />
                <button
                  onClick={handleSelectAllAmount}
                  className="bg-green-400 text-black w-[150px] px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
                >
                  Select All
                </button>
              </div>
            </div>
            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={handleWithdraw}
            >
              Withdraw
            </button>
          </div>
        </div>
      )}
      <h1 className="text-start text-lg mt-8 mb-8">Delegation Summary</h1>
      <table>
        <thead className="bg-gray-100 ">
          <tr>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold text-sm">
              DELEGATION ADDRESS
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold text-sm">
              POOL ADDRESS{" "}
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold text-sm">
              DELEGATION BALANCE
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold text-sm"></th>
          </tr>
        </thead>
        <tbody>
          {(props.delegationBalances.find(
            (balance) =>
              balance.wallet_id === props.currentWallet?.wallet_id &&
              balance.account_id === props.currentAccountId
          )
            ? Object.values(
                props.delegationBalances.find(
                  (balance) =>
                    balance.wallet_id === props.currentWallet?.wallet_id &&
                    balance.account_id === props.currentAccountId
                )?.delegations_balance || {}
              )
            : []
          ).map(([pool_id, amount], index) => {
            let delegation_ids = Object.keys(
              props.delegationBalances.find(
                (balance) =>
                  balance.wallet_id === props.currentWallet?.wallet_id &&
                  balance.account_id === props.currentAccountId
              )?.delegations_balance || {}
            );

            return (
              <tr
                key={index}
                className="hover:bg-gray-50 transition duration-200"
              >
                <td className="py-2 px-4 border-b border-gray-200">
                  <div className="flex justify-between space-x-2">
                    <p>
                      {delegation_ids[index].slice(0, 9)}...
                      {delegation_ids[index].slice(-4)}
                    </p>
                    <button
                      onClick={() =>
                        navigator.clipboard.writeText(delegation_ids[index])
                      }
                      className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
                    >
                      <AiOutlineCopy />
                    </button>
                  </div>
                </td>
                <div className="flex justify-between space-x-2">
                  <p>
                    {pool_id.slice(0, 8)}...
                    {pool_id.slice(-4)}
                  </p>
                  <button
                    onClick={() => navigator.clipboard.writeText(pool_id)}
                    className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
                  >
                    <AiOutlineCopy />
                  </button>
                </div>
                <div className="flex justify-between space-x-2">
                  <p>
                    {pool_id.slice(0, 8)}...
                    {pool_id.slice(-4)}
                  </p>
                  <button
                    onClick={() => navigator.clipboard.writeText(pool_id)}
                    className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
                  >
                    <AiOutlineCopy />
                  </button>
                </div>
                <td className="py-2 px-4 border-b border-gray-200">
                  {parseInt(amount.atoms) / 100000000000}
                </td>

                <td className="py-2 px-4 border-b border-gray-200 flex justify-between space-x-2">
                  <button
                    onClick={() => {
                      setShowDepositModal(true);
                      setCurrentDelegationId(delegation_ids[index]);
                    }}
                    className="px-2 py-1 rounded-lg bg-[#69EE96] text-[#000000] hover:text-[#69EE96] hover:bg-black "
                  >
                    DEPOSIT
                  </button>
                  <button
                    onClick={() => {
                      setShowWithdrawModal(true);
                      setCurrentDelegationId(delegation_ids[index]);
                    }}
                    className="px-2 py-1 rounded-lg bg-[#69EE96] text-[#000000] hover:text-[#69EE96] hover:bg-black "
                  >
                    WITHDRAW
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <hr className="my-12 h-[2px] bg-gradient-to-r from-transparent via-neutral-500 to-transparent opacity-25 dark:via-neutral-800" />{" "}
      <h2 className="mt-8 mb-8 text-lg text-start">Create New Delegation</h2>
      <p className="text-start">
        Maturity period: {props.maturityPeriod} blocks (a block takes on average
        120 seconds)
      </p>
      <div className="container pt-8 text-start">
        <p className="pb-2">Pool address for new delegation</p>
        <input
          type="text"
          placeholder="Enter address"
          className="rounded-lg"
          value={poolAddress}
          onChange={(e) => setPoolAddress(e.target.value)}
        />
      </div>
      <div className="container pt-8 text-start">
        <p className="pb-2">Delegation address</p>
        <input
          type="text"
          placeholder="Enter address"
          className="rounded-lg"
          value={delegationAddress}
          onChange={(e) => setDelegationAddress(e.target.value)}
        />
      </div>
      <div>
        <button
          onClick={handleCreateDelegation}
          className="w-60 py-1 px-2 bg-[#69EE96] text-[#000000] rounded-lg hover:text-[#69EE96] hover:bg-black mt-8 mb-8"
        >
          Create Delegation
        </button>
      </div>
    </div>
  );
};

export default Delegation;
