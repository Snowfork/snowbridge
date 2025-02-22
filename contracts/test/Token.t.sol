// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {IERC20} from "../src/interfaces/IERC20.sol";
import {IERC20Permit} from "../src/interfaces/IERC20Permit.sol";
import {Token} from "../src/Token.sol";
import {TokenLib} from "../src/TokenLib.sol";

// See https://mirror.xyz/horsefacts.eth/Jex2YVaO65dda6zEyfM_-DXlXhOWCAoSpOx5PLocYgw for invariant testing

contract TokenTest is Test {
    function setUp() public {}

    function testFuzz_metadata(string memory name, string memory symbol, uint8 decimals) public {
        Token token = new Token(name, symbol, decimals);

        assertEq(token.name(), name);
        assertEq(token.symbol(), symbol);
        assertEq(token.decimals(), decimals);
    }

    function testFuzz_mint(address account, uint256 amount) public {
        vm.assume(account != address(0));

        Token token = new Token("", "", 18);

        token.mint(account, amount);

        assertEq(token.totalSupply(), amount);
        assertEq(token.balanceOf(account), amount);
    }

    function testFuzz_burn(address account, uint256 balance, uint256 burnAmount) public {
        vm.assume(account != address(0));
        vm.assume(balance >= burnAmount);

        Token token = new Token("", "", 18);

        token.mint(account, balance);
        token.burn(account, burnAmount);

        assertEq(token.totalSupply(), balance - burnAmount);
        assertEq(token.balanceOf(account), balance - burnAmount);
    }

    function testFuzz_approve(address account, uint256 amount) public {
        vm.assume(account != address(0));

        Token token = new Token("", "", 18);

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

        Token token = new Token("", "", 18);

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

        Token token = new Token("", "", 18);

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

        Token token = new Token("", "", 18);

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
        Token token = new Token("Test", "TST", 18);

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

    function testPermit() public {
        Token token = new Token("Test Token", "TEST", 18);

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
        Token token = new Token("Test Token", "TEST", 18);

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
        Token token = new Token("Test Token", "TEST", 18);

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

    function testMintAndBurnAfterOwnerChange() public {
        Token token = new Token("Test Token", "TEST", 18);

        // Setup accounts
        address newOwner = makeAddr("newOwner");
        address user = makeAddr("user");

        // Initial owner (address(this)) can mint and burn
        token.mint(user, 1000);
        assertEq(token.balanceOf(user), 1000);

        token.burn(user, 500);
        assertEq(token.balanceOf(user), 500);

        // Change owner
        token.setOwner(newOwner);
        assertEq(token.owner(), newOwner);

        // Original owner should no longer be able to mint or burn
        vm.expectRevert(Token.Unauthorized.selector);
        token.mint(user, 1000);

        vm.expectRevert(Token.Unauthorized.selector);
        token.burn(user, 100);

        // New owner should be able to mint and burn
        vm.prank(newOwner);
        token.mint(user, 1000);
        assertEq(token.balanceOf(user), 1500);

        vm.prank(newOwner);
        token.burn(user, 500);
        assertEq(token.balanceOf(user), 1000);

        // Random user should not be able to mint or burn
        vm.prank(makeAddr("random"));
        vm.expectRevert(Token.Unauthorized.selector);
        token.mint(user, 1000);

        vm.prank(makeAddr("random"));
        vm.expectRevert(Token.Unauthorized.selector);
        token.burn(user, 100);
    }

    function testSetOwnerOnlyOwner() public {
        Token token = new Token("Test Token", "TEST", 18);
        address newOwner = makeAddr("newOwner");

        // Random address cannot set owner
        vm.prank(makeAddr("random"));
        vm.expectRevert(Token.Unauthorized.selector);
        token.setOwner(newOwner);

        // Current owner can set new owner
        token.setOwner(newOwner);
        assertEq(token.owner(), newOwner);

        // Original owner can no longer set new owner
        vm.expectRevert(Token.Unauthorized.selector);
        token.setOwner(address(this));

        // New owner can set another owner
        vm.prank(newOwner);
        token.setOwner(address(1234));
        assertEq(token.owner(), address(1234));
    }
}
