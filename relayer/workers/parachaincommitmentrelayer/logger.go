package parachaincommitmentrelayer

import (
	"encoding/hex"
	"encoding/json"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/basic"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/incentivized"
)

type ParaheadPartialLog struct {
	ParentHash     string
	Number         uint32
	StateRoot      string
	ExtrinsicsRoot string
}

type ParaHeadProofLog struct {
	Pos   *big.Int
	Width *big.Int
	Proof []string
}

type BeefyMMRLeafPartialLog struct {
	ParentNumber         uint32
	ParentHash           string
	NextAuthoritySetId   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot string
}

type BasicInboundChannelMessageLog struct {
	Target  common.Address
	Nonce   uint64
	Payload string
}

type IncentivizedInboundChannelMessageLog struct {
	Target  common.Address
	Nonce   uint64
	Fee     *big.Int
	Payload string
}

type BasicSubmitInput struct {
	Messages            []BasicInboundChannelMessageLog
	ParaheadPartial     ParaheadPartialLog
	ParaHeadProof       ParaHeadProofLog
	BeefyMMRLeafPartial BeefyMMRLeafPartialLog
}

type IncentivizedSubmitInput struct {
	Messages            []IncentivizedInboundChannelMessageLog
	ParaheadPartial     ParaheadPartialLog
	ParaHeadProof       ParaHeadProofLog
	BeefyMMRLeafPartial BeefyMMRLeafPartialLog
}

func (wr *EthereumChannelWriter) logBasicTx(
	messages []basic.BasicInboundChannelMessage,
	paraheadPartial basic.ParachainLightClientOwnParachainHeadPartial,
	paraHeadProof basic.ParachainLightClientParachainHeadProof,
	paraHeadProofRoot []byte,
	beefyMMRLeafPartial basic.ParachainLightClientBeefyMMRLeafPartial) error {

	var basicMessagesLog []BasicInboundChannelMessageLog
	for _, item := range messages {
		basicMessagesLog = append(basicMessagesLog, BasicInboundChannelMessageLog{
			Target:  item.Target,
			Nonce:   item.Nonce,
			Payload: hex.EncodeToString(item.Payload),
		})
	}
	var paraHeadProofString []string
	for _, item := range paraHeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, hex.EncodeToString(item[:]))
	}
	input := &BasicSubmitInput{
		Messages: basicMessagesLog,
		ParaheadPartial: ParaheadPartialLog{
			ParentHash:     hex.EncodeToString(paraheadPartial.ParentHash[:]),
			Number:         paraheadPartial.Number,
			StateRoot:      hex.EncodeToString(paraheadPartial.StateRoot[:]),
			ExtrinsicsRoot: hex.EncodeToString(paraheadPartial.ExtrinsicsRoot[:]),
		},
		ParaHeadProof: ParaHeadProofLog{
			Pos:   paraHeadProof.Pos,
			Width: paraHeadProof.Width,
			Proof: paraHeadProofString,
		},
		BeefyMMRLeafPartial: BeefyMMRLeafPartialLog{
			ParentNumber:         beefyMMRLeafPartial.ParentNumber,
			ParentHash:           hex.EncodeToString(beefyMMRLeafPartial.ParentHash[:]),
			NextAuthoritySetId:   beefyMMRLeafPartial.NextAuthoritySetId,
			NextAuthoritySetLen:  beefyMMRLeafPartial.NextAuthoritySetLen,
			NextAuthoritySetRoot: hex.EncodeToString(beefyMMRLeafPartial.NextAuthoritySetRoot[:]),
		}}
	b, err := json.Marshal(input)
	if err != nil {
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"input":             string(b),
		"paraHeadProofRoot": hex.EncodeToString(paraHeadProofRoot[:]),
	}).Info("Submitting tx")
	return nil
}

func (wr *EthereumChannelWriter) logIncentivizedTx(
	messages []incentivized.IncentivizedInboundChannelMessage,
	paraheadPartial incentivized.ParachainLightClientOwnParachainHeadPartial,
	paraHeadProof incentivized.ParachainLightClientParachainHeadProof,
	paraHeadProofRoot []byte,
	beefyMMRLeafPartial incentivized.ParachainLightClientBeefyMMRLeafPartial) error {

	var incentivizedMessagesLog []IncentivizedInboundChannelMessageLog
	for _, item := range messages {
		incentivizedMessagesLog = append(incentivizedMessagesLog, IncentivizedInboundChannelMessageLog{
			Target:  item.Target,
			Nonce:   item.Nonce,
			Fee:     item.Fee,
			Payload: hex.EncodeToString(item.Payload),
		})
	}

	var paraHeadProofString []string
	for _, item := range paraHeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, hex.EncodeToString(item[:]))
	}
	input := &IncentivizedSubmitInput{
		Messages: incentivizedMessagesLog,
		ParaheadPartial: ParaheadPartialLog{
			ParentHash:     hex.EncodeToString(paraheadPartial.ParentHash[:]),
			Number:         paraheadPartial.Number,
			StateRoot:      hex.EncodeToString(paraheadPartial.StateRoot[:]),
			ExtrinsicsRoot: hex.EncodeToString(paraheadPartial.ExtrinsicsRoot[:]),
		},
		ParaHeadProof: ParaHeadProofLog{
			Pos:   paraHeadProof.Pos,
			Width: paraHeadProof.Width,
			Proof: paraHeadProofString,
		},
		BeefyMMRLeafPartial: BeefyMMRLeafPartialLog{
			ParentNumber:         beefyMMRLeafPartial.ParentNumber,
			ParentHash:           hex.EncodeToString(beefyMMRLeafPartial.ParentHash[:]),
			NextAuthoritySetId:   beefyMMRLeafPartial.NextAuthoritySetId,
			NextAuthoritySetLen:  beefyMMRLeafPartial.NextAuthoritySetLen,
			NextAuthoritySetRoot: hex.EncodeToString(beefyMMRLeafPartial.NextAuthoritySetRoot[:]),
		}}
	b, err := json.Marshal(input)
	if err != nil {
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"input":             string(b),
		"paraHeadProofRoot": hex.EncodeToString(paraHeadProofRoot[:]),
	}).Info("Submitting tx")
	return nil
}
