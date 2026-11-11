package parachainV3

import (
	"context"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type fakeServiceFees struct {
	fee *big.Int
	err error
	ids []string
}

func (f *fakeServiceFees) ServiceFeeToEthereumWei(_ context.Context, messageID string) (*big.Int, error) {
	f.ids = append(f.ids, messageID)
	return f.fee, f.err
}

func TestV2CollectedFeeWei(t *testing.T) {
	proof := &MessageProof{
		Message: OutboundQueueMessageWithFee{
			OriginalMessage: OutboundQueueMessage{
				Nonce: types.NewU64(9),
				Topic: types.NewH256(common.HexToHash("0xab").Bytes()),
			},
			Fee: *big.NewInt(50_000_000_000_000),
		},
	}

	t.Run("nil fees returns attached only", func(t *testing.T) {
		got := v2CollectedFeeWei(context.Background(), proof, nil)
		assert.Equal(t, "50000000000000", got.String())
	})

	t.Run("service fee is added", func(t *testing.T) {
		fees := &fakeServiceFees{fee: big.NewInt(100_000_000_000_000)}
		got := v2CollectedFeeWei(context.Background(), proof, fees)
		assert.Equal(t, "150000000000000", got.String())
		require.Len(t, fees.ids, 1)
		assert.Equal(t, proof.Message.OriginalMessage.Topic.Hex(), fees.ids[0])
	})

	t.Run("lookup failure falls back to attached fee", func(t *testing.T) {
		fees := &fakeServiceFees{err: assert.AnError}
		got := v2CollectedFeeWei(context.Background(), proof, fees)
		assert.Equal(t, "50000000000000", got.String())
	})

	t.Run("zero service fee does not change total", func(t *testing.T) {
		fees := &fakeServiceFees{fee: big.NewInt(0)}
		got := v2CollectedFeeWei(context.Background(), proof, fees)
		assert.Equal(t, "50000000000000", got.String())
	})
}
