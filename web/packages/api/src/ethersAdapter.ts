
import { AbstractProvider, JsonRpcProvider, WebSocketProvider } from "ethers"

export interface SnowbridgeEthereumAdapter<Client> {
    create(url: string): Client
    destroy(client: Client): void
}

export class SnowbridgeEthersAdapter implements SnowbridgeEthereumAdapter<AbstractProvider> {
    create(url: string): AbstractProvider {
        if (url.startsWith("http")) {
            return new JsonRpcProvider(url)
        } else {
            return new WebSocketProvider(url)
        }
    }
    destroy(client: AbstractProvider) {
        client.destroy()
    }
}