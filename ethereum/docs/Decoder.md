## `Decoder`
Decoder is a library for decoding SCALE encoded data

### `decodeUint256(bytes data) → uint256` (public)
Decodes a SCALE encoded uint256

### `slice(bytes _bytes, uint256 _start, uint256 _length) → bytes` (internal)
Utility function to slice a segment of bytes from a byte array

### `sliceAddress(bytes _bytes, uint256 _start) → address payable` (internal)
Utility slicing function for slicing an address from an array of bytes
