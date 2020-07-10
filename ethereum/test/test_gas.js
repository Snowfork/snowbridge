const Bank = artifacts.require('Bank');
const TestToken = artifacts.require('TestToken');

const BigNumber = web3.BigNumber;

require('chai')
  .use(require('chai-as-promised'))
  .use(require('chai-bignumber')(BigNumber))
  .should();

contract('Gas expenditures', function (accounts) {
    // Accounts
    const owner = accounts[0];
    const userOne = accounts[1];

    // Constants
    const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"

    describe('Gas costs', function(){

        beforeEach(async function () {
            this.bank = await Bank.new();
        });

        it('sendETH gas usage', async function () {
            // Prepare transaction parameters
            const targetAppID = web3.utils.utf8ToHex("targetapp123");
            const recipient = web3.utils.utf8ToHex(POLKADOT_ADDRESS);
            const weiAmount = web3.utils.toWei("0.25", "ether");

            // Deposit Ethereum to the contract
            const result = await this.bank.sendETH(
                targetAppID,
                recipient,
                {from: userOne, value: weiAmount}
            ).should.be.fulfilled;

            console.log('\tsendETH gas: ' + result.receipt.gasUsed);
        });

        // Set up an ERC20 token for testing token deposits
        before(async function () {
            this.symbol = "TEST";
            this.token = await TestToken.new(100000, "Test Token", this.symbol);
    
            // Load user account with 'TEST' ERC20 tokens
            await this.token.transfer(userOne, 1000, {
            from: owner
            }).should.be.fulfilled;
        });

        it('sendERC20 gas usage', async function () {
            // Prepare transaction parameters
            const targetAppID = web3.utils.utf8ToHex("targetapp123");
            const recipient = web3.utils.utf8ToHex(POLKADOT_ADDRESS);
            const amount = 100;

            // Approve tokens to contract
            await this.token.approve(this.bank.address, amount, {
                from: userOne
            }).should.be.fulfilled;

            // Deposit ERC20 tokens to the contract
            const result = await this.bank.sendERC20(
                targetAppID,
                recipient,
                this.token.address,
                amount,
                {
                    from: userOne,
                    value: 0
                }
            ).should.be.fulfilled;
        
            console.log('\tsendERC20 gas: ' + result.receipt.gasUsed);
        }); 
    });
});
