package parachainV3

import (
	"context"
	"math/big"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/prices"
)

// serviceFeeLookup returns the ether service fee the dApp deposited on Asset Hub for a
// message. It is Snowbridge revenue that the relayer does not receive, but it still counts
// toward gas break-even on the P→E fast path.
type serviceFeeLookup interface {
	ServiceFeeToEthereumWei(ctx context.Context, messageID string) (*big.Int, error)
}

// v2CollectedFeeWei returns the attached v2 message fee plus any UI/service fee from the
// indexer. Lookup failures count as zero service fee so the batch can still proceed on the
// attached fee alone.
func v2CollectedFeeWei(ctx context.Context, proof *MessageProof, fees serviceFeeLookup) *big.Int {
	fee := new(big.Int).Set(&proof.Message.Fee)
	if fees == nil || proof == nil {
		return fee
	}
	message := proof.Message.OriginalMessage
	serviceFee, err := fees.ServiceFeeToEthereumWei(ctx, message.Topic.Hex())
	if err != nil {
		log.WithField("nonce", message.Nonce).WithError(err).Warn("Failed to look up service fee; using attached fee only")
		return fee
	}
	if serviceFee != nil && serviceFee.Sign() > 0 {
		fee.Add(fee, serviceFee)
		log.WithFields(log.Fields{
			"nonce":          message.Nonce,
			"attachedFeeWei": proof.Message.Fee.String(),
			"serviceFeeWei":  serviceFee.String(),
			"collectedFeeWei": fee.String(),
		}).Debug("Included UI/service fee in v2 collected fee for gas break-even")
	}
	return fee
}

// newServiceFeeLookupFromEnv returns the indexer service-fee client when configured.
func newServiceFeeLookupFromEnv() serviceFeeLookup {
	if client := prices.NewServiceFeeClientFromEnv(); client != nil {
		return client
	}
	return nil
}
