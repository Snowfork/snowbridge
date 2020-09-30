## `ERC20App`
ERC20App is an application for cross-chain ERC20 transfers between Ethereum and Substrate

### `register(address _bridge)` (public)
Registers the Bridge contract on the application

### `sendERC20(bytes32 _recipient, address _tokenAddr, uint256 _amount)` (public)
Input method for users transferring tokens to Substrate

### `handle(bytes _data)` (public)
Handles a SCALE encoded message from the Bridge

### `sendTokens(address _recipient, address _token, uint256 _amount)` (internal)
Unlocks and transfers ERC20 tokens as the result of a successful data decoding from Substrate

### `AppTransfer(address _sender, bytes32 _recipient, address _token, uint256 _amount)`
Event the Relayer subscribes to which denotes that ERC20 has been sent to the Bridge

### `Unlock(bytes _sender, address _recipient, address _token, uint256 _amount)`
Event that indicates that ERC20 has been unlocked as the result of a successful transfer from Substrate
