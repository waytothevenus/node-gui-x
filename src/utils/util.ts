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

import { toast } from "react-toastify";

export const DECIMALS = 100000000000;

export const notify = (message: string, type: string) => {
  console.log("notification is displayed");
  switch (type) {
    case "error":
      toast.error(message);
      break;
    case "info":
      toast.info(message);
      break;
    case "success":
      toast.success(message);
      break;
    default:
      toast.info(message);
  }
};


export const formatTransactionSummary = (summary: string) => {
  const formattedSummary = summary.replace(/, /g, ",\n  ");

  return formattedSummary;
};
