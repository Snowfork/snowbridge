import type { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import type { CallDryRunEffects, XcmDryRunApiError } from "@polkadot/types/interfaces"
import type { ISubmittableResult } from "@polkadot/types/types"
import { Result } from "@polkadot/types"
import type { AssetRegistry, EthereumProviderTypes } from "@snowbridge/base-types"
import type { Context } from "../index"
import type {
    TipAddition,
    TipAdditionParams,
    TipAdditionResponse,
    TipAdditionValidationLog,
    ValidatedTipAddition,
} from "../types/addTip"
import { TipAdditionValidationKind } from "../types/addTip"
import type { AddTipInterface } from "./addTipInterface"
import { DOT_LOCATION, ETHER_TOKEN_ADDRESS } from "../assets_v2"
import { ensureValidationSuccess } from "../utils"
import { erc20Location } from "../xcmBuilder"

export class AddTip<T extends EthereumProviderTypes> implements AddTipInterface<T> {
    constructor(
        readonly context: Context<T>,
        private readonly registry: AssetRegistry,
    ) {}

    #tipAssetLocation(tipAsset: TipAdditionParams["tipAsset"]) {
        if (tipAsset === "DOT") {
            return DOT_LOCATION
        }
        if (tipAsset === "ETH") {
            return erc20Location(this.registry.ethChainId, ETHER_TOKEN_ADDRESS)
        }
        throw new Error(`Unsupported tip asset: ${tipAsset}`)
    }

    async fee(params: TipAdditionParams, signerAddress: string): Promise<bigint> {
        const { tx } = await this.tx(params)
        const paymentInfo = await tx.paymentInfo(signerAddress)
        return paymentInfo.partialFee.toBigInt()
    }

    async tx(params: TipAdditionParams): Promise<TipAddition> {
        const assetHub = await this.context.assetHub()
        const { direction, nonce, tipAmount } = params
        const tipAssetLocation = this.#tipAssetLocation(params.tipAsset)

        const versionedAsset = {
            id: tipAssetLocation,
            fun: {
                Fungible: tipAmount,
            },
        }

        let tx
        if (direction === "Inbound") {
            tx = assetHub.tx.snowbridgeSystemFrontend.addTip({ Inbound: nonce }, versionedAsset)
        } else if (direction === "Outbound") {
            tx = assetHub.tx.snowbridgeSystemFrontend.addTip({ Outbound: nonce }, versionedAsset)
        } else {
            throw new Error(`Invalid message direction: ${direction}`)
        }

        return {
            input: params,
            computed: {
                tipAssetLocation,
            },
            tx,
        }
    }

    async validate(tipAddition: TipAddition, signerAddress: string): Promise<ValidatedTipAddition> {
        const assetHub = await this.context.assetHub()
        const extrinsicFee = await tipAddition.tx
            .paymentInfo(signerAddress)
            .then((info: { partialFee: { toBigInt(): bigint } }) => info.partialFee.toBigInt())
        const logs: TipAdditionValidationLog[] = []

        try {
            const origin = { system: { signed: signerAddress } }
            const dryRunResult: Result<CallDryRunEffects, XcmDryRunApiError> =
                await assetHub.call.dryRunApi.dryRunCall<
                    Result<CallDryRunEffects, XcmDryRunApiError>
                >(origin, tipAddition.tx, 5)

            const success = dryRunResult.isOk && dryRunResult.asOk.executionResult.isOk

            if (!success) {
                let errorMessage = "Unknown error"

                if (dryRunResult.isOk && dryRunResult.asOk.executionResult.isErr) {
                    errorMessage = dryRunResult.asOk.executionResult.asErr.toHuman() as string
                } else if (dryRunResult.isErr) {
                    errorMessage = dryRunResult.asErr.toHuman() as string
                }

                logs.push({
                    kind: TipAdditionValidationKind.Error,
                    message: `Dry run failed: ${errorMessage}`,
                })

                return {
                    ...tipAddition,
                    logs,
                    success: false,
                    data: {
                        extrinsicFee,
                        errorMessage,
                        executionResult: dryRunResult.toHuman(),
                    },
                }
            }

            return {
                ...tipAddition,
                logs,
                success: true,
                data: {
                    extrinsicFee,
                    executionResult: dryRunResult.asOk.executionResult.toHuman(),
                },
            }
        } catch (error) {
            const errorMessage = `Dry run failed: ${error}`
            logs.push({
                kind: TipAdditionValidationKind.Error,
                message: errorMessage,
            })

            return {
                ...tipAddition,
                logs,
                success: false,
                data: {
                    extrinsicFee,
                    errorMessage,
                },
            }
        }
    }

    async build(params: TipAdditionParams, signerAddress: string): Promise<ValidatedTipAddition> {
        const tipAddition = await this.tx(params)
        return ensureValidationSuccess(await this.validate(tipAddition, signerAddress))
    }

    async signAndSend(
        tipAddition: TipAddition,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<TipAdditionResponse> {
        const assetHub = await this.context.assetHub()

        return new Promise((resolve, reject) => {
            tipAddition.tx
                .signAndSend(account, options, (result: ISubmittableResult) => {
                    if (result.status.isFinalized) {
                        if (result.dispatchError) {
                            if (result.dispatchError.isModule) {
                                const decoded = assetHub.registry.findMetaError(
                                    result.dispatchError.asModule,
                                )
                                reject(
                                    new Error(
                                        `${decoded.section}.${decoded.name}: ${decoded.docs}`,
                                    ),
                                )
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
                .catch((error: unknown) => {
                    reject(error)
                })
        })
    }
}
