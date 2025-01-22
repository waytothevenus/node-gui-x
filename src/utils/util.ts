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
import * as blake from "blakejs";
import { AccountType, TransactionData } from "../types/Types";

export const encodeToHash = (data: string) => {
  const hash = blake.blake2bHex(data, undefined, 32);
  return hash;
};

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

export const getCoinAmount = (
  outputValue: TransactionData | undefined,
  type: string
): number => {
  const findOutput = (key: string) => {
    return outputValue?.serialized_tx.V1.outputs.find(
      (output) => key in output
    );
  };

  const getAtoms = (output: any, key: string) => {
    return output?.[key][0].type === "Coin"
      ? parseInt(output?.[key][0].value.atoms || "0")
      : 0;
  };

  if (type === "LockThenTransfer") {
    const output = findOutput("Transfer");
    return getAtoms(output, "Transfer");
  } else if (type === "LockThennTransfer") {
    const output = findOutput("LockThenTransfer");
    return getAtoms(output, "LockThenTransfer");
  }

  return 0;
};

export const getDecimals = (accountInfo: AccountType | undefined) => {
  return parseInt(accountInfo ? accountInfo.balance.coins.decimal : "0");
};
