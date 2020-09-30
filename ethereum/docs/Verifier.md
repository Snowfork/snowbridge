## `Verifier`
Verifier verifies tx origin and signatures

### `constructor(address _operator)` (public)
Constructor sets the operator's address

### `verifyOperator() → bool` (public)
Verifies the operator as the original tx sender

### `verifyBytes(bytes _rawMessage, bytes _signature) → bool` (public)
Recreates the hashed prefixed message signed on the client from raw message bytes

### `prefixed(bytes32 _hashedMessage) → bytes32` (internal)
Builds a prefixed hash to mimic the behavior of eth_sign

### `verify(bytes32 _hash, bytes _signature) → bool` (public)
Verify if a hashed message was signed by the contract's operator

### `recover(bytes32 _hash, bytes _signature) → address` (public)
Recover signer address from a message using their signature
