import type { InjectedAccountWithMeta, InjectedExtension } from '@polkadot/extension-inject/types';
import { Context, status } from '@snowbridge/api';
import { BrowserProvider, JsonRpcSigner, Network } from 'ethers';
import { Config } from '../Config';

export interface AppProps {
    context?: Context,
    config: Config,
}

export interface WalletInfo {
    isConnected: boolean,
    hasError: boolean,
    error?: string,
}

export interface EthWalletInfo extends WalletInfo {
    signer?: JsonRpcSigner,
    provider?: BrowserProvider,
    network?: Network,
}

export interface SubWalletInfo extends WalletInfo {
    accounts?: InjectedAccountWithMeta[]
    injectedExtensions?: InjectedExtension[]
}

export enum TransactionStatus { Failed, InProgress, Complete }
export type TransactionHistoryItem = {
    when: string
    status: TransactionStatus
    messages: string[]
    type: string
    result: any
}

export type TransactionHistoryStore = {
    items: TransactionHistoryItem[]
}

export interface AccountInfo { name: string, type: 'ethereum' | 'substrate', account: string, balance: bigint }

export interface BridgeStatus {
    statusInfo: status.BridgeStatusInfo,
    channelStatusInfos: { name: string, status: status.ChannelStatusInfo }[]
    assetHubChannel: status.ChannelStatusInfo
    relayers: AccountInfo[]
    accounts: AccountInfo[]
}