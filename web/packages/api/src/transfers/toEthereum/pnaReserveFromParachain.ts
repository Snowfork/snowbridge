import {
    AssetRegistry,
    EthereumChain,
    EthereumProviderTypes,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import { Context } from "../.."
import { PNAFromParachain } from "./pnaFromParachain"

/** A parachain-native asset whose reserve is the source parachain. */
export class PNAReserveFromParachain<T extends EthereumProviderTypes> extends PNAFromParachain<T> {
    constructor(
        context: Context<T>,
        registry: AssetRegistry,
        route: TransferRoute,
        source: Parachain,
        destination: EthereumChain,
    ) {
        super(context, registry, route, source, destination, "reserveDeposit")
    }
}
