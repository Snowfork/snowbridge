package parachainV3

import (
	"testing"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts"
)

// applyRetention models `hops` EIP-150 63/64 gas retentions (floor at each hop),
// mirroring what the reserved gas actually survives on-chain before the Gateway's
// gasleft()*63/64 dispatch check.
func applyRetention(gas uint64, hops int) uint64 {
	for i := 0; i < hops; i++ {
		gas = gas * 63 / 64
	}
	return gas
}

func TestGasWithForwardingReserveSurvivesRetention(t *testing.T) {
	for _, gas := range []uint64{1, 10_000, 24_000, 100_000, 500_000, 1_000_000, 5_000_000} {
		for hops := 1; hops <= 3; hops++ {
			reserved := gasWithForwardingReserve(gas, hops)
			if got := applyRetention(reserved, hops); got < gas {
				t.Errorf("reserve(%d, %d)=%d survives to %d, want >= %d", gas, hops, reserved, got, gas)
			}
		}
	}
}

// TestMinGasForV2SubmitProofCoversDispatchCheck asserts the reserved gas for a V2
// message, after the three 63/64 retentions to reach v2_dispatch's check, still leaves
// gasleft()*63/64 >= cmd.gas + overhead for every command — i.e. the on-chain check
// passes. BaseMessageVerificationGas covers _verifyCommitment consumed before dispatch.
func TestMinGasForV2SubmitProofCoversDispatchCheck(t *testing.T) {
	cmdGas := uint64(500_000)
	proof := &MessageProof{}
	proof.Message.OriginalMessage.Commands = []CommandWrapper{{MaxDispatchGas: types.NewU64(cmdGas)}}

	reserved := minGasForV2SubmitProof(proof)

	// Gas available at v2_dispatch after aggregate3 -> v2_submit -> v2_dispatch, minus
	// the verification consumed in v2_submit.
	atDispatch := applyRetention(reserved-BaseMessageVerificationGas, 2)
	required := cmdGas + gatewayDispatchOverheadBuffer
	if atDispatch*63/64 < required {
		t.Fatalf("v2_dispatch check would fail: gasleft*63/64=%d < required %d", atDispatch*63/64, required)
	}

	// The reserve must exceed the old linear budget (cmd.gas + overhead + base).
	if linear := BaseMessageVerificationGas + cmdGas + gatewayDispatchOverheadBuffer; reserved <= linear {
		t.Fatalf("reserve %d not greater than linear budget %d; 63/64 amplification missing", reserved, linear)
	}
}

func TestMulticallGasFloorIncludesBeefyHeadroom(t *testing.T) {
	msgGas := uint64(336_426)
	floor := multicallGasFloor(msgGas)
	wantMin := BaseBeefyFiatShamirGas + multicallBeefyGasHeadroom + multicall3AggregateOverheadBuffer + msgGas
	if floor != wantMin {
		t.Fatalf("multicallGasFloor=%d, want %d", floor, wantMin)
	}
	// Headroom must keep the floor above the pre-fix budget that lost nonce 638
	// (BaseBeefyFiatShamirGas + overhead + msg only).
	legacy := BaseBeefyFiatShamirGas + multicall3AggregateOverheadBuffer + msgGas
	if floor <= legacy {
		t.Fatalf("floor %d did not exceed legacy budget %d", floor, legacy)
	}
}

func TestEstimatedGasForV2SubmitProofUsesUnlockNotMaxDispatch(t *testing.T) {
	proof := &MessageProof{}
	proof.Message.OriginalMessage.Commands = []CommandWrapper{
		{Kind: 2, MaxDispatchGas: types.NewU64(200_000)},
	}
	limitGas := minGasForV2SubmitProof(proof)
	feeGas := estimatedGasForV2SubmitProof(proof)
	if feeGas >= limitGas {
		t.Fatalf("estimated fee gas %d should be below MaxDispatchGas-based limit gas %d", feeGas, limitGas)
	}
	// Unlock path should price BaseUnlockGas (80k), not MaxDispatchGas (200k).
	wantFee := gasWithForwardingReserve(BaseMessageVerificationGas, 1) +
		gasWithForwardingReserve(BaseUnlockGas+gatewayDispatchOverheadBuffer, 3)
	if feeGas != wantFee {
		t.Fatalf("estimatedGasForV2SubmitProof=%d, want %d", feeGas, wantFee)
	}
}

func TestStrictAggregate3CallsDisallowFailure(t *testing.T) {
	in := []contracts.Multicall3Call3{
		{Target: common.HexToAddress("0x1"), AllowFailure: false, CallData: []byte{1}},
		{Target: common.HexToAddress("0x2"), AllowFailure: true, CallData: []byte{2}},
	}
	out := strictAggregate3Calls(in)
	if len(out) != 2 {
		t.Fatalf("len=%d", len(out))
	}
	if !in[1].AllowFailure {
		t.Fatal("input should be unchanged")
	}
	for i, c := range out {
		if c.AllowFailure {
			t.Fatalf("call[%d] AllowFailure still true", i)
		}
		if c.Target != in[i].Target || string(c.CallData) != string(in[i].CallData) {
			t.Fatalf("call[%d] fields not copied", i)
		}
	}
}

func TestApplyEffectiveGasLimitRaisesOpts(t *testing.T) {
	opts := &bind.TransactOpts{GasLimit: 2_000_000}
	applyEffectiveGasLimit(opts, &DryRunMetrics{EffectiveGasLimit: 4_000_000})
	if opts.GasLimit != 4_000_000 {
		t.Fatalf("GasLimit=%d, want 4000000", opts.GasLimit)
	}
	applyEffectiveGasLimit(opts, &DryRunMetrics{EffectiveGasLimit: 3_000_000})
	if opts.GasLimit != 4_000_000 {
		t.Fatalf("GasLimit should not shrink, got %d", opts.GasLimit)
	}
}
