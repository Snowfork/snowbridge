import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { AssetRegistry } from "@snowbridge/base-types"
import { DeliveryFee, Direction, MessageReceipt, Transfer, ValidationResult } from "../../forKusama"

export interface TransferInterface {
    getDeliveryFee(
        sourceAssetHub: ApiPromise,
        destAssetHub: ApiPromise,
        direction: Direction,
        registry: AssetRegistry,
        tokenAddress: string,
    ): Promise<DeliveryFee>

    createTransfer(
        parachain: ApiPromise,
        direction: Direction,
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer>

    validateTransfer(
        connections: {
            sourceAssetHub: ApiPromise
            destAssetHub: ApiPromise
        },
        direction: Direction,
        transfer: Transfer,
    ): Promise<ValidationResult>

    signAndSend(
        parachain: ApiPromise,
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}
