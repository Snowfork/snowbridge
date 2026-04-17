import type { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import type { EthereumProviderTypes } from "@snowbridge/base-types"
import type {
    TipAddition,
    TipAdditionParams,
    TipAdditionResponse,
    ValidatedTipAddition,
} from "../types/addTip"

export interface AddTipInterface<T extends EthereumProviderTypes> {
    fee(params: TipAdditionParams, signerAddress: string): Promise<bigint>

    tx(params: TipAdditionParams): Promise<TipAddition>

    validate(tipAddition: TipAddition, signerAddress: string): Promise<ValidatedTipAddition>

    build(params: TipAdditionParams, signerAddress: string): Promise<ValidatedTipAddition>

    signAndSend(
        tipAddition: TipAddition,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<TipAdditionResponse>
}
