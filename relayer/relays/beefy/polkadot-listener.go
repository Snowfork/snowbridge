package beefy

import (
	"context"
	"fmt"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/substrate"
)

type PolkadotListener struct {
	config *SourceConfig
	conn   *relaychain.Connection
}

func NewPolkadotListener(
	config *SourceConfig,
	conn *relaychain.Connection,
) *PolkadotListener {
	return &PolkadotListener{
		config: config,
		conn:   conn,
	}
}

func (li *PolkadotListener) Start(
	ctx context.Context,
	eg *errgroup.Group,
	currentBeefyBlock uint64,
) (<-chan Request, error) {
	requests := make(chan Request, 1)

	eg.Go(func() error {
		defer close(requests)
		err := li.scanCommitments(ctx, currentBeefyBlock, requests)
		if err != nil {
			return err
		}
		return nil
	})

	return requests, nil
}

func (li *PolkadotListener) scanCommitments(
	ctx context.Context,
	currentBeefyBlock uint64,
	requests chan<- Request,
) error {
	in, err := ScanCommitments(ctx, li.conn.Metadata(), li.conn.API(), currentBeefyBlock+1)
	if err != nil {
		return fmt.Errorf("scan provable commitments: %w", err)
	}

	for {
		select {
		case <-ctx.Done():
			return nil
		case result, ok := <-in:
			if !ok {
				return nil
			}
			if result.Error != nil {
				return fmt.Errorf("scan safe commitments: %w", result.Error)
			}

			committedBeefyBlock := uint64(result.SignedCommitment.Commitment.BlockNumber)
			validatorSetID := result.SignedCommitment.Commitment.ValidatorSetID
			nextValidatorSetID := uint64(result.Proof.Leaf.BeefyNextAuthoritySet.ID)

			validators, err := li.queryBeefyAuthorities(result.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch beefy authorities at block %v: %w", result.BlockHash, err)
			}

			task := Request{
				Validators:       validators,
				SignedCommitment: result.SignedCommitment,
				Proof:            result.Proof,
			}

			log.WithFields(log.Fields{
				"commitment": log.Fields{
					"blockNumber":        committedBeefyBlock,
					"validatorSetID":     validatorSetID,
					"nextValidatorSetID": nextValidatorSetID,
				},
			}).Info("Sending BEEFY commitment to ethereum writer")

			select {
			case <-ctx.Done():
				return ctx.Err()
			case requests <- task:
			}
		}
	}
}

func (li *PolkadotListener) queryBeefyAuthorities(blockHash types.Hash) ([]substrate.Authority, error) {
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "Beefy", "Authorities", nil, nil)
	if err != nil {
		return nil, fmt.Errorf("create storage key: %w", err)
	}
	var authorities []substrate.Authority
	ok, err := li.conn.API().RPC.State.GetStorage(storageKey, &authorities, blockHash)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("beefy authorities not found")
	}

	return authorities, nil
}

// Generate a Beefy update for a specific relay block.
// If the relay block is set to zero, just return the latest update.
// Loop until the specified relay block has been Beefy-finalized,
// or return the next block that contains a Beefy justification.
func (li *PolkadotListener) generateBeefyUpdate(relayBlockNumber uint64) (Request, error) {
	api := li.conn.API()
	meta := li.conn.Metadata()
	var request Request
	beefyBlockHash, err := li.findNextBeefyBlock(relayBlockNumber)
	if err != nil {
		return request, fmt.Errorf("find match beefy block: %w", err)
	}

	commitment, proof, err := fetchCommitmentAndProof(meta, api, beefyBlockHash)
	if err != nil {
		return request, fmt.Errorf("fetch commitment and proof: %w", err)
	}

	committedBeefyBlockNumber := uint64(commitment.Commitment.BlockNumber)
	committedBeefyBlockHash, err := api.RPC.Chain.GetBlockHash(uint64(committedBeefyBlockNumber))

	validators, err := li.queryBeefyAuthorities(committedBeefyBlockHash)
	if err != nil {
		return request, fmt.Errorf("fetch beefy authorities at block %v: %w", committedBeefyBlockHash, err)
	}
	request = Request{
		Validators:       validators,
		SignedCommitment: *commitment,
		Proof:            *proof,
	}

	return request, nil
}

// Polkadot session length is ~4h ≈ 2400 blocks. Ops catch-up only needs the
// next validator-set handover, so we search one session (+margin) ahead of the
// light client rather than walking the full tip lag.
const sessionCatchupWindowBlocks = uint64(2500)

// generateBeefyUpdateForwardFromLightClient walks an ~one-session window upward
// from state.LatestBeefyBlock and returns the farthest still-compatible BEEFY
// commitment (ValidatorSetID <= NextValidatorSetID). With monotonic set IDs and
// an early stop past Next, that farthest commitment is the session handover
// whenever one exists in the window.
// Shared by the ops forward-beefy-catchup CLI and by generateBeefyUpdateWithMaxValidatorSet
// (parachain-v3 BuildConsensusTask) so both avoid an O(lag) tip→LC walk.
func (li *PolkadotListener) generateBeefyUpdateForwardFromLightClient(state *BeefyClientState, tip uint64) (*Request, error) {
	api := li.conn.API()
	meta := li.conn.Metadata()

	if tip <= state.LatestBeefyBlock {
		return nil, nil
	}

	start := state.LatestBeefyBlock + 1
	searchTip := tip
	if start+sessionCatchupWindowBlocks < tip {
		searchTip = start + sessionCatchupWindowBlocks
	}
	log.WithFields(log.Fields{
		"from":             start,
		"searchTip":        searchTip,
		"tip":              tip,
		"latestBeefyBlock": state.LatestBeefyBlock,
		"nextValidatorSet": state.NextValidatorSetID,
		"sessionWindow":    sessionCatchupWindowBlocks,
	}).Info("Searching session window for BEEFY validator-set handover commitment")

	readCommitmentAt := func(candidate uint64) (*types.SignedCommitment, types.Hash, error) {
		candidateHash, err := api.RPC.Chain.GetBlockHash(candidate)
		if err != nil {
			return nil, types.Hash{}, fmt.Errorf("fetch candidate block hash %d: %w", candidate, err)
		}
		candidateBlock, err := api.RPC.Chain.GetBlock(candidateHash)
		if err != nil {
			return nil, types.Hash{}, fmt.Errorf("fetch candidate block %d: %w", candidate, err)
		}
		for j := range candidateBlock.Justifications {
			sc := types.OptionalSignedCommitment{}
			if candidateBlock.Justifications[j].EngineID() == "BEEF" {
				err := types.DecodeFromBytes(candidateBlock.Justifications[j].Payload(), &sc)
				if err != nil {
					return nil, types.Hash{}, fmt.Errorf("decode BEEFY signed commitment at block %d: %w", candidate, err)
				}
				ok, value := sc.Unwrap()
				if ok {
					return &value, candidateHash, nil
				}
			}
		}
		return nil, types.Hash{}, nil
	}
	findOnGrid := func(candidate uint64) (*types.SignedCommitment, types.Hash, uint64, error) {
		// BEEFY justifications are every ~8 blocks but phase relative to
		// LatestBeefyBlock can be +4 rather than +8; search a full period.
		for d := int64(0); d <= 7; d++ {
			for _, delta := range []int64{d, -d} {
				if d == 0 && delta != 0 {
					continue
				}
				c := int64(candidate) + delta
				if c < int64(start) || c > int64(searchTip) {
					continue
				}
				block := uint64(c)
				commitment, hash, err := readCommitmentAt(block)
				if err != nil {
					return nil, types.Hash{}, 0, err
				}
				if commitment != nil {
					return commitment, hash, block, nil
				}
			}
		}
		return nil, types.Hash{}, 0, nil
	}

	maxK := (searchTip - state.LatestBeefyBlock) / 8

	// Linear scan over the session grid. Binary search is unsafe here: an empty
	// probe carries no ValidatorSetID signal, so discarding either half can
	// skip the handover range. Last kept probe is the farthest compatible
	// commitment (= handover when a Next-set commitment exists before the early stop).
	// When lag < 8 (maxK == 0), scan every block in [start, searchTip] — the
	// BEEFY phase can be +4 after a handover, so a strict 8-block grid misses.
	var bestCandidate uint64
	var bestCommitment *types.SignedCommitment
	record := func(foundAt uint64, commitment *types.SignedCommitment) bool {
		commitmentBlock := uint64(commitment.Commitment.BlockNumber)
		setID := uint64(commitment.Commitment.ValidatorSetID)
		if commitmentBlock <= state.LatestBeefyBlock {
			return false
		}
		if setID > state.NextValidatorSetID {
			log.WithFields(log.Fields{
				"candidate":        foundAt,
				"commitmentBlock":  commitmentBlock,
				"validatorSetID":   setID,
				"nextValidatorSet": state.NextValidatorSetID,
			}).Info("Reached validator set beyond light client; stopping session-window scan")
			return true // stop
		}
		bestCandidate = foundAt
		bestCommitment = commitment
		return false
	}

	if maxK == 0 {
		for candidate := start; candidate <= searchTip; candidate++ {
			commitment, _, err := readCommitmentAt(candidate)
			if err != nil {
				return nil, err
			}
			if commitment == nil {
				continue
			}
			if record(candidate, commitment) {
				break
			}
		}
	} else {
		for k := uint64(1); k <= maxK; k++ {
			candidate := state.LatestBeefyBlock + k*8
			commitment, _, foundAt, err := findOnGrid(candidate)
			if err != nil {
				return nil, err
			}
			if commitment == nil {
				continue
			}
			if record(foundAt, commitment) {
				break
			}
		}
	}

	if bestCommitment == nil {
		return nil, fmt.Errorf("no compatible BEEFY commitment found between %d and %d (latest light client block %d, next validator set %d)", start, searchTip, state.LatestBeefyBlock, state.NextValidatorSetID)
	}

	// Session-boundary leaves sometimes trap in mmr_generateProof; walk back the
	// 8-block grid until a proveable commitment is found (cap attempts).
	var (
		commitmentWithProof *types.SignedCommitment
		proof               *merkle.SimplifiedMMRProof
		usedCandidate       uint64
	)
	for attempt, candidate := 0, bestCandidate; attempt < 64 && candidate > state.LatestBeefyBlock; attempt, candidate = attempt+1, candidate-8 {
		c, h, err := readCommitmentAt(candidate)
		if err != nil {
			return nil, err
		}
		if c == nil {
			continue
		}
		if uint64(c.Commitment.BlockNumber) <= state.LatestBeefyBlock {
			continue
		}
		if uint64(c.Commitment.ValidatorSetID) > state.NextValidatorSetID {
			continue
		}
		commitmentWithProof, proof, err = fetchCommitmentAndProof(meta, api, h)
		if err != nil {
			log.WithError(err).WithField("candidate", candidate).Warn("MMR proof failed; trying earlier commitment")
			continue
		}
		usedCandidate = candidate
		break
	}
	if commitmentWithProof == nil || proof == nil {
		return nil, fmt.Errorf("no proveable compatible BEEFY commitment found between %d and %d (best candidate %d)", start, searchTip, bestCandidate)
	}

	committedBeefyBlockNumber := uint64(commitmentWithProof.Commitment.BlockNumber)
	committedBeefyBlockHash, err := api.RPC.Chain.GetBlockHash(committedBeefyBlockNumber)
	if err != nil {
		return nil, fmt.Errorf("fetch commitment block hash %d: %w", committedBeefyBlockNumber, err)
	}
	validators, err := li.queryBeefyAuthorities(committedBeefyBlockHash)
	if err != nil {
		return nil, fmt.Errorf("fetch beefy authorities at block %v: %w", committedBeefyBlockHash, err)
	}

	log.WithFields(log.Fields{
		"candidate":       usedCandidate,
		"bestCandidate":   bestCandidate,
		"commitmentBlock": committedBeefyBlockNumber,
		"validatorSetID":  commitmentWithProof.Commitment.ValidatorSetID,
	}).Info("Found farthest proveable compatible BEEFY commitment")

	return &Request{
		Validators:       validators,
		SignedCommitment: *commitmentWithProof,
		Proof:            *proof,
	}, nil
}

// tipCommitmentProbeBlocks caps the common-case downward walk from the relay tip.
// BEEFY justifications land about every 8 blocks; a short probe finds the newest
// commitment without scanning the full lag.
const tipCommitmentProbeBlocks = uint64(32)

// generateBeefyUpdateWithMaxValidatorSet returns the newest provable BEEFY commitment
// at or below relayBeefyBlockNumber that satisfies:
//
//	commitmentBlock > state.LatestBeefyBlock
//	commitment.ValidatorSetID <= state.NextValidatorSetID
//
// Returns (nil, nil) when relayBeefyBlockNumber is not ahead of the light client's latest
// beefy block — there is nothing new to prove.
//
// Hot path (LC nearly caught up): walk a few blocks down from the tip for the newest
// commitment and use it when ValidatorSetID <= Next (~16 RPCs, as before).
// Lagging path (tip set ahead of Next): fall back to the session-window forward search
// instead of an O(lag) tip→LC walk that previously stuck parachain-v3 under mutex.
func (li *PolkadotListener) generateBeefyUpdateWithMaxValidatorSet(state *BeefyClientState, relayBeefyBlockNumber uint64) (*Request, error) {
	api := li.conn.API()
	meta := li.conn.Metadata()

	if relayBeefyBlockNumber <= state.LatestBeefyBlock {
		log.WithFields(log.Fields{
			"relayBeefyBlockNumber": relayBeefyBlockNumber,
			"latestBeefyBlock":      state.LatestBeefyBlock,
		}).Debug("No new BEEFY commitment to prove")
		return nil, nil
	}

	floor := state.LatestBeefyBlock
	if relayBeefyBlockNumber > tipCommitmentProbeBlocks && relayBeefyBlockNumber-tipCommitmentProbeBlocks > floor {
		floor = relayBeefyBlockNumber - tipCommitmentProbeBlocks
	}

	for candidate := relayBeefyBlockNumber; candidate > floor; candidate-- {
		candidateHash, err := api.RPC.Chain.GetBlockHash(candidate)
		if err != nil {
			return nil, fmt.Errorf("fetch candidate block hash %d: %w", candidate, err)
		}
		candidateBlock, err := api.RPC.Chain.GetBlock(candidateHash)
		if err != nil {
			return nil, fmt.Errorf("fetch candidate block %d: %w", candidate, err)
		}

		var commitment *types.SignedCommitment
		for j := range candidateBlock.Justifications {
			sc := types.OptionalSignedCommitment{}
			if candidateBlock.Justifications[j].EngineID() == "BEEF" {
				err := types.DecodeFromBytes(candidateBlock.Justifications[j].Payload(), &sc)
				if err != nil {
					return nil, fmt.Errorf("decode BEEFY signed commitment at block %d: %w", candidate, err)
				}
				ok, value := sc.Unwrap()
				if ok {
					commitment = &value
				}
			}
		}
		if commitment == nil {
			continue
		}

		commitmentBlock := uint64(commitment.Commitment.BlockNumber)
		if commitmentBlock <= state.LatestBeefyBlock {
			continue
		}
		setID := uint64(commitment.Commitment.ValidatorSetID)
		if setID > state.NextValidatorSetID {
			log.WithFields(log.Fields{
				"candidate":        candidate,
				"commitmentBlock":  commitmentBlock,
				"validatorSetID":   setID,
				"nextValidatorSet": state.NextValidatorSetID,
			}).Info("Tip BEEFY commitment beyond light client next set; using session-window forward search")
			return li.generateBeefyUpdateForwardFromLightClient(state, relayBeefyBlockNumber)
		}

		commitmentWithProof, proof, err := fetchCommitmentAndProof(meta, api, candidateHash)
		if err != nil {
			return nil, fmt.Errorf("fetch commitment and proof: %w", err)
		}
		committedBeefyBlockNumber := uint64(commitmentWithProof.Commitment.BlockNumber)
		committedBeefyBlockHash, err := api.RPC.Chain.GetBlockHash(committedBeefyBlockNumber)
		if err != nil {
			return nil, fmt.Errorf("fetch commitment block hash %d: %w", committedBeefyBlockNumber, err)
		}
		validators, err := li.queryBeefyAuthorities(committedBeefyBlockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch beefy authorities at block %v: %w", committedBeefyBlockHash, err)
		}
		return &Request{
			Validators:       validators,
			SignedCommitment: *commitmentWithProof,
			Proof:            *proof,
		}, nil
	}

	// No justification in the short tip probe (odd phase / tiny lag) — scan forward.
	return li.generateBeefyUpdateForwardFromLightClient(state, relayBeefyBlockNumber)
}

func (li *PolkadotListener) findNextBeefyBlock(beefyBlockNumber uint64) (beefyBlockHash types.Hash, err error) {
	api := li.conn.API()
	var finalizedBeefyBlockHash types.Hash
	for {
		finalizedBeefyBlockHash, err = api.RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return beefyBlockHash, fmt.Errorf("fetch beefy finalized head: %w", err)
		}
		finalizedBeefyBlockHeader, err := api.RPC.Chain.GetHeader(finalizedBeefyBlockHash)
		if err != nil {
			return beefyBlockHash, fmt.Errorf("fetch block header: %w", err)
		}
		latestBeefyBlockNumber := uint64(finalizedBeefyBlockHeader.Number)
		if latestBeefyBlockNumber < beefyBlockNumber {
			// The relay block not finalized yet, just wait and retry
			time.Sleep(6 * time.Second)
		} else {
			beefyBlockHash = finalizedBeefyBlockHash
			break
		}
	}
	return beefyBlockHash, nil
}
