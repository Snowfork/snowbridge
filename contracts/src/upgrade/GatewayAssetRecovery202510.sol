// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import "../Gateway.sol";
import {UnlockNativeTokenParams} from "../v2/Types.sol";

// New Gateway logic contract with an asset recovery initializer
contract GatewayAssetRecovery202510 is Gateway {
    constructor(address beefyClient, address agentExecutor) Gateway(beefyClient, agentExecutor) {}

    function initialize(bytes calldata) external override {
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }
        
// # select token_address, destination_address, amount from transfer_status_to_ethereum where status = 0;
//                 token_address                |            destination_address             |         amount
// --------------------------------------------+-------------------------
//  0x0000000000000000000000000000000000000000 | 0xad8d4c544a6ce24b89841354b2738e026a12bca4 |      350000000000000000
//  0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9 | 0xa9c415d6881e1a992861a7fa6bef3ed4736152c2 |   212116067921877821839
//  0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9 | 0x2265a7503597ab32bab72eaa186e6329fb7b68f3 |    33044703802651993696
//  0x56072c95faa701256059aa122697b133aded9279 | 0x601d579ecd0464a1a090ceef81a703465a1679cd | 90413710543975890000000
//  0x18084fba666a33d37592fa2633fd49a74dd93a88 | 0x601d579ecd0464a1a090ceef81a703465a1679cd |      250830765728855800
//  0x9d39a5de30e57443bff2a8307a4256c8797a3497 | 0x9117900a3794ad6d167dd97853f82a1aa07f9bbc | 16716000000000000000000


        // 0.35 ETH to 0xAd8D4c544a6ce24B89841354b2738E026a12BcA4
        UnlockNativeTokenParams memory params = UnlockNativeTokenParams({
            token: address(0),
            recipient: 0xAd8D4c544a6ce24B89841354b2738E026a12BcA4,
            amount: 350000000000000000
        });
        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, abi.encode(params));

        // 212.116067921877821839 AAVE to 0xa9C415d6881e1a992861A7FA6BEf3Ed4736152c2
        params = UnlockNativeTokenParams({
            token: address(0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9),
            recipient: 0xa9C415d6881e1a992861A7FA6BEf3Ed4736152c2,
            amount: 212116067921877821839
        });
        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, abi.encode(params));

        // 33.044703802651993696 AAVE to 0x2265a7503597AB32baB72EaA186e6329fB7B68f3
        params = UnlockNativeTokenParams({
            token: address(0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9),
            recipient: 0x2265a7503597AB32baB72EaA186e6329fB7B68f3,
            amount: 33044703802651993696
        });
        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, abi.encode(params));

        // 90413.710543975890000000 SKY to 0x601D579eCD0464A1A090cEef81a703465A1679cD
        params = UnlockNativeTokenParams({
            token: address(0x56072C95FAA701256059aa122697B133aDEd9279),
            recipient: 0x601D579eCD0464A1A090cEef81a703465A1679cD,
            amount: 90413710543975890000000
        });
        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, abi.encode(params));

        // 0.250830765728855800 tBTC to 0x601D579eCD0464A1A090cEef81a703465A1679cD
        params = UnlockNativeTokenParams({
            token: address(0x18084fbA666a33d37592fA2633fD49a74DD93a88),
            recipient: 0x601D579eCD0464A1A090cEef81a703465A1679cD,
            amount: 250830765728855800
        });
        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, abi.encode(params));

        // 16716.000000000000000000 sUSDe to 0x9117900a3794AD6D167Dd97853f82A1aA07F9BBc
        params = UnlockNativeTokenParams({
            token: address(0x9D39A5DE30e57443BfF2A8307A4256c8797A3497),
            recipient: 0x9117900a3794AD6D167Dd97853f82A1aA07F9BBc,
            amount: 16716000000000000000000
        });
        HandlersV2.unlockNativeToken(AGENT_EXECUTOR, abi.encode(params));
    }
}
