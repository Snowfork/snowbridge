package state

import (
	"encoding/hex"
	"errors"
	"testing"
)

// The 2 attestations of Plataberget block 115968. Their progressive-list root is one of
// the 13 field roots that compose that block's body root, so reproducing it validates the
// ProgressiveBitlist primitive, the Attestation progressive container and the list shape
// against data a live node signed.
func attestationsSlot115968() []*AttestationGloas {
	return []*AttestationGloas{
		{
			AggregationBits: hexBytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffffffffffffffffffffffffffffffffffffffffffbfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdffffffffffffffffffffffffffffffffffffff03"),
			CommitteeBits:   hexBytes("ffff0f0000000000"),
			Signature:       hexBytes("8e064d39be366b94a7722c70428853295ad998ad29e2323ab5ee11faf9e90267bd64c3aa6987b4c6e5e273c60d3dac200ca5f44c515360f4f28e6386fb1575868efc7e8bd6bffc6ff28dc511de3bc25c583dd128ff3c5386dfceabafb8e0fd8e"),
			Data: &AttestationDataGloas{
				Slot:            115967,
				Index:           0,
				BeaconBlockRoot: hexBytes("84a3016f712471145043f63fe36f9c9dbf7c16b3a709ce4a2b22678b77acbc80"),
				Source:          &Checkpoint{Epoch: 3622, Root: hexBytes("c6556df57af68c8f9e6009f3500f685919a7746dbb718e2ce5055013d967ec59")},
				Target:          &Checkpoint{Epoch: 3623, Root: hexBytes("18d5382c90919ba10e456b428c38db932453b18d6567aea05770a571a99cce6a")},
			},
		},
		{
			AggregationBits: hexBytes("0000000000000000000000000008000020"),
			CommitteeBits:   hexBytes("0000010000000000"),
			Signature:       hexBytes("b0fc6e23e5e321d142325759711844fffab770ba3a851ac7d5374765f5775d90dddaa9b668f13060cc3536f48ce6bacf17b87d32625f4827271503a0b2b6f280b1c79ad660b779a5acfd97cdd3aaf5a89af81812970e4d6226d02d8db09bb22b"),
			Data: &AttestationDataGloas{
				Slot:            115953,
				Index:           0,
				BeaconBlockRoot: hexBytes("4d541d27e535531d35c2ae75f4f0fb5e8b3475797a0327808f8ff488606a622e"),
				Source:          &Checkpoint{Epoch: 3622, Root: hexBytes("c6556df57af68c8f9e6009f3500f685919a7746dbb718e2ce5055013d967ec59")},
				Target:          &Checkpoint{Epoch: 3623, Root: hexBytes("4d541d27e535531d35c2ae75f4f0fb5e8b3475797a0327808f8ff488606a622e")},
			},
		},
	}
}

const knownAttestationsRoot = "d8b5d6075c9a531cf80f112d078df95249a260eef7b143558ab13c963979865e"

func TestGloasAttestationsRootMatchesRealBlock(t *testing.T) {
	root, err := AttestationsRoot(attestationsSlot115968())
	if err != nil {
		t.Fatalf("attestations root: %v", err)
	}
	if hex.EncodeToString(root[:]) != knownAttestationsRoot {
		t.Fatalf("attestations root mismatch\n got  %x\n want %s", root, knownAttestationsRoot)
	}
}

// The 2 payload attestations of the same block. Validates the PayloadAttestation
// progressive container and its Bitvector aggregation bits, which are a different shape from
// the attestation bitlist above.
func payloadAttestationsSlot115968() []*PayloadAttestationGloas {
	return []*PayloadAttestationGloas{
		{
			AggregationBits: hexBytes("80040010000000000000090000010c00000000000000880002000200000000002040800800001008004000800000080200800000400100020000400010080000"),
			Signature:       hexBytes("8b124ba6fbd7985b3c39a929966dffe0788dedb0a1a49cfa811c2a0b67e04e3728ce9bd88c45e89659bfb7f1d092d0d1166089f75dc6aea261daa2c4e11703630e8dfc0d14e4ecfbe57c233da48259802839ee55f6675e153cf73dc6b13bb3b8"),
			Data: &PayloadAttestationDataGloas{
				BeaconBlockRoot:   hexBytes("84a3016f712471145043f63fe36f9c9dbf7c16b3a709ce4a2b22678b77acbc80"),
				Slot:              115967,
				PayloadPresent:    true,
				BlobDataAvailable: false,
			},
		},
		{
			AggregationBits: hexBytes("7ffbffeffffffffffffff6fffffef3ffffffffffffff77fffdfffdffffffffffdfbf7ff7ffffeff7ffbfff7ffffff7fdff7fffffbffefffdffffbfffeff7ffff"),
			Signature:       hexBytes("b8d8f6fcf0641be957bdcecb40aaa0ec48246bddff87c51e3fbd7142ee4fc122e0fbc71737fd7fc63b7221b8cf3f944511f2b88fd9e035e63137e2f15d60fa9c0804aa53eb514703295ac6771c4313571061f59abfd64ee54a7da7588b43b1d3"),
			Data: &PayloadAttestationDataGloas{
				BeaconBlockRoot:   hexBytes("84a3016f712471145043f63fe36f9c9dbf7c16b3a709ce4a2b22678b77acbc80"),
				Slot:              115967,
				PayloadPresent:    true,
				BlobDataAvailable: true,
			},
		},
	}
}

const knownPayloadAttestationsRoot = "30deed299e2c1cafa6a2c00e26b22cb2d9b431cd17e31e6a833812f21bfcb52a"

func TestGloasPayloadAttestationsRootMatchesRealBlock(t *testing.T) {
	root, err := PayloadAttestationsRoot(payloadAttestationsSlot115968())
	if err != nil {
		t.Fatalf("payload attestations root: %v", err)
	}
	if hex.EncodeToString(root[:]) != knownPayloadAttestationsRoot {
		t.Fatalf("mismatch\n got  %x\n want %s", root, knownPayloadAttestationsRoot)
	}
}

// Every empty list in a body merkleizes to the same root regardless of element type, which is
// why a green body-root test says nothing about the lists that happened to be empty.
func TestEmptyProgressiveListRootIsElementAgnostic(t *testing.T) {
	empty := EmptyProgressiveListRoot()
	a, err := AttestationsRoot(nil)
	if err != nil {
		t.Fatal(err)
	}
	p, err := PayloadAttestationsRoot(nil)
	if err != nil {
		t.Fatal(err)
	}
	if a != empty || p != empty {
		t.Fatal("empty lists of different element types should share a root")
	}
}

// parent_execution_requests of block 115968: five empty lists. Matches field root 12 of that
// block's body, so the empty case of the container is validated even though its element types
// are not.
func TestGloasExecutionRequestsEmptyRootMatchesRealBlock(t *testing.T) {
	root, err := (&ExecutionRequestsGloas{}).HashTreeRoot()
	if err != nil {
		t.Fatalf("execution requests root: %v", err)
	}
	const want = "87b69a306c8e430d0857f7c4ac5e27cecffa1108d43c2e5df7388056fea7a423"
	if hex.EncodeToString(root[:]) != want {
		t.Fatalf("mismatch\n got  %x\n want %s", root, want)
	}
}

// A non-empty request list must fail loudly rather than produce a root nothing validated.
func TestGloasExecutionRequestsRefusesUnsupported(t *testing.T) {
	_, err := (&ExecutionRequestsGloas{Deposits: 1}).HashTreeRoot()
	var unsupported ErrUnsupportedBodyList
	if !errors.As(err, &unsupported) {
		t.Fatalf("expected ErrUnsupportedBodyList, got %v", err)
	}
}

// The sync aggregate of block 115968, matching field root 8.
func TestGloasSyncAggregateRootMatchesRealBlock(t *testing.T) {
	root := SyncAggregateRoot(
		hexBytes("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffbfffffffffffffffffffffffff"),
		hexBytes("878bad4793e3e804765ba3ed094d671ca1debfdaa2877afc8349427d05e0f1ffab526d393bc868211f9160886a430c3c0580879e3be37ed11f9c44187a2368e85ce19f5a6a962afb93bd7941e719d1e616bd1d1f7d27df71ff7590df9cf3100f"),
	)
	const want = "54bb8f69222b00d7081d812919207bc3622184c41c513047afbbcf9214518b94"
	if hex.EncodeToString(root[:]) != want {
		t.Fatalf("mismatch\n got  %x\n want %s", root, want)
	}
}
