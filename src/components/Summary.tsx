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

import { ChainInfoType } from "../types/Types";

const SummaryTab = (props: {
  network: string;
  chainInfo: ChainInfoType | undefined;
}) => {
  return (
    <div className="m-8 rounded-lg bg-white p-8 text-center">
      <p className="py-10">
        The following is a syncing state of your node. In a healthy network, the
        block timestamp should be close to the current wall-clock time.
      </p>
      <table className="w-[100%] order border-gray-200 shadow">
        <tbody className="rounded-lg">
          <tr key="network" className="border-b">
            <td className="py-2 px-4 font-semibold text-gray-700 border-r border-gray-300">
              Network
            </td>
            <td className="py-2 px-4 font-semibold text-gray-600">
              {props.network}
            </td>
          </tr>
          <tr key="best_block_id" className="border-b">
            <td className="py-2 px-4 font-semibold text-gray-700 border-r border-gray-300 ">
              Best Block ID
            </td>
            <td className="py-2 px-4 font-semibold text-gray-600 w-[85%] overflow-hidden whitespace-nowrap text-ellipsis">
              {props.chainInfo?.best_block_id}
            </td>
          </tr>
          <tr key="best_block_height" className="border-b">
            <td className="py-2 px-4 font-semibold text-gray-700 border-r border-gray-300">
              Best Block Height
            </td>
            <td className="py-2 px-4 font-semibold text-gray-600">
              {props.chainInfo?.best_block_height}
            </td>
          </tr>
          <tr key="best_block_timestamp" className="border-b">
            <td className="py-2 px-4 font-semibold text-gray-700 border-r border-gray-300">
              Best Block Timestamp (GMT)
            </td>
            <td className="py-2 px-4 font-semibold text-gray-600">
              {new Date(
                (props.chainInfo?.best_block_timestamp.timestamp
                  ? props.chainInfo.best_block_timestamp.timestamp
                  : 0) * 1000
              ).toUTCString()}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
};

export default SummaryTab;
