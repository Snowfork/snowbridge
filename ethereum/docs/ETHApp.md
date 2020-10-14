## `ETHApp`
ETHApp is an application for cross-chain ETH transfers between Ethereum and Substrate

### `register(address _bridge)` (public)
Registers the Bridge contract on the application

### `sendETH(bytes32 _recipient)` (public)
Input method for users transferring funds to Substrate

### `handle(bytes _data)` (public)
Handles a SCALE encoded message from the Bridge

### `unlockETH(address payable _recipient, uint256 _amount)` (internal)
Unlocks and transfers Ethereum as the result of a successful data decoding from Substrate

### `AppTransfer(address _sender, bytes32 _recipient, uint256 _amount)`
Event the Relayer subscribes to which denotes that Ethereum has been sent to the Bridge

### `Unlock(bytes _sender, address _recipient, uint256 _amount)`
Event that indicates that Ethereum has been unlocked as the result of a successful transfer from Substrate
