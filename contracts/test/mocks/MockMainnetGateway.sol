
import {IGateway} from "../../src/interfaces/IGateway.sol";
import {Verification} from "../../src/Verification.sol";
import {SubstrateTypes} from "../../src/SubstrateTypes.sol";
import {
    Channel,
    ChannelID,
    InboundMessage,
    OperatingMode,
    ParaID,
    Command,
    MultiAddress,
    Ticket,
    Costs
} from "../../src/Types.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";


contract MockMainnetGateway is IGateway {

    uint64 internal nonce;
    ParaID internal constant assetHubParaID = ParaID.wrap(1000);

    function emitOutboundMessage() external {
        nonce = nonce + 1;
        bytes memory payload = SubstrateTypes.SendTokenToAssetHubAddress32(
            // WETH
            0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2,
            // Alice
            0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d,
            // xcm fee
            1_000_000_000,
            // amount
            1
        );
        ChannelID channelID = assetHubParaID.into();
        bytes32 messageID = keccak256(abi.encodePacked(channelID, nonce));
        emit IGateway.OutboundMessageAccepted(channelID, nonce, messageID, payload);
    }

    function channelNoncesOf(ChannelID) external view returns (uint64, uint64) {
        return (0, nonce);
    }
    function operatingMode() external view returns (OperatingMode) {
        return OperatingMode.Normal;
    }
    function channelOperatingModeOf(ChannelID channelID) external view returns (OperatingMode) {
        return OperatingMode.Normal;
    }

    function agentOf(bytes32) external view returns (address) {
        return address(0);
    }

    function pricingParameters() external view returns (UD60x18, uint128) {
            return (ud60x18(0), 0);
    }
    function implementation() external view returns (address) {
        return address(0);
    }

    function submitV1(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external {

    }

    function isTokenRegistered(address token) external view returns (bool) {
        return true;
    }

    function quoteRegisterTokenFee() external view returns (uint256) {
        return 0;
    }

    function registerToken(address token) external payable {
    }

    function quoteSendTokenFee(address token, ParaID destinationChain, uint128 destinationFee)
        external
        view
        returns (uint256) {
            return 0;
        }

    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {

    }
}
