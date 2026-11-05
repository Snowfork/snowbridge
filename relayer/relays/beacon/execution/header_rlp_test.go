package execution

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"os"
	"testing"

	"github.com/ethereum/go-ethereum/common"
)

// The execution block committed to by slot 115968 of the Platåberget Gloas testnet, which is
// the block the beacon body test proves. Re-encoding it must reproduce its own hash, which is
// what makes the encoder's field list and ordering verifiable rather than assumed.
const gloasBlockHash = "0x9af0c93804597fe505c22f3b15abb47acb01f7c6106474b196b52f592fae0e39"

func loadHeaderFixture(t *testing.T) map[string]string {
	t.Helper()
	raw, err := os.ReadFile("testdata/header_gloas_113202.json")
	if err != nil {
		t.Fatalf("read fixture: %v", err)
	}
	var fields map[string]string
	if err := json.Unmarshal(raw, &fields); err != nil {
		t.Fatalf("unmarshal fixture: %v", err)
	}
	return fields
}

func TestEncodeHeaderRLPReproducesRealBlockHash(t *testing.T) {
	encoded, err := EncodeHeaderRLP(loadHeaderFixture(t), common.HexToHash(gloasBlockHash))
	if err != nil {
		t.Fatalf("encode header: %v", err)
	}
	t.Logf("encoded %d bytes: %s", len(encoded), hex.EncodeToString(encoded[:16]))
	if len(encoded) > MaxHeaderRLPSize {
		t.Fatalf("encoding is %d bytes, over the pallet limit of %d", len(encoded), MaxHeaderRLPSize)
	}
}

// A header shape this encoder does not know must be rejected, not encoded short.
func TestEncodeHeaderRLPRejectsUnknownShape(t *testing.T) {
	fields := loadHeaderFixture(t)
	delete(fields, "slotNumber")
	_, err := EncodeHeaderRLP(fields, common.HexToHash(gloasBlockHash))
	if err == nil {
		t.Fatal("expected an error for a header missing a field")
	}
}

// A tampered field must fail the hash check rather than pass through.
func TestEncodeHeaderRLPRejectsTamperedField(t *testing.T) {
	fields := loadHeaderFixture(t)
	fields["receiptsRoot"] = "0x" + hex.EncodeToString(make([]byte, 32))
	_, err := EncodeHeaderRLP(fields, common.HexToHash(gloasBlockHash))
	var mismatch ErrHeaderHashMismatch
	if !errors.As(err, &mismatch) {
		t.Fatalf("expected ErrHeaderHashMismatch, got %v", err)
	}
}
