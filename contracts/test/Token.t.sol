// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {IERC20} from "../src/interfaces/IERC20.sol";
import {IERC20Permit} from "../src/interfaces/IERC20Permit.sol";
import {Token} from "../src/Token.sol";
import {TokenLib} from "../src/TokenLib.sol";

contract TokenTest is Test {

    string public tokenName = "Test Token";
    Token public token;

    function setUp() public {
        token = new Token(tokenName, "TEST", 18);
    }

    function testFuzz_mint(address account, uint256 amount) public {
        vm.assume(account != address(0));

        token.mint(account, amount);

        assertEq(token.totalSupply(), amount);
        assertEq(token.balanceOf(account), amount);
    }

    function testFuzz_burn(address account, uint256 balance, uint256 burnAmount) public {
        vm.assume(account != address(0));
        vm.assume(balance >= burnAmount);

        token.mint(account, balance);
        token.burn(account, burnAmount);

        assertEq(token.totalSupply(), balance - burnAmount);
        assertEq(token.balanceOf(account), balance - burnAmount);
    }

    function testFuzz_approve(address account, uint256 amount) public {
        vm.assume(account != address(0));

        assertTrue(token.approve(account, amount));
        assertEq(token.allowance(address(this), account), amount);
    }

    function testFuzz_transferCorrectlyUpdatesBalances(
        address sender,
        address receiver,
        uint256 mintAmount,
        uint256 transferAmount
    ) public {
        vm.assume(sender != address(0) && receiver != address(0) && sender != receiver);
        vm.assume(mintAmount > 0 && transferAmount > 0 && mintAmount >= transferAmount);

        token.mint(sender, mintAmount);

        uint256 initialSenderBalance = token.balanceOf(sender);
        uint256 initialReceiverBalance = token.balanceOf(receiver);
        uint256 initialTotalSupply = token.totalSupply();

        vm.prank(sender);
        token.transfer(receiver, transferAmount);

        assertEq(token.balanceOf(sender), initialSenderBalance - transferAmount);
        assertEq(token.balanceOf(receiver), initialReceiverBalance + transferAmount);
        assertEq(token.totalSupply(), initialTotalSupply);
    }

    function testFuzz_transferFrom(
        address owner,
        address spender,
        address receiver,
        uint256 mintAmount,
        uint256 allowanceAmount,
        uint256 transferAmount
    ) public {
        // Avoid zero address and ensure distinct addresses
        vm.assume(owner != address(0) && spender != address(0) && receiver != address(0));
        vm.assume(owner != spender && spender != receiver && owner != receiver);

        // Ensure amounts are valid
        vm.assume(mintAmount > 0 && allowanceAmount > 0 && transferAmount > 0);
        vm.assume(mintAmount >= transferAmount);
        vm.assume(allowanceAmount >= transferAmount);
        vm.assume(allowanceAmount < type(uint256).max);

        // Mint tokens to owner
        token.mint(owner, mintAmount);

        // Set allowance for spender
        vm.prank(owner);
        token.approve(spender, allowanceAmount);

        uint256 initialOwnerBalance = token.balanceOf(owner);
        uint256 initialReceiverBalance = token.balanceOf(receiver);
        uint256 initialAllowance = token.allowance(owner, spender);
        uint256 initialTotalSupply = token.totalSupply();

        // Perform transferFrom as spender
        vm.prank(spender);
        token.transferFrom(owner, receiver, transferAmount);

        // Verify balances are updated correctly
        assertEq(token.balanceOf(owner), initialOwnerBalance - transferAmount);
        assertEq(token.balanceOf(receiver), initialReceiverBalance + transferAmount);

        // Verify allowance is reduced
        assertEq(token.allowance(owner, spender), initialAllowance - transferAmount);

        // Verify total supply remains unchanged
        assertEq(token.totalSupply(), initialTotalSupply);
    }

    // This additional test specifically verifies the behavior when
    // using maximum allowance (`type(uint256).max`), which is a special case where the allowance
    // should not be reduced after the transfer.
    function testFuzz_transferFromWithMaxAllowance(
        address owner,
        address spender,
        address receiver,
        uint256 mintAmount,
        uint256 transferAmount
    ) public {
        vm.assume(owner != address(0) && spender != address(0) && receiver != address(0));
        vm.assume(owner != spender && spender != receiver && owner != receiver);
        vm.assume(mintAmount > 0 && transferAmount > 0);
        vm.assume(mintAmount >= transferAmount);

        token.mint(owner, mintAmount);

        // Approve maximum allowance
        vm.prank(owner);
        token.approve(spender, type(uint256).max);

        uint256 initialOwnerBalance = token.balanceOf(owner);
        uint256 initialReceiverBalance = token.balanceOf(receiver);

        vm.prank(spender);
        token.transferFrom(owner, receiver, transferAmount);

        // Verify balances
        assertEq(token.balanceOf(owner), initialOwnerBalance - transferAmount);
        assertEq(token.balanceOf(receiver), initialReceiverBalance + transferAmount);

        // Verify allowance remains unchanged for max approval
        assertEq(token.allowance(owner, spender), type(uint256).max);
    }

    function test_transferFromToZeroAddressReverts() public {
        address owner = makeAddr("owner");
        address spender = makeAddr("spender");
        uint256 amount = 100;

        // Mint tokens to owner
        token.mint(owner, amount);

        // Approve spender
        vm.prank(owner);
        token.approve(spender, amount);

        // Attempt transfer to zero address should revert
        vm.prank(spender);
        vm.expectRevert(abi.encodeWithSelector(IERC20.InvalidReceiver.selector, address(0)));
        token.transferFrom(owner, address(0), amount);
    }

    function test_transferFromFailsWithInsufficientAllowance() public {
        // Setup test parameters
        address owner = makeAddr("owner");
        address spender = makeAddr("spender");
        address receiver = makeAddr("receiver");

        uint256 mintAmount = 1000;
        uint256 allowanceAmount = 500;
        uint256 transferAmount = 600; // Greater than the allowance

        // Mint tokens to owner
        token.mint(owner, mintAmount);

        // Set allowance less than the transfer amount
        vm.prank(owner);
        token.approve(spender, allowanceAmount);

        // Verify initial state
        assertEq(token.balanceOf(owner), mintAmount);
        assertEq(token.allowance(owner, spender), allowanceAmount);

        // Attempt to transfer more than allowed - should revert
        vm.prank(spender);
        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20.InsufficientAllowance.selector,
                spender,
                allowanceAmount,
                transferAmount
            )
        );
        token.transferFrom(owner, receiver, transferAmount);

        // Verify balances remain unchanged
        assertEq(token.balanceOf(owner), mintAmount);
        assertEq(token.balanceOf(receiver), 0);
        assertEq(token.allowance(owner, spender), allowanceAmount);

        // Now try with exactly the allowance amount - should work
        vm.prank(spender);
        token.transferFrom(owner, receiver, allowanceAmount);

        // Verify the successful transfer
        assertEq(token.balanceOf(owner), mintAmount - allowanceAmount);
        assertEq(token.balanceOf(receiver), allowanceAmount);
        assertEq(token.allowance(owner, spender), 0);
    }

    function test_transferFromExactAllowance() public {
        // Setup test parameters
        address owner = makeAddr("owner");
        address spender = makeAddr("spender");
        address receiver = makeAddr("receiver");

        uint256 mintAmount = 1000;
        uint256 allowanceAmount = 500;

        // Mint tokens to owner
        token.mint(owner, mintAmount);

        // Set allowance less than the transfer amount
        vm.prank(owner);
        token.approve(spender, allowanceAmount);

        vm.prank(spender);
        token.transferFrom(owner, receiver, allowanceAmount);

        // Verify the successful transfer
        assertEq(token.balanceOf(owner), mintAmount - allowanceAmount);
        assertEq(token.balanceOf(receiver), allowanceAmount);
        assertEq(token.allowance(owner, spender), 0);
    }

    function testPermit() public {
        // Setup accounts
        uint256 ownerPrivateKey = 0x1234;
        address owner = vm.addr(ownerPrivateKey);
        address spender = makeAddr("spender");

        // Permit parameters
        uint256 value = 1000;
        uint256 deadline = block.timestamp + 1 hours;

        // Calculate permit digest
        bytes32 PERMIT_TYPEHASH =
            keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

        bytes32 structHash =
            keccak256(abi.encode(PERMIT_TYPEHASH, owner, spender, value, token.nonces(owner), deadline));

        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", token.DOMAIN_SEPARATOR(), structHash));

        // Sign the digest
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, digest);

        // Initial checks
        assertEq(token.allowance(owner, spender), 0);
        assertEq(token.nonces(owner), 0);

        // Execute permit
        token.permit(owner, spender, value, deadline, v, r, s);

        // Verify results
        assertEq(token.allowance(owner, spender), value);
        assertEq(token.nonces(owner), 1);
    }

    function testPermitExpired() public {
        // Setup accounts
        uint256 ownerPrivateKey = 0x1234;
        address owner = vm.addr(ownerPrivateKey);
        address spender = makeAddr("spender");

        // Permit parameters with expired deadline
        uint256 value = 1000;
        uint256 deadline = block.timestamp - 1;

        // Calculate permit digest
        bytes32 PERMIT_TYPEHASH =
            keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

        bytes32 structHash =
            keccak256(abi.encode(PERMIT_TYPEHASH, owner, spender, value, token.nonces(owner), deadline));

        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", token.DOMAIN_SEPARATOR(), structHash));

        // Sign the digest
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, digest);

        // Expect revert due to expired deadline
        vm.expectRevert(IERC20Permit.PermitExpired.selector);
        token.permit(owner, spender, value, deadline, v, r, s);
    }

    function testPermitInvalidSignature() public {
        // Setup accounts
        uint256 ownerPrivateKey = 0x1234;
        uint256 wrongPrivateKey = 0x5678;
        address owner = vm.addr(ownerPrivateKey);
        address spender = makeAddr("spender");

        // Permit parameters
        uint256 value = 1000;
        uint256 deadline = block.timestamp + 1 hours;

        // Calculate permit digest
        bytes32 PERMIT_TYPEHASH =
            keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

        bytes32 structHash =
            keccak256(abi.encode(PERMIT_TYPEHASH, owner, spender, value, token.nonces(owner), deadline));

        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", token.DOMAIN_SEPARATOR(), structHash));

        // Sign with wrong private key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(wrongPrivateKey, digest);

        // Expect revert due to invalid signature
        vm.expectRevert(IERC20Permit.InvalidSignature.selector);
        token.permit(owner, spender, value, deadline, v, r, s);
    }

    function testDomainSeparator() public {
        // Manually calculate the expected domain separator
        bytes32 expectedDomainSeparator = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(tokenName)),
                keccak256(bytes("1")),
                block.chainid,
                address(token)
            )
        );

        // Get the domain separator from the contract
        bytes32 actualDomainSeparator = token.DOMAIN_SEPARATOR();

        // Verify that they match
        assertEq(actualDomainSeparator, expectedDomainSeparator);

        // Also test that domain separator changes when chain ID changes
        uint256 originalChainId = block.chainid;
        vm.chainId(originalChainId + 1);

        bytes32 newExpectedDomainSeparator = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(tokenName)),
                keccak256(bytes("1")),
                block.chainid,
                address(token)
            )
        );

        bytes32 newActualDomainSeparator = token.DOMAIN_SEPARATOR();

        // The domain separator should be different on a different chain
        assertTrue(newActualDomainSeparator != actualDomainSeparator);
        // And it should match our new expected value
        assertEq(newActualDomainSeparator, newExpectedDomainSeparator);
    }

    function test_onlyGatewayCanMintAndBurn() public {
        // Setup
        address gatewayAddress = address(this);
        address regularUser = makeAddr("regularUser");
        address tokenReceiver = makeAddr("tokenReceiver");
        uint256 amount = 1000;

        // Verify that gateway (this contract) can mint
        token.mint(tokenReceiver, amount);
        assertEq(token.balanceOf(tokenReceiver), amount);

        // Verify that gateway can burn
        token.burn(tokenReceiver, amount / 2);
        assertEq(token.balanceOf(tokenReceiver), amount / 2);

        // Try to mint as a non-gateway address
        vm.prank(regularUser);
        vm.expectRevert(abi.encodeWithSelector(Token.Unauthorized.selector));
        token.mint(tokenReceiver, amount);

        // Verify balance hasn't changed after failed mint
        assertEq(token.balanceOf(tokenReceiver), amount / 2);

        // Try to burn as a non-gateway address
        vm.prank(regularUser);
        vm.expectRevert(abi.encodeWithSelector(Token.Unauthorized.selector));
        token.burn(tokenReceiver, amount / 4);

        // Verify balance hasn't changed after failed burn
        assertEq(token.balanceOf(tokenReceiver), amount / 2);

        // Confirm gateway field is correctly set
        assertEq(token.gateway(), gatewayAddress);

        // Final mint/burn with gateway to confirm functionality still works
        token.mint(tokenReceiver, amount);
        assertEq(token.balanceOf(tokenReceiver), amount + (amount / 2));

        token.burn(tokenReceiver, amount);
        assertEq(token.balanceOf(tokenReceiver), amount / 2);
    }
}
