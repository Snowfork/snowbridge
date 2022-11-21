import { ethers, loadFixture, deployMockContract } from "../setup"
import {
    ChannelRegistry__factory,
    OutboundChannel__factory,
    ETHApp__factory,
    EtherVault__factory,
    ERC20App__factory,
    ERC20Vault__factory,
    DOTApp__factory,
    WrappedToken__factory,
    TestToken__factory,
} from "../../src"

export { ethAppFixture, erc20AppFixture, dotAppFixture }

// Sets up a fixture for app dependencies (ChannelRegistry, channels, and so on)
async function baseFixture() {
    let [owner, user] = await ethers.getSigners()

    // mock outbound channel
    let mockOutboundChannel = await deployMockContract(owner, OutboundChannel__factory.abi)
    await mockOutboundChannel.mock.submit.returns()

    let registry = await new ChannelRegistry__factory(owner).deploy()
    await registry.deployed()

    // Add mock inbound and outbound channels to registry
    await registry.updateChannel(0, owner.address, mockOutboundChannel.address)

    return {
        registry,
        mockOutboundChannel,
        owner,
        user,
    }
}

async function ethAppFixture() {
    let { registry, owner, user } = await loadFixture(baseFixture)

    let vault = await new EtherVault__factory(owner).deploy()
    await vault.deployed()

    let app = await new ETHApp__factory(
        owner
    ).deploy(owner.address, vault.address, registry.address)
    await app.deployed()

    await vault.transferOwnership(app.address)

    return {
        app,
        vault,
        owner,
        user,
        channelID: 0,
    }
}

async function erc20AppFixture() {
    let { registry, owner, user } = await loadFixture(baseFixture)

    let vault = await new ERC20Vault__factory(owner).deploy()
    await vault.deployed()

    let app = await new ERC20App__factory(
        owner
    ).deploy(vault.address, registry.address)
    await app.deployed()

    await vault.transferOwnership(app.address)

    let token = await new TestToken__factory(owner).deploy("Test Token", "TEST")
    await token.deployed()

    await token.mint(user.address, 100)
    await token.connect(user).approve(vault.address, 100)

    return {
        app,
        vault,
        token,
        owner,
        user,
        channelID: 0,
    }
}

async function dotAppFixture() {
    let { registry, mockOutboundChannel, owner, user } = await loadFixture(baseFixture)

    let token = await new WrappedToken__factory(owner).deploy("Wrapped DOT", "WDOT")
    await token.deployed()

    let app = await new DOTApp__factory(
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
