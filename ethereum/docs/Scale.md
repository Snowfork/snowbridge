## `Scale`
Scale implements decoding of SCALE encoded compact uints; not currently used, included to support future work on generalized data relay

### `decodeUint256(bytes data) → uint256` (public)
Decodes a SCALE encoded uint256

### `decodeUintCompact(bytes data) → uint256` (public)
Decodes a SCALE encoded compact uint

### `readByteAtIndex(bytes data, uint8 index) → uint8` (internal)
Reads a byte from an array of bytes, required for compact uint decoding
