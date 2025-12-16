// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script} from "forge-std/Script.sol";
import {XCM_PRECOMPILE} from "./Constants.sol";
import {IXcm} from "./Interfaces.sol";

contract TestXcm is Script {
    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);
        IXcm xcm = IXcm(XCM_PRECOMPILE);
        xcm.execute(
            hex"051400080100000784c57f342e020109079edaa802000780f330912e300100000784c57f342e1608140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a31020109079edaa80201020004020109079edaa802000740b121912e0104020004020109079edaa8020002093d00080d01020c00010300302f0b71b8ad3cf6dd90adb668e49b2168d652fd2c33c4d6f62a0064a02f388f88e393167d38d2a4a0b8c563f524a8b68e4e5982172c33c4d6f62a0064a02f388f88e393167d38d2a4a0b8c563f524a8b68e4e598217",
            IXcm.Weight({refTime: 15_000_000_000, proofSize: 8_000_000})
        );
        vm.stopBroadcast();
        return;
    }
}
