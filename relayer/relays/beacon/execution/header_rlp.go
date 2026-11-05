package execution

// Construction of canonical Ethereum execution header RLP for the Gloas execution proof.
//
// From Gloas the beacon block no longer carries the execution payload, so the relayer cannot
// build the execution commitment from consensus data alone. The commitment is the execution
// block hash, which is by definition the Keccak hash of the header's canonical RLP, so the
// relayer fetches the header from an execution node and re-encodes it.
//
// The header is read as raw JSON rather than through go-ethereum's Header struct on purpose.
// That struct only knows the fields of the release it shipped in, and it drops the rest
// silently: as of go-ethereum v1.15.11 it has neither block_access_list_hash nor the
// slot_number that Gloas adds. A dropped field yields a shorter encoding, a different hash,
// and a proof that fails on chain for no visible reason.

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/big"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/rlp"
)

// MaxHeaderRLPSize mirrors MAX_EXECUTION_HEADER_RLP_SIZE in the ethereum-client pallet.
// Exceeding it is rejected here so the relayer fails where the cause is legible rather than
// at decode on chain.
const MaxHeaderRLPSize = 2048

type fieldKind int

const (
	kindBytes fieldKind = iota
	kindNumber
)

type headerField struct {
	name string
	kind fieldKind
}

// canonicalHeaderFields lists the execution header in RLP order.
//
// The order is consensus, not convention: a wrong order still encodes, and produces a hash
// that silently fails to match. It is checked against a real block in the tests.
var canonicalHeaderFields = []headerField{
	{"parentHash", kindBytes},
	{"sha3Uncles", kindBytes},
	{"miner", kindBytes},
	{"stateRoot", kindBytes},
	{"transactionsRoot", kindBytes},
	{"receiptsRoot", kindBytes},
	{"logsBloom", kindBytes},
	{"difficulty", kindNumber},
	{"number", kindNumber},
	{"gasLimit", kindNumber},
	{"gasUsed", kindNumber},
	{"timestamp", kindNumber},
	{"extraData", kindBytes},
	{"mixHash", kindBytes},
	{"nonce", kindBytes},
	{"baseFeePerGas", kindNumber},
	{"withdrawalsRoot", kindBytes},
	{"blobGasUsed", kindNumber},
	{"excessBlobGas", kindNumber},
	{"parentBeaconBlockRoot", kindBytes},
	{"requestsHash", kindBytes},
	{"blockAccessListHash", kindBytes},
	{"slotNumber", kindNumber},
}

// ErrHeaderHashMismatch means the re-encoded header does not hash to the block hash the
// consensus proof commits to. The usual cause is a fork adding a header field this encoder
// does not know about.
type ErrHeaderHashMismatch struct {
	Want, Got common.Hash
}

func (e ErrHeaderHashMismatch) Error() string {
	return fmt.Sprintf("execution header rlp hashes to %s, expected %s; the header shape has likely changed", e.Got, e.Want)
}

func decodeQuantity(name, s string) (*big.Int, error) {
	v, ok := new(big.Int).SetString(strings.TrimPrefix(s, "0x"), 16)
	if !ok {
		return nil, fmt.Errorf("header field %s: %q is not a quantity", name, s)
	}
	return v, nil
}

// EncodeHeaderRLP encodes a header from its JSON-RPC representation and checks that it hashes
// to blockHash. It returns an error rather than a best effort: the bytes are consensus input.
func EncodeHeaderRLP(raw map[string]string, blockHash common.Hash) ([]byte, error) {
	values := make([]interface{}, 0, len(canonicalHeaderFields))
	for _, f := range canonicalHeaderFields {
		s, ok := raw[f.name]
		if !ok {
			return nil, fmt.Errorf("execution header is missing field %s", f.name)
		}
		switch f.kind {
		case kindNumber:
			n, err := decodeQuantity(f.name, s)
			if err != nil {
				return nil, err
			}
			values = append(values, n)
		default:
			b, err := hexToBytes(f.name, s)
			if err != nil {
				return nil, err
			}
			values = append(values, b)
		}
	}

	encoded, err := rlp.EncodeToBytes(values)
	if err != nil {
		return nil, fmt.Errorf("encode execution header: %w", err)
	}
	if got := common.BytesToHash(crypto.Keccak256(encoded)); got != blockHash {
		return nil, ErrHeaderHashMismatch{Want: blockHash, Got: got}
	}
	if len(encoded) > MaxHeaderRLPSize {
		return nil, fmt.Errorf("execution header rlp is %d bytes, over the %d byte limit the pallet accepts", len(encoded), MaxHeaderRLPSize)
	}
	return encoded, nil
}

func hexToBytes(name, s string) ([]byte, error) {
	s = strings.TrimPrefix(s, "0x")
	if len(s)%2 != 0 {
		s = "0" + s
	}
	b, err := hex.DecodeString(s)
	if err != nil {
		return nil, fmt.Errorf("header field %s: %w", name, err)
	}
	return b, nil
}

// HeaderFetcher reads raw execution headers. It is an interface so the proof path can be
// tested without an execution node.
type HeaderFetcher interface {
	CallContext(ctx context.Context, result interface{}, method string, args ...interface{}) error
}

// FetchHeaderRLP fetches the execution header for blockHash and returns its canonical RLP.
func FetchHeaderRLP(ctx context.Context, client HeaderFetcher, blockHash common.Hash) ([]byte, error) {
	fields, err := FetchHeaderFields(ctx, client, blockHash)
	if err != nil {
		return nil, err
	}
	return EncodeHeaderRLP(fields, blockHash)
}

// FetchHeaderFields returns the scalar JSON-RPC fields of an execution header.
func FetchHeaderFields(ctx context.Context, client HeaderFetcher, blockHash common.Hash) (map[string]string, error) {
	var raw json.RawMessage
	if err := client.CallContext(ctx, &raw, "eth_getBlockByHash", blockHash, false); err != nil {
		return nil, fmt.Errorf("fetch execution block %s: %w", blockHash, err)
	}
	if len(raw) == 0 || string(raw) == "null" {
		return nil, fmt.Errorf("execution node does not have block %s", blockHash)
	}
	return headerFieldsFromJSON(raw)
}

// FetchHeaderSlotNumber returns the Gloas slotNumber header field when present.
func FetchHeaderSlotNumber(ctx context.Context, client HeaderFetcher, blockHash common.Hash) (uint64, bool, error) {
	fields, err := FetchHeaderFields(ctx, client, blockHash)
	if err != nil {
		return 0, false, err
	}
	slotHex, ok := fields["slotNumber"]
	if !ok || slotHex == "" {
		return 0, false, nil
	}
	slot, err := decodeQuantity("slotNumber", slotHex)
	if err != nil {
		return 0, false, err
	}
	if !slot.IsUint64() {
		return 0, false, fmt.Errorf("slotNumber %s does not fit in uint64", slotHex)
	}
	return slot.Uint64(), true, nil
}

// headerFieldsFromJSON keeps only the scalar members. Anything structured, such as the
// withdrawals list, is part of the block body and not of the header.
func headerFieldsFromJSON(raw json.RawMessage) (map[string]string, error) {
	var members map[string]json.RawMessage
	if err := json.Unmarshal(raw, &members); err != nil {
		return nil, fmt.Errorf("decode execution block: %w", err)
	}
	fields := make(map[string]string, len(members))
	for name, value := range members {
		var s string
		if err := json.Unmarshal(value, &s); err == nil {
			fields[name] = s
		}
	}
	return fields, nil
}
