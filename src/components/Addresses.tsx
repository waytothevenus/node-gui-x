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

import { AiOutlineCopy } from "react-icons/ai";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import AddressIcon from "../assets/account_icon.png";
import { notify } from "../utils/util";

const Addresses = (props: {
  isLoading: boolean;
  setIsLoading: (isLoading: boolean) => void;
  loadingMessage: string;
  setLoadingMessage: (loadingMessage: string) => void;
  addresses: Record<string, string>;
  accountId: number;
  walletId: number;
  handleUpdateCurrentAccount: (
    wallet_id: number,
    account_id: number,
    index: string,
    address: string
  ) => void;
}) => {
  const handleAddAddress = async () => {
    try {
      props.setIsLoading(true);
      const unsubscribe = await listen("NewAddress", (event) => {
        const newAddress: {
          wallet_id: number;
          account_id: number;
          index: number;
          address: string;
        } = event.payload as {
          wallet_id: number;
          account_id: number;
          index: number;
          address: string;
        };
        if (newAddress) {
          props.handleUpdateCurrentAccount(
            newAddress.wallet_id,
            newAddress.account_id,
            newAddress.index.toString(),
            newAddress.address
          );
          notify("New address added", "success");
        }
        unsubscribe();
      });
      await invoke("new_address_wrapper", {
        request: { wallet_id: props.walletId, account_id: props.accountId },
      });
      props.setIsLoading(false);
    } catch (err: any) {
      const regex = /Wallet error: (.+)/;
      const errorMessage = new String(err).match(regex);
      if (errorMessage) {
        notify(errorMessage[1], "error");
      }
      props.setIsLoading(false);
    }
  };
  return (
    <div className="pt-0">
      <table className="w-full border border-gray-200 rounded-lg overflow-hidden shadow">
        <thead className="bg-gray-100 ">
          <tr>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold "></th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              ADDRESS
            </th>

            <th className="py-3 px-4 text-center text-gray-600 font-semibold"></th>
          </tr>
        </thead>
        <tbody>
          {Object.values(props.addresses).map((address, index) => (
            <tr
              key={index}
              className="hover:bg-gray-50 transition duration-200"
            >
              <td className="py-2 px-4 border-b border-gray-200">{index}</td>
              <td className="py-2 px-4 border-b border-gray-200">{address}</td>
              <td className="py-2 px-4 border-b border-gray-200">
                <button
                  onClick={() => navigator.clipboard.writeText(address)}
                  className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
                >
                  <AiOutlineCopy />
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <button
        onClick={handleAddAddress}
        className="mt-3 bottom-8 mb-4 py-1 px-2 w-40 text-[#000000] rounded-lg bg-[#69EE96] text-center items-center flex justify-center translation shadow-none border-none "
      >
        <img src={AddressIcon} alt="address_icon" className="pr-2" />
        New Address
      </button>
    </div>
  );
};

export default Addresses;
