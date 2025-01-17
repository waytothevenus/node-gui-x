// Copyright (c) 2024 RBB S.r.l
// opensource@mintlayer.org
// SPDX-License-Identifier: MIT
// Licensed under the MIT License;
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/mintlayer/mintlayer-core/blob/master/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { PeerConnected } from "../types/Types";

const NetworkingTab = (props: {
  peerInfo: PeerConnected["PeerConnected"][];
}) => {
  return (
    <div className="pt-0 mt-8 m-8 rounded-lg text-center bg-white p-8">
      <p className="py-10">
        The following is a list of peers connected to your node
      </p>
      <table className="min-w-full border border-gray-200">
        <thead className="bg-gray-100 ">
          <tr>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              PEER ID
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              #SOCKET
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              INBOUND
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              USER AGENT
            </th>
            <th className="py-3 px-4 text-center text-gray-600 font-semibold">
              VERSION
            </th>
          </tr>
        </thead>
        <tbody>
          {props.peerInfo.map((netInfo) => {
            return (
              <tr
                key={netInfo.address}
                className="hover:bg-gray-50 transition duration-200"
              >
                <td className="py-2 px-4 border-b border-gray-200">
                  {netInfo.id}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {netInfo.address}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {netInfo.inbound === true ? "Inbound" : "Outbound"}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {netInfo.user_agent
                    ?.map((value) => String.fromCharCode(value))
                    .join("")}
                </td>
                <td className="py-2 px-4 border-b border-gray-200">
                  {netInfo.software_version.major}.
                  {netInfo.software_version.minor}.
                  {netInfo.software_version.patch}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default NetworkingTab;
