import { ApiPromise } from "@polkadot/api"
import { AssetRegistry } from "@snowbridge/base-types"
import { TransferInterface } from "./transferInterface"
import {
    IGatewayV1__factory as IGateway__factory,
    IGatewayV2 as IGateway,
} from "@snowbridge/contract-types"
import { Context } from "../../index"
import {
    buildMessageId,
    DeliveryFee,
    encodeNativeAsset,
    hexToBytes,
    Transfer,
} from "../../toPolkadotSnowbridgeV2"
import { accountId32Location, DOT_LOCATION, erc20Location } from "../../xcmBuilder"
import { paraImplementation } from "../../parachains"
import { ETHER_TOKEN_ADDRESS, swapAsset1ForAsset2 } from "../../assets_v2"
import { beneficiaryMultiAddress, padFeeByPercentage } from "../../utils"
import { resolveInputs } from "../../toPolkadot_v2"
import {
    buildAssetHubXcm,
    buildParachainERC20ReceivedXcmOnDestination,
} from "../../xcmbuilders/toPolkadot/erc20ToParachain"
import { Contract } from "ethers"
import { sendMessageXCM } from "../../xcmbuilders/toPolkadot/erc20ToParachain"

export class ERC20ToParachain implements TransferInterface {
    async getDeliveryFee(
        context:
            | Context
            | {
                  gateway: IGateway
                  assetHub: ApiPromise
                  bridgeHub: ApiPromise
                  destination: ApiPromise
              },
        registry: AssetRegistry,
        tokenAddress: string,
        destinationParaId: number,
        paddFeeByPercentage?: bigint
    ): Promise<DeliveryFee> {
        const { assetHub, bridgeHub, destination } =
            context instanceof Context
                ? {
                      assetHub: await context.assetHub(),
                      bridgeHub: await context.bridgeHub(),
                      destination: await context.parachain(destinationParaId),
                  }
                : context

        const { destParachain } = resolveInputs(registry, tokenAddress, destinationParaId)

        // AssetHub fees
        let assetHubXcm = buildAssetHubXcm(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            1000000000000n,
            1000000000000n,
            accountId32Location(
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            ),
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        console.dir(assetHubXcm.toHuman(), { depth: 100 })
        const bridgeHubImpl = await paraImplementation(bridgeHub)
        const assetHubImpl = await paraImplementation(assetHub)
        let ether = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)

        // Delivery fee BridgeHub to AssetHub
        const deliveryFeeInDOT = await bridgeHubImpl.calculateDeliveryFeeInDOT(
            registry.assetHubParaId,
            assetHubXcm
        )
        // AssetHub execution fee
        let assetHubExecutionFeeDOT = await assetHubImpl.calculateXcmFee(assetHubXcm, DOT_LOCATION)
        // Swap to ether
        const deliveryFeeInEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            deliveryFeeInDOT
        )
        let assetHubExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, assetHubExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        // Destination fees
        let destinationXcm = buildParachainERC20ReceivedXcmOnDestination(
            destination.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            3402823669209384634633746074317682114n,
            3402823669209384634633746074317682114n,
            destParachain.info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        console.dir(destinationXcm.toHuman(), { depth: 100 })
        const destinationImpl = await paraImplementation(destination)
        // Delivery fee AssetHub to Destination
        let destinationDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            destinationParaId,
            destinationXcm
        )
        // Destination execution fee
        let destinationExecutionFeeDOT = await destinationImpl.calculateXcmFee(
            destinationXcm,
            DOT_LOCATION
        )

        // Swap to ether
        const destinationDeliveryFeeEther = await swapAsset1ForAsset2(
            assetHub,
            DOT_LOCATION,
            ether,
            destinationDeliveryFeeDOT
        )
        let destinationExecutionFeeEther = padFeeByPercentage(
            await swapAsset1ForAsset2(assetHub, DOT_LOCATION, ether, destinationExecutionFeeDOT),
            paddFeeByPercentage ?? 33n
        )

        const relayerFee = 1_000_000_000_000_000n // TODO configure
        const totalFeeInWei = deliveryFeeInEther + assetHubExecutionFeeEther + relayerFee
        return {
            assetHubDeliveryFeeEther: deliveryFeeInEther,
            assetHubExecutionFeeEther: assetHubExecutionFeeEther,
            destinationDeliveryFeeEther: destinationDeliveryFeeEther,
            destinationExecutionFeeEther: destinationExecutionFeeEther,
            relayerFee: relayerFee,
            totalFeeInWei: totalFeeInWei,
        }
    }

    async createTransfer(
        destination: ApiPromise,
        registry: AssetRegistry,
        destinationParaId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee
    ): Promise<Transfer> {
        const { tokenErcMetadata, destParachain, ahAssetMetadata, destAssetMetadata } =
            resolveInputs(registry, tokenAddress, destinationParaId)
        const minimalBalance =
            ahAssetMetadata.minimumBalance > destAssetMetadata.minimumBalance
                ? ahAssetMetadata.minimumBalance
                : destAssetMetadata.minimumBalance

        let { address: beneficiary, hexAddress: beneficiaryAddressHex } =
            beneficiaryMultiAddress(beneficiaryAccount)
        let value = fee.totalFeeInWei
        if (tokenAddress === ETHER_TOKEN_ADDRESS) {
            value += amount
        }
        const ifce = IGateway__factory.createInterface()
        const con = new Contract(registry.gatewayAddress, ifce)

        const topic = buildMessageId(
            destinationParaId,
            sourceAccount,
            tokenAddress,
            beneficiaryAddressHex,
            amount
        )

        const xcm = hexToBytes(
            sendMessageXCM(
                destination.registry,
                registry.ethChainId,
                destinationParaId,
                tokenAddress,
                beneficiaryAddressHex,
                amount,
                fee.destinationExecutionFeeEther,
                topic
            ).toHex()
        )
        let assets: any = [encodeNativeAsset(tokenAddress, amount)]
        let claimer: any = []
        const executionFee = fee.assetHubExecutionFeeEther
        const relayerFee = 1_000_000_000_000_000n // TODO configure

        const tx = await con
            .getFunction("sendMessageV2")
            .populateTransaction(xcm, assets, claimer, executionFee, relayerFee, {
                value,
                from: sourceAccount,
            })

        return {
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                destinationParaId,
                amount,
                fee,
            },
            computed: {
                gatewayAddress: registry.gatewayAddress,
                beneficiaryAddressHex,
                beneficiaryMultiAddress: beneficiary,
                totalValue: value,
                tokenErcMetadata,
                ahAssetMetadata,
                destAssetMetadata,
                minimalBalance,
                destParachain,
                topic,
            },
            tx,
        }
    }
}
