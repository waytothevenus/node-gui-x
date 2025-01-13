import { AiOutlineCopy } from "react-icons/ai";
import { TransactionType } from "../types/Types";
import { DECIMAL } from "../utils/util";

const Transactions = (props: { transactions: TransactionType | undefined }) => {
  return (
    <div>
      <table className="min-w-full border border-gray-200 rounded-lg overflow-hidden shadow">
        <thead className="bg-gray-100 ">
          <tr>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold ">
              #
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              TRANSACTION ID
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              TIMESTAMP (UTC)
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              TYPE
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              AMOUNT
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              STATE
            </th>
          </tr>
        </thead>
        <tbody>
          {props.transactions?.txs?.map((transactionInfo, index) => {
            return (
              <tr
                className="hover:bg-gray-50 transition duration-200"
                key={index}
              >
                <td className="py-2 px-4 border-b border-gray-200">
                  {index + 1}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  <div className="flex justify-between">
                    <p>
                      {transactionInfo.txid.slice(0, 4)}...
                      {transactionInfo.txid.slice(-4)}
                    </p>
                    <button
                      onClick={() =>
                        navigator.clipboard.writeText(transactionInfo.txid)
                      }
                      className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
                    >
                      <AiOutlineCopy />
                    </button>
                  </div>
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {new Date(
                    transactionInfo?.timestamp?.timestamp * 1000
                  ).toLocaleString()}
                </td>
                {(() => {
                  if ("Redeposit" in transactionInfo.tx_type) {
                    return (
                      <td className="py-2 px-4 border-b border-gray-200">
                        Redeposit
                      </td>
                    );
                  }
                  if ("Received" in transactionInfo.tx_type) {
                    return (
                      <td className="py-2 px-4 border-b border-gray-200">
                        Received
                      </td>
                    );
                  }
                  if ("Sent" in transactionInfo.tx_type) {
                    return (
                      <td className="py-2 px-4 border-b border-gray-200">
                        Sent
                      </td>
                    );
                  }
                  if ("Other" in transactionInfo.tx_type) {
                    return (
                      <td className="py-2 px-4 border-b border-gray-200">
                        Other
                      </td>
                    );
                  }
                })()}

                {(() => {
                  if ("Redeposit" in transactionInfo.tx_type) {
                    return (
                      <td className="py-2 px-4 border-b border-gray-200"></td>
                    );
                  }

                  if ("Received" in transactionInfo.tx_type) {
                    const { amount } = transactionInfo.tx_type.Received;
                    return (
                      <td className="py-2 px-4 border-b border-gray-200">
                        {parseInt(new String(amount.atoms).toString()) /
                          DECIMAL}
                      </td>
                    );
                  }

                  if ("Sent" in transactionInfo.tx_type) {
                    const { amount } = transactionInfo.tx_type.Sent;
                    return (
                      <td className="py-2 px-4 border-b border-gray-200">
                        {parseInt(new String(amount.atoms).toString()) /
                          DECIMAL}
                      </td>
                    );
                  }

                  if ("Other" in transactionInfo.tx_type) {
                    return (
                      <td className="py-2 px-4 border-b border-gray-200"></td>
                    );
                  }
                })()}
                {(() => {
                  if ("Confirmed" in transactionInfo.state) {
                    return (
                      <td className="border-b mt-2 border-gray-200 flex items-center justify-center bg-green-200 text-black text-xs font-bold py-1 px-2 rounded-full h-full">
                        Confirmed
                      </td>
                    );
                  }

                  if ("InMempool" in transactionInfo.state) {
                    return (
                      <td className="border-b mt-2 border-gray-200 flex items-center justify-center bg-green-200 text-black text-xs font-bold py-1 px-2 rounded-full h-full">
                        InMempool
                      </td>
                    );
                  }

                  if ("Conflicted" in transactionInfo.state) {
                    return (
                      <td className="border-b mt-2 border-gray-200 flex items-center justify-center bg-green-200 text-black text-xs font-bold py-1 px-2 rounded-full h-full">
                        Conflicted
                      </td>
                    );
                  }

                  if ("Inactive" in transactionInfo.tx_type) {
                    return (
                      <td className="border-b mt-2 border-gray-200 flex items-center justify-center bg-green-200 text-black text-xs font-bold py-1 px-2 rounded-full h-full">
                        Inactive
                      </td>
                    );
                  }

                  if ("Abandoned" in transactionInfo.tx_type) {
                    return (
                      <td className="border-b mt-2 border-gray-200 flex items-center justify-center bg-green-200 text-black text-xs font-bold py-1 px-2 rounded-full h-full">
                        Abandoned
                      </td>
                    );
                  }
                })()}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default Transactions;
