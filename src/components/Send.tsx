import { useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { IoCloseSharp } from "react-icons/io5";
import { invoke } from "@tauri-apps/api/core";
import { encodeToHash, notify } from "../utils/util";
import { AccountType, Data } from "../types/Types";
const Send = (props: {
  isLoading: boolean;
  setIsLoading: (isLoading: boolean) => void;
  loadingMessage: string;
  setLoadingMessage: (loadingMessage: string) => void;
  currentAccount: AccountType | undefined;
  walletId: number;
  accountId: number;
}) => {
  const [address, setAddress] = useState("");
  const [amount, setAmount] = useState("");
  const [transactionInfo, setTransactionInfo] = useState<Data | undefined>();
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [showSuccessModal, setShowSuccessModal] = useState(false);
  const handleSend = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    event.stopPropagation();
    try {
      await invoke("send_amount_wrapper", {
        request: {
          wallet_id: props.walletId,
          account_id: props.accountId,
          amount: amount,
          address: address,
        },
      });
      const unsubscribe = await listen("SendAmount", (event) => {
        const transactionResult = event.payload as Data;
        if (transactionResult) {
          setTransactionInfo(transactionResult);
          setShowConfirmModal(true);
        }
        unsubscribe();
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
    }
  };

  const handleConfirmTransaction = async () => {
    props.setLoadingMessage("Confirming transaction. Please wait.");
    props.setIsLoading(true);
    try {
      const unsubscribe = await listen("Broadcast", (event) => {
        const result = event.payload as number;
        if (result === props.walletId) {
          notify("Transaction submitted successfully!", "success");
          setShowConfirmModal(false);
          setShowSuccessModal(true);
        }
        unsubscribe();
        props.setIsLoading(false);
      });
      await invoke("submit_transaction_wrapper", {
        request: {
          wallet_id: transactionInfo?.transaction_info.wallet_id,
          tx: transactionInfo?.transaction_info,
        },
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
      props.setIsLoading(false);
    }
  };
  return (
    <div className="container mx-auto p-2">
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

        /* Adjust the position of the clear button */
        .input-container button {
          right: 36px; /* Adjust this value as needed */
        }
      `}</style>

      {showConfirmModal && (
        <div className="fixed inset-0 flex items-center justify-center z-50">
          <div className="absolute inset-0 bg-black opacity-50"></div>
          <div
            className="bg-white rounded-lg shadow-lg z-10 p-4 max-w-xl mx-auto relative space-y-4"
            style={{ minWidth: "300px", maxWidth: "90%" }}
          >
            {/* Close Button */}
            <button
              className="absolute top-2 right-2 bg-transparent border-none shadow-none focus:outline-none "
              onClick={() => setShowConfirmModal(false)}
            >
              <IoCloseSharp />
            </button>
            <h2 className="text-xl font-bold mb-4">Confirm Transaction</h2>
            <p className="text-start text-lg text-bold">Transaction summary</p>
            <div>
              <p className="text-start text-bold">BEGIN OF INPUTS</p>
              <p className="text-start whitespace-nowrap">
                -Transaction id ({""}
                {encodeToHash(
                  JSON.stringify(transactionInfo?.serialized_tx.V1)
                )}
                )
              </p>
              <p className="text-start whitespace-nowrap">
                -Transaction ({"0x"}
                {
                  transactionInfo?.serialized_tx.V1.inputs.find(
                    (output) => "Utxo" in output
                  )?.Utxo.id.Transaction
                }
                {", "}
                {
                  transactionInfo?.serialized_tx.V1.inputs.find(
                    (output) => "Utxo" in output
                  )?.Utxo.index
                }
                )
              </p>
            </div>
            <div>
              <p className="text-start  text-bold">END OF INPUTS</p>
            </div>
            <div>
              <p className="text-start">BEGIN OF OUTPUTS</p>
              <p className="text-start whitespace-nowrap">
                -Transfer({address}, {amount}),
              </p>
              <p className="text-start">
                -Transfer({props.currentAccount?.addresses[0]},{" "}
                {props.currentAccount?.balance.coins.atoms
                  ? props.currentAccount.balance.coins.atoms
                  : 0 - parseInt(amount)}
                ),
              </p>
            </div>
            <div>
              <p className="text-start text-bold">END OF OUTPUTS</p>
            </div>
            <button
              className="bg-green-400 text-black w-full px-2 py-1 rounded-lg hover:bg-[#000000] hover:text-green-400 transition duration-200"
              onClick={() => {
                handleConfirmTransaction();
                setShowConfirmModal(false);
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
      <form
        onSubmit={(e) => handleSend(e)}
        className="flex flex-col items-center space-y-2"
      >
        <p className="text-lg font-semibold text-start w-80">Address</p>
        <div className="relative">
          <input
            type="text"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="Enter address"
            className="w-80 shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 border border-gray-300 rounded-lg py-2 px-3"
          />
          <button
            className="absolute inset-y-0 right-0 hover:outline-none text-gray-800 p-2 rounded focus:outline-none shadow-none"
            onClick={(e) => {
              e.preventDefault();
              setAddress("");
            }}
          >
            <IoCloseSharp />
          </button>
        </div>

        <p className="text-lg font-semibold text-start w-80">Amount</p>
        <div className="relative">
          <button
            className="absolute inset-y-0 right-0 hover:outline-none text-gray-800 p-2 rounded focus:outline-none shadow-none"
            onClick={(e) => {
              e.preventDefault();
              setAmount("");
            }}
          >
            <IoCloseSharp />
          </button>
          <input
            type="number"
            value={amount}
            min={0}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="Enter amount"
            className="w-80 shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 border border-gray-300 rounded-lg py-2 px-3"
          />
        </div>

        <button
          type="submit"
          className="py-1 px-4 rounded-lg bg-[#69EE96] text-[#000000] hover:text-[#69EE96] hover:bg-black "
        >
          Send
        </button>
      </form>
    </div>
  );
};

export default Send;
