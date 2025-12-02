import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { AssetRegistry } from "@snowbridge/base-types"
import { DOT_LOCATION, erc20Location } from "./xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"
import { CallDryRunEffects, XcmDryRunApiError } from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"

export type MessageDirection = "Inbound" | "Outbound"

export type TipAsset = "DOT" | "ETH"

export type AddTipParams = {
    direction: MessageDirection
    nonce: bigint
    tipAsset: TipAsset
    tipAmount: bigint
}

export type AddTipResult = {
    tx: SubmittableExtrinsic<"promise">
    tipAmount: bigint
    tipAssetLocation: any
    direction: MessageDirection
    nonce: bigint
}

export async function createAddTip(
    assetHub: ApiPromise,
    registry: AssetRegistry,
    params: AddTipParams,
): Promise<AddTipResult> {
    const { direction, nonce, tipAsset, tipAmount } = params

    let tipAssetLocation: any
    if (tipAsset === "DOT") {
        tipAssetLocation = DOT_LOCATION
    } else if (tipAsset === "ETH") {
        tipAssetLocation = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)
    } else {
        throw new Error(`Unsupported tip asset: ${tipAsset}`)
    }

    const versionedAsset = {
        id: tipAssetLocation,
        fun: {
            Fungible: tipAmount,
        },
    }

    let tx: SubmittableExtrinsic<"promise">

    if (direction === "Inbound") {
        tx = assetHub.tx.snowbridgeSystemFrontend.addTip(
            {
                Inbound: nonce,
            },
            versionedAsset,
        )
    } else if (direction === "Outbound") {
        tx = assetHub.tx.snowbridgeSystemFrontend.addTip(
            {
                Outbound: nonce,
            },
            versionedAsset,
        )
    } else {
        throw new Error(`Invalid message direction: ${direction}`)
    }

    return {
        tx,
        tipAmount,
        tipAssetLocation,
        direction,
        nonce,
    }
}

export async function getFee(
    assetHub: ApiPromise,
    registry: AssetRegistry,
    params: AddTipParams,
    signerAddress: string,
): Promise<bigint> {
    const { tx } = await createAddTip(assetHub, registry, params)

    const paymentInfo = await tx.paymentInfo(signerAddress)
    return paymentInfo.partialFee.toBigInt()
}

export type DryRunResult = {
    success: boolean
    errorMessage?: string
    executionResult?: any
}

export async function dryRunAddTip(
    assetHub: ApiPromise,
    registry: AssetRegistry,
    params: AddTipParams,
    signerAddress: string,
): Promise<DryRunResult> {
    try {
        const { tx } = await createAddTip(assetHub, registry, params)

        const origin = { system: { signed: signerAddress } }
        const dryRunResult: Result<CallDryRunEffects, XcmDryRunApiError> =
            await assetHub.call.dryRunApi.dryRunCall<Result<CallDryRunEffects, XcmDryRunApiError>>(
                origin,
                tx,
                5,
            )

        const success = dryRunResult.isOk && dryRunResult.asOk.executionResult.isOk

        if (!success) {
            let errorMessage = "Unknown error"

            if (dryRunResult.isOk && dryRunResult.asOk.executionResult.isErr) {
                const error = dryRunResult.asOk.executionResult.asErr
                errorMessage = error.toHuman() as string
            } else if (dryRunResult.isErr) {
                errorMessage = dryRunResult.asErr.toHuman() as string
            }

            return {
                success: false,
                errorMessage,
                executionResult: dryRunResult.toHuman(),
            }
        }

        return {
            success: true,
            executionResult: dryRunResult.asOk.executionResult.toHuman(),
        }
    } catch (error) {
        return {
            success: false,
            errorMessage: `Dry run failed: ${error}`,
        }
    }
}

export type AddTipResponse = {
    blockHash: string
    txHash: string
}

export async function signAndSend(
    assetHub: ApiPromise,
    tipResult: AddTipResult,
    account: AddressOrPair,
    options: Partial<SignerOptions>,
): Promise<AddTipResponse> {
    return new Promise((resolve, reject) => {
        tipResult.tx
            .signAndSend(account, options, (result: ISubmittableResult) => {
                if (result.status.isFinalized) {
                    if (result.dispatchError) {
                        if (result.dispatchError.isModule) {
                            const decoded = assetHub.registry.findMetaError(
                                result.dispatchError.asModule,
                            )
                            reject(new Error(`${decoded.section}.${decoded.name}: ${decoded.docs}`))
                        } else {
                            reject(new Error(result.dispatchError.toString()))
                        }
                    } else {
                        resolve({
                            blockHash: result.status.asFinalized.toHex(),
                            txHash: result.txHash?.toHex() ?? "",
                        })
                    }
                }
            })
            .catch((error) => {
                reject(error)
            })
    })
}
