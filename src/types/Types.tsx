export type RpcAmountType = {
  atoms: string;
  decimal: string;
};

export type BalanceType = {
  coins: RpcAmountType;
  tokens: Record<string, RpcAmountType>;
};

export type PoolInfoType = {
  pool_id: string;
  pledge: RpcAmountType;
  balance: RpcAmountType;
  height: number;
  block_timestamp: { timestamp: number };
  vrf_public_key: string;
  decommission_key: string;
  staker: string;
  margin_ratio_per_thousand: number;
  cost_per_block: RpcAmountType;
};

export type AccountType = {
  name: string;
  addresses: Record<string, string>;
  staking_enabled: boolean;
  balance: BalanceType;
  staking_balance: Record<string, PoolInfoType>;
  delegations_balance: Record<string, [pool_id: string, amount: AmountType]>;
  transaction_list: TransactionType;
};

export type StakingBalancesType = {
  account_id: number;
  wallet_id: number;
  staking_balance: Record<string, PoolInfoType>;
};

export type DelegationBalancesType = {
  wallet_id: number;
  account_id: number;
  delegations_balance: Record<string, [pool_id: string, amount: AmountType]>;
};

export type AmountType = {
  atoms: string;
};

export type TxType =
  | { Received: { amount: AmountType } }
  | { Redeposit: {} }
  | { Sent: { amount: AmountType } }
  | { Other: {} };

export type TxState =
  | {
      Confirmed: {
        height: number;
        timestamp: { timestamp: number };
        someValue: number;
      };
    }
  | { InMempool: { someValue: number } }
  | { Conflicted: { id: string } }
  | { Inactive: { someValue: number } }
  | { Abandoned: {} };

export type TransactionInfoType = {
  txid: string;
  tx_type: TxType;
  timestamp: { timestamp: number };
  state: TxState;
};

export type TransactionType = {
  count: number;
  skip: number;
  total: number;
  txs: TransactionInfoType[];
};

export type WalletInfo = {
  wallet_id: number;
  path: string;
  encryption: string;
  accounts: Record<string, AccountType>;
  best_block: [string, number];
  wallet_type: string;
};

export type AddressInfo = {
  wallet_id: number;
  account_id: string;
  index: number;
  address: string;
};

export type NewAccountResultType = {
  wallet_id: string;
  account_id: string;
  account_info: AccountType;
};

export type ToggleStakingResultType = {
  wallet_id: string;
  account_id: string;
  enabled: boolean;
};

export type ChainInfoType = {
  best_block_height: number;
  best_block_id: string;
  best_block_timestamp: {
    timestamp: number;
  };
  median_time: {
    timestamp: number;
  };
  is_initial_block_download: boolean;
};

export type PeerDisconnected = {
  P2p: {
    PeerDisConnected: number;
  };
};

export type PeerConnected = {
  PeerConnected: {
    id: number;
    services: number;
    address: string;
    inbound: boolean;
    user_agent: number[];
    software_version: {
      major: number;
      minor: number;
      patch: number;
    };
  };
};

export type P2p = PeerConnected | PeerDisconnected;

export type Transaction = {
  V1: {
    version: number | null;
    flags: number;
    inputs: Input[];
    outputs: Output[];
  };
};

export type Input = Utxo | Account;
export type Account = {
  Account: {
    account: {
      DelegationBalance: [string, { atoms: string }];
    };
    nonce: number;
  };
};

export type Utxo = {
  Utxo: {
    id: {
      Transaction: string;
    };
    index: number;
  };
};

export type Output =
  | CreateStakePoolOutput
  | TransferOutput
  | LockThenTransferOutput
  | DelegateStakingOutput
  | IssueFungibleTokenOutput
  | ProduceBlockFromStakeOutput
  | CreateDelegationId;

export type CreateStakePoolOutput = {
  CreateStakePool: [
    string,
    {
      pledge: {
        atoms: string;
      };
      staker: string;
      vrf_public_key: string;
      decommission_key: string;
      margin_ratio_per_thousand: string;
      cost_per_block: {
        atoms: string;
      };
    }
  ];
};

export type CreateDelegationId = {
  CreateDelegationId: [string, string];
};

export type TransferOutput = {
  Transfer: [
    (
      | { type: "Coin"; value: Coin }
      | { type: "TokenV0"; value: TokenV0 }
      | { type: "TokenV1"; value: TokenV1 }
    ),
    string
  ];
};

type Coin = {
  atoms: string;
};

type TokenV0 = TokenTransfer | TokenInsurance | NftInsurance;
type TokenV1 = {
  TokenId: string;
  amount: {
    atoms: string;
  };
};

type TokenTransfer = {
  token_id: string;
  amount: {
    atoms: string;
  };
};

type TokenInsurance = {
  token_ticker: string;
  amount_to_issue: {
    atoms: string;
  };
  number_of_decimals: number;
  metadata_uri: string;
};

type NftInsurance = {
  metadata: {
    creator: {
      public_key: string;
      name: string;
      ticker: string;
      icon_uri: string;
      additional_metadata_uri: string;
      media_uri: string;
      media_hash: string;
    };
  };
};

export type LockThenTransferOutput = {
  LockThenTransfer: [
    (
      | { type: "Coin"; value: Coin }
      | { type: "TokenV0"; value: TokenV0 }
      | { type: "TokenV1"; value: TokenV1 }
    ),
    string,
    {
      content: number;
      type: string;
    }
  ];
};

export type DelegateStakingOutput = {
  DelegateStaking: [
    {
      atoms: string;
    },
    string
  ];
};

export type ProduceBlockFromStakeOutput = {
  ProduceBlockFromStake: [string, string];
};

export type IssueFungibleTokenOutput = {
  IssueFungibleToken: [string];
};

export type Signature = {
  Standard: {
    sighash_type: number;
    raw_signature: number[];
  };
};

export type TransactionData = {
  transaction_info: {
    wallet_id: number;
    tx: { tx: string };
  };
  serialized_tx: Transaction;
};

export type DelegateStakingResult = {
  transaction_info: {
    wallet_id: number;
    tx: { tx: string };
  };
  serialized_tx: Transaction;
  delegation_id: string;
};

type SetStatus = {
  SetStatus: { status: string; print_message: string };
};

type Print = {
  Print: string;
};

type ClearScreen = string;
type PrintHistory = string;
type ClearHistory = string;
type Exit = string;

export type ConsoleCommand =
  | Print
  | ClearScreen
  | PrintHistory
  | ClearHistory
  | Exit
  | SetStatus
  | string;
