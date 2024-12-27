import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AiOutlineCopy } from "react-icons/ai";
import { IoCloseSharp } from "react-icons/io5";
import { encodeToHash, notify, DECIMAL } from "../utils/util";
import {
  AccountType,
  WalletInfo,
  Data,
  ChainInfoType,
  StakingBalancesType,
} from "../types/Types";

const Staking = (props: {
  isLoading: boolean;
  setIsLoading: (isLoading: boolean) => void;
  loadingMessage: string;
  setLoadingMessage: (loadingMessage: string) => void;
  chainInfo: ChainInfoType | undefined;
  maturityPeriod: number;
  currentAccount: AccountType | undefined;
  currentWallet: WalletInfo | undefined;
  stakingBalances: StakingBalancesType[];
  currentAccountId: number | undefined;
  currentWalletId: number | undefined;
  handleUpdateStakingState: (enabled: boolean) => void;
}) => {
  const [currentPoolId, setCurrentPoolId] = useState("");
  const [pledgeAmount, setPledgeAmount] = useState(0);
  const [costPerBlock, setCostPerBlock] = useState(0);
  const [marginRatio, setMarginRatio] = useState(0);
  const [decommissionAddress, setDecommissionAddress] = useState("");
  const [isStakingStarted, setIsStakingStarted] = useState(
    props.currentAccount?.staking_enabled
  );
  const [showDecommissionModal, setShowDecommissionModal] = useState(false);
  const [poolAddress, setPoolAddress] = useState("");
  const [receiveAddress, setReceiveAddress] = useState("");
  const [transactionInfo, setTransactionInfo] = useState<Data>();
  const [showConfirmTransactionModal, setShowConfirmTransactionModal] =
    useState(false);
  const [showSuccessModal, setShowSuccessModal] = useState(false);
  const handleToggleStaking = async () => {
    try {
      props.setLoadingMessage(
        isStakingStarted
          ? "Stopping Staking. Please wait."
          : "Starting Staking, Please wait."
      );
      props.setIsLoading(true);
      const unsubscribe = await listen("ToggleStaking", (event) => {
        if (Array.isArray(event.payload)) {
          const [wallet_id, account_id, enabled] = event.payload;
          if (
            wallet_id === props.currentWalletId &&
            account_id === props.currentAccountId &&
            event.payload
          ) {
            setIsStakingStarted(enabled);
            props.handleUpdateStakingState(enabled);
            notify(enabled ? "Staking started" : "Staking stopped", "success");
          }
        }
        unsubscribe();
      });
      await invoke("toggle_staking_wrapper", {
        request: {
          wallet_id: props.currentWalletId ? props.currentWalletId : 0,
          account_id: props.currentAccountId ? props.currentAccountId : 0,
          enabled: !isStakingStarted,
        },
      });

      await invoke("toggle_staking_wrapper", {
        request: {
          wallet_id: props.currentWalletId ? props.currentWalletId : 0,
          account_id: props.currentAccountId ? props.currentAccountId : 0,
          enabled: !isStakingStarted,
        },
      });

      props.setIsLoading(false);
    } catch (error) {
      const errorMessage = new String(error);
      notify(errorMessage.toString(), "error");
    }
    props.setIsLoading(false);
  };
  const handleDecommissionStaking = async () => {
    try {
      props.setLoadingMessage("Decommissioning Staking Pool. Please wait.");
      props.setIsLoading(true);
      const unsubscribe = await listen("DecommissionPool", (event) => {
        const transactionResult = event.payload as Data;
        if (transactionResult) {
          const transactionResult = event.payload as Data;
          if (transactionResult) {
            setTransactionInfo(transactionResult);
            setShowConfirmTransactionModal(true);
          }
        }
        unsubscribe();
      });
      await invoke("decommission_pool_wrapper", {
        request: {
          wallet_id: props.currentWalletId ? props.currentWalletId : 0,
          account_id: props.currentAccountId ? props.currentAccountId : 0,
          pool_id: currentPoolId,
          output_address: receiveAddress,
        },
      });
      await invoke("decommission_pool_wrapper", {
        request: {
          wallet_id: props.currentWalletId ? props.currentWalletId : 0,
          account_id: props.currentAccountId ? props.currentAccountId : 0,
          pool_id: currentPoolId,
          output_address: receiveAddress,
        },
      });
      props.setIsLoading(false);
    } catch (error) {
      const regex = /Wallet error: (.+)/;
      const errorMessage = new String(error).match(regex);
      if (errorMessage) {
        notify(errorMessage[1], "error");
      }
    }
    setShowDecommissionModal(false);
  };

  const handleCreateStakingPool = async () => {
    try {
      props.setLoadingMessage("Creating Staking Pool. Please wait");
      props.setIsLoading(true);
      const unsubscribe = await listen("StakeAmount", (event) => {
        const transactionResult = event.payload as Data;
        if (transactionResult) {
          setTransactionInfo(transactionResult);
          setShowConfirmTransactionModal(true);
        }
        unsubscribe();
      });
      const unsubscribe = await listen("StakeAmount", (event) => {
        const transactionResult = event.payload as Data;
        if (transactionResult) {
          setTransactionInfo(transactionResult);
          setShowConfirmTransactionModal(true);
        }
        unsubscribe();
      });
      await invoke("stake_amount_wrapper", {
        request: {
          wallet_id: props.currentWalletId ? props.currentWalletId : 0,
          account_id: props.currentAccountId ? props.currentAccountId : 0,
          pledge_amount: pledgeAmount.toString(),
          mpt: marginRatio.toString(),
          cost_per_block: costPerBlock.toString(),
          decommission_address: decommissionAddress,
        },
      });
      props.setIsLoading(false);
    } catch (error) {
      const regex = /Wallet error: (.+)/;
      const errorMessage = new String(error).match(regex);
      if (errorMessage) {
        notify(errorMessage[1], "error");
      }
    }
    props.setIsLoading(false);
  };
  const handleConfirmTransaction = async () => {
    try {
      props.setLoadingMessage("Confirming transaction. Please wait.");
      props.setIsLoading(true);
      const unsubscribe = await listen("Broadcast", (event) => {
        const result = event.payload as number;
        if (result === props.currentWallet?.wallet_id) {
          notify("Transaction submitted successfully!", "success");
          setShowConfirmTransactionModal(false);
          setShowSuccessModal(true);
        }
        unsubscribe();
      });
      await invoke("submit_transaction_wrapper", {
        request: {
          wallet_id: transactionInfo?.transaction_info.wallet_id,
          tx: transactionInfo?.transaction_info,
        },
      });
      await invoke("submit_transaction_wrapper", {
        request: {
          wallet_id: transactionInfo?.transaction_info.wallet_id,
          tx: transactionInfo?.transaction_info,
        },
      });
      props.setIsLoading(false);
    } catch (error) {
      notify(new String(error).toString(), "error");
      props.setIsLoading(false);
    }
  };

  return (
    <div className="container overflow-y-auto px-4 pt-1 py-2">
      <style>{`
        /* Hide the spin buttons for WebKit-based browsers */
        input[type="number"]::-webkit-inner-spin-button,
        input[type="number"]::-webkit-outer-spin-button {
          -webkit-appearance: none;
          margin: 0;
        }

        /* Hide the spin buttons for Firefox */
        input[type="number"] {
          -moz-appearance: textfield;
        }
      `}</style>
      {showDecommissionModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-lg mx-auto relative space-y-4">
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowDecommissionModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-lg font-bold mb-4">
              Decommission Staking Pool
            </h2>
            <p className="text-start">Pool address to decommission</p>

            <input
              placeholder="Enter address"
              type="text"
              className="w-full rounded-lg"
              value={poolAddress}
              onChange={(e) => setPoolAddress(e.target.value)}
            />
            <p className="text-start">
              Address that will receive the proceeds from the staking pool
            </p>

            <input
              placeholder="Enter address"
              type="text"
              className="w-full rounded-lg"
              value={receiveAddress}
              onChange={(e) => setReceiveAddress(e.target.value)}
            />
            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={handleDecommissionStaking}
            >
              Decommission
            </button>
          </div>
        </div>
      )}

      {showConfirmTransactionModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div
            className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-lg mx-auto relative space-y-4"
            style={{ minWidth: "300px", maxWidth: "90%" }}
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
            <div>
              <p className="text-start text-bold">BEGIN OF INPUTS</p>
              <p className="text-start">
                -Transaction({"0x"}
                {
                  transactionInfo?.serialized_tx.V1.inputs.find(
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
              {transactionInfo?.serialized_tx.V1.outputs.find(
                (output) => "CreateStakePool" in output
              ) ? (
                <>
                  <p className="text-start">
                    -CreateStakePool(Id(
                    {new String(
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "CreateStakePool" in output
                      )?.CreateStakePool[0]
                    ).toString()}
                    ), Pledge(
                    {pledgeAmount}), Staker(
                    {new String(
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "CreateStakePool" in output
                      )?.CreateStakePool[1].staker
                    ).toString()}
                    ), VRFPubKey(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "CreateStakePool" in output
                      )?.CreateStakePool[1].vrf_public_key
                    }
                    ), Margin Ratio(
                    {
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "CreateStakePool" in output
                      )?.CreateStakePool[1].margin_ratio_per_thousand
                    }
                    ), CostPerBlock(
                    {parseInt(
                      new String(
                        transactionInfo?.serialized_tx.V1.outputs.find(
                          (output) => "CreateStakePool" in output
                        )?.CreateStakePool[1].cost_per_block.atoms
                      ).toString()
                    ) / DECIMAL}
                    ))
                  </p>
                  <p className="text-start">
                    -Transfer(
                    {new String(
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "Transfer" in output
                      )?.Transfer[1]
                    ).toString()}
                    ,{" "}
                    {parseInt(
                      new String(
                        transactionInfo?.serialized_tx.V1.outputs.find(
                          (output) => "Transfer" in output
                        )?.Transfer[0].Coin.atoms
                      ).toString()
                    ) / DECIMAL}
                    )
                  </p>
                </>
              ) : (
                <>
                  <p className="text-start">
                    -LockThenTransfer(
                    {new String(
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "LockThenTransfer" in output
                      )?.LockThenTransfer[1]
                    ).toString()}
                    ,{" "}
                    {parseInt(
                      new String(
                        transactionInfo?.serialized_tx.V1.outputs.find(
                          (output) => "LockThenTransfer" in output
                        )?.LockThenTransfer[0]?.Coin?.atoms
                      ).toString()
                    ) / DECIMAL}
                    {", "}OutputTimeLock::ForBlockCount(
                    {new String(
                      transactionInfo?.serialized_tx.V1.outputs.find(
                        (output) => "LockThenTransfer" in output
                      )?.LockThenTransfer[2].content
                    ).toString()}{" "}
                    blocks))
                  </p>
                </>
              )}
            </div>
            <div>
              <p className="text-start text-bold">END OF OUTPUTS</p>
            </div>
            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={() => {
                handleConfirmTransaction();
                setShowConfirmTransactionModal(false);
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
      <div className="border border-gray-200  rounded-lg w-full py-6">
        <p className="font-bold text-lg text-center">RUN STAKING POOLS</p>
        <p className="text-center py-6">
          {isStakingStarted
            ? "Staking is active"
            : "Staking has not yet started"}
        </p>
        <button
          className={
            isStakingStarted
              ? "py-1 px-4 border text-[#E02424] border-[#E02424] bg-white rounded-lg transition-all duration-200 hover:outline-none hover:bg-[#E02424] hover:text-white hover:border-[#E02424]"
              : "w-40 py-1 px-2 rounded-lg bg-[#69EE96] text-[#000000]  hover:text-[#69EE96] hover:bg-black "
          }
          onClick={handleToggleStaking}
        >
          {isStakingStarted ? "STOP STAKING" : "START STAKING"}
        </button>
      </div>
      <p className="text-lg text-start py-8">Staking Pool Summary</p>
      <table className=" rounded-lg overflow-hidden shadow">
        <thead className="bg-gray-100 ">
          <tr>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold"></th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              POOL ADDRESS
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              MARGIN RATIO
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              COST PER BLOCK
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              POOL BALANCE
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold"></th>
          </tr>
        </thead>
        <tbody>
          {(props.stakingBalances.find(
            (balance) =>
              balance.wallet_id === props.currentWalletId &&
              balance.account_id === props.currentAccountId
          )
            ? Object.values(
                props.stakingBalances.find(
                  (balance) =>
                    balance.wallet_id === props.currentWalletId &&
                    balance.account_id === props.currentAccountId
                )?.staking_balance || {}
              )
            : []
          ).map((stakeInfo, index) => {
            return (
              <tr
                key={stakeInfo.pool_id}
                className="hover:bg-gray-50 transition duration-200"
              >
                <td className="py-2 px-4 border-b border-gray-200">
                  {index + 1}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  <div className="flex justify-between space-x-2">
                    <p>
                      {stakeInfo.pool_id.slice(0, 9)}...
                      {stakeInfo.pool_id.slice(-4)}
                    </p>
                    <button
                      onClick={() =>
                        navigator.clipboard.writeText(stakeInfo.pool_id)
                      }
                      className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
                    >
                      <AiOutlineCopy />
                    </button>
                  </div>
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {stakeInfo.margin_ratio_per_thousand}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {stakeInfo.cost_per_block.decimal}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {stakeInfo.balance.decimal}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  <button
                    onClick={() => {
                      setShowDecommissionModal(true);
                      setCurrentPoolId(stakeInfo.pool_id);
                    }}
                    className="py-1 px-4 border text-[#E02424] border-[#E02424] bg-white rounded-lg transition-all duration-200 hover:outline-none hover:bg-[#E02424] hover:text-white hover:border-[#E02424]"
                  >
                    DECOMMISSION
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <p className="text-lg text-start py-8">Create Staking Pool</p>
      <p className="text-start">
<<<<<<< HEAD
        Maturity period: {props.maturityPeriod} blocks (a block takes on average 
        120 seconds)
=======
        Maturity period: 2000 blocks (a block takes on average 120 seconds)
>>>>>>> 0284165 (fix(frontend): fix issue related to loading messages)
      </p>
      <div className="container pt-4">
        <p className="text-start">Pledge amount for the new staking pool</p>
        <input
          type="number"
          placeholder="Enter amount"
          className=" rounded-lg"
          value={pledgeAmount}
          onChange={(e) => setPledgeAmount(parseInt(e.target.value))}
        />
      </div>
      <div className="container pt-4">
        <p className="text-start">Cost per block</p>
        <input
          type="number"
          placeholder="Enter amount"
          className=" rounded-lg"
          value={costPerBlock}
          onChange={(e) => setCostPerBlock(parseInt(e.target.value))}
        />
      </div>
      <div className="container pt-4">
        <p className="text-start">
          Margin ratio per thousand. The decimal must be in the range [0.0001,
          1.000] or [0.1%, 100%]
        </p>
        <input
          type="number"
          placeholder="Enter amount"
          step="0.001"
          min={0}
          className=" rounded-lg"
          value={marginRatio}
          onChange={(e) => setMarginRatio(parseFloat(e.target.value))}
        />
      </div>
      <div className="container pt-4 pb-2">
        <p className="text-start">Decommission address</p>
        <input
          placeholder="Enter address"
          className=" rounded-lg border-black p-2"
          value={decommissionAddress}
          type="text"
          onChange={(e) => setDecommissionAddress(e.target.value)}
        />
      </div>
      <div>
        <button
          onClick={handleCreateStakingPool}
          className="w-60 py-1 px-2 rounded-lg bg-[#69EE96] text-[#000000]  hover:text-[#69EE96] hover:bg-black mt-8 mb-8"
        >
          Create Staking Pool
        </button>
      </div>
    </div>
  );
};

export default Staking;
