import { ethers, expect, loadFixture, deployMockContract } from "../setup"
import {
    ChannelRegistry__factory,
    ScaleCodec__factory,
    OutboundChannel__factory,
    ETHApp__factory,
    ERC20App__factory,
    DOTApp__factory,
    WrappedToken__factory,
    TestToken__factory,
} from "../../src"

export { ethAppFixture, erc20AppFixture, dotAppFixture }

// Sets up a fixture for app dependencies (ChannelRegistry, channels, and so on)
async function baseFixture() {
    let [owner, user] = await ethers.getSigners()

    let codec = await new ScaleCodec__factory(owner).deploy()
    await codec.deployed()

    let MockOutboundChannel = await ethers.getContractFactory("MockOutboundChannel")
    let outboundChannel = await MockOutboundChannel.deploy()

    // mock outbound channel
    let mockOutboundChannel = await deployMockContract(owner as any, OutboundChannel__factory.abi)
    await mockOutboundChannel.mock.submit.returns()

    let registry = await new ChannelRegistry__factory(owner).deploy()
    await registry.deployed()

    // Add mock inbound and outbound channels to registry
    await registry.updateChannel(0, owner.address, outboundChannel.address)

    return {
        registry,
        mockOutboundChannel,
        codec,
        owner,
        user,
    }
}

async function ethAppFixture() {
    let { registry, codec, owner, user } = await loadFixture(baseFixture)

    let app = await new ETHApp__factory(
        {
            "contracts/ScaleCodec.sol:ScaleCodec": codec.address,
        },
        owner
    ).deploy(owner.address, registry.address)
    await app.deployed()

    return {
        app,
        owner,
        user,
        channelID: 0,
    }
}

async function erc20AppFixture() {
    let { registry, codec, owner, user } = await loadFixture(baseFixture)

    let app = await new ERC20App__factory(
        {
            "contracts/ScaleCodec.sol:ScaleCodec": codec.address,
        },
        owner
    ).deploy(registry.address)
    await app.deployed()

    let token = await new TestToken__factory(owner).deploy("Test Token", "TEST")
    await token.deployed()

    await token.mint(user.address, 100)
    await token.connect(user).approve(app.address, 100)

    return {
        app,
        token,
        owner,
        user,
        channelID: 0,
    }
}

async function dotAppFixture() {
    let { registry, mockOutboundChannel, codec, owner, user } = await loadFixture(baseFixture)

    let token = await new WrappedToken__factory(owner).deploy("Wrapped DOT", "WDOT")
    await token.deployed()

    let app = await new DOTApp__factory(
        {
            "contracts/ScaleCodec.sol:ScaleCodec": codec.address,
        },
        owner
    ).deploy(token.address, mockOutboundChannel.address, registry.address)
    await app.deployed()

    await token.transferOwnership(app.address)

    return {
        app,
        token,
        owner,
        user,
        channelID: 0,
    }
}
