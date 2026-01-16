import { AbstractProvider } from "ethers"
import { IERC20__factory } from "@snowbridge/contract-types"

export const ETHER_TOKEN_ADDRESS = "0x0000000000000000000000000000000000000000"

export async function erc20Balance(
    ethereum: AbstractProvider,
    tokenAddress: string,
    owner: string,
    spender: string,
) {
    const tokenContract = IERC20__factory.connect(tokenAddress, ethereum)
    const [balance, gatewayAllowance] = await Promise.all([
        tokenContract.balanceOf(owner),
        tokenContract.allowance(owner, spender),
    ])
    return {
        balance,
        gatewayAllowance,
    }
}
