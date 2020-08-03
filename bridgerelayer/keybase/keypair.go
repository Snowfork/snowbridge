package keybase

type KeyType = string

const SubstrateKeyType KeyType = "sr25519"
const EthereumKeyType KeyType = "secp256k1"

type Keypair interface {
	// Encode is used to write the key to a file
	Encode() []byte
	// Decode is used to retrieve a key from a file
	Decode([]byte) error
	// Address provides the address for the keypair
	Address() string
	// PublicKey returns the keypair's public key an encoded a string
	PublicKey() string
}
