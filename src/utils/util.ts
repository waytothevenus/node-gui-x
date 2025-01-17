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
import { bech32m } from "bech32";

export const DECIMAL = 100000000000;

export const encodeToHash = (data: string) => {
  const hash = blake.blake2bHex(data, undefined, 32);
  return hash;
};

export const encode = (prefix: string, data: ArrayLike<number>) => {
  const words = convertTo5BitWords(data);
  let address = bech32m.encode(prefix, words);
  return address;
};

export const encodeToBytesForAddress = (data: string) => {
  const hexString = data.match(/{(.*?)}/);
  if (hexString && hexString[1]) {
    const content = hexString[1];

    const byteArray = new Uint8Array(
      content.match(/.{1,2}/g)?.map((byte) => parseInt(byte, 16)) || []
    );
    return byteArray;
  } else {
    return new Uint8Array();
  }
};

const convertTo5BitWords = (data: ArrayLike<number>): number[] => {
  const words: number[] = [];
  let bitAccumulator = 0;
  let bitCount = 0;

  const byteArray = Array.from(data);

  for (let byte of byteArray) {
    bitAccumulator = (bitAccumulator << 8) | byte;
    bitCount += 8;

    while (bitCount >= 5) {
      words.push(bitAccumulator & 0x1f);
      bitAccumulator >>= 5;
      bitCount -= 5;
    }
  }

  if (bitCount > 0) {
    words.push((bitAccumulator << (5 - bitCount)) & 0x1f);
  }

  return words;
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
