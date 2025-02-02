import { MultiAddressStruct } from "@snowbridge/contract-types/src/IGateway";
import { Contract, ContractTransaction } from "ethers";
import { beneficiaryMultiAddress } from "./utils";
import { IGateway__factory } from "@snowbridge/contract-types";
import { getSendFee } from "./toPolkadot";
import { Asset, AssetRegistry, ERC20Metadata } from "./assets_v2";


export const getDeliveryFee = getSendFee

export type TokenTransfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string;
        beneficiaryAccount: string;
        tokenAddress: string;
        destinationParaId: number;
        amount: bigint;
        deliveryFeeInWei: bigint;
        parachainExecutionFeeInDOT: bigint;
    },
    computed: {
        gatewayAddress: string;
        beneficiaryAddressHex: string;
        beneficiaryMultiAddress: MultiAddressStruct;
        totalValue: bigint;
        tokenErcMetadata: ERC20Metadata;
        ahAssetMetadata: Asset;
        destAssetMetadata: Asset;
        minimalBalance: bigint;
    },
    tx: ContractTransaction
}

export async function createTransfer(
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    destinationParaId: number,
    amount: bigint,
    deliveryFeeInWei: bigint,
    parachainExecutionFeeInDOT: bigint,
): Promise<TokenTransfer> {
    const tokenErcMetadata = registry.ethereumChains[registry.ethChainId.toString()].assets[tokenAddress.toLowerCase()];
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const destParachain = registry.parachains[destinationParaId.toString()]
    if (!destParachain) {
        throw Error(`Could not find ${destinationParaId} in the asset registry.`)
    }
    const ahAssetMetadata = registry.parachains[registry.assetHubParaId].assets[tokenAddress.toLowerCase()]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const destAssetMetadata = destParachain.assets[tokenAddress.toLowerCase()]
    if (!destAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on destination parachain ${destinationParaId}.`)
    }
    const minimalBalance = ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
        ? ahAssetMetadata.minimumBalance : destAssetMetadata.minimumBalance

    if (amount < minimalBalance) {
        throw Error(`Amount ${amount} is less than minimum transfer amount ${minimalBalance}.`)
    }

    let { address: beneficiary, hexAddress: beneficiaryAddressHex } = beneficiaryMultiAddress(beneficiaryAccount)
    const value = deliveryFeeInWei
    const ifce = IGateway__factory.createInterface()
    const con = new Contract(registry.gatewayAddress, ifce);
    const tx = await con.getFunction("sendToken").populateTransaction(
        tokenAddress,
        destinationParaId,
        beneficiary,
        parachainExecutionFeeInDOT,
        amount,
        {
            value,
            from: sourceAccount
        }
    )

    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            destinationParaId,
            amount,
            deliveryFeeInWei,
            parachainExecutionFeeInDOT,
        }, computed: {
            gatewayAddress: registry.gatewayAddress,
            beneficiaryAddressHex,
            beneficiaryMultiAddress: beneficiary,
            totalValue: value,
            tokenErcMetadata,
            ahAssetMetadata,
            destAssetMetadata,
            minimalBalance,
        },
        tx,
    }
}

export async function validateTransfer(transfer: TokenTransfer) {
    // Check if user has token allowance
    // Check if user has total fee

    // Check if asset can be received on asset hub (dry run)
    // Check if asset can be received on destination chain (dry run)
}