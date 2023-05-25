package parachain

import (
	"context"
	"fmt"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/rpc/author"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"golang.org/x/sync/errgroup"
)

type ParachainWriter struct {
	conn                 *Connection
	nonce                uint32
	pool                 *ExtrinsicPool
	genesisHash          types.Hash
	maxWatchedExtrinsics int64
	mu                   sync.Mutex
}

func NewParachainWriter(
	conn *Connection,
	maxWatchedExtrinsics int64,
) *ParachainWriter {
	return &ParachainWriter{
		conn:                 conn,
		maxWatchedExtrinsics: maxWatchedExtrinsics,
	}
}

func (wr *ParachainWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	nonce, err := wr.queryAccountNonce()
	if err != nil {
		return err
	}

	wr.nonce = nonce

	genesisHash, err := wr.conn.API().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}
	wr.genesisHash = genesisHash

	wr.pool = NewExtrinsicPool(eg, wr.conn, wr.maxWatchedExtrinsics)

	return nil
}

func (wr *ParachainWriter) WriteToParachainAndRateLimit(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	wr.mu.Lock()
	defer wr.mu.Unlock()

	extI, err := wr.prepExtrinstic(ctx, extrinsicName, payload...)
	if err != nil {
		return err
	}

	callback := func(h types.Hash) error { return nil }

	err = wr.pool.WaitForSubmitAndWatch(ctx, extI, callback)
	if err != nil {
		return err
	}

	wr.nonce = wr.nonce + 1

	return nil
}

func (wr *ParachainWriter) WriteToParachainAndWatch(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	wr.mu.Lock()
	defer wr.mu.Unlock()

	sub, err := wr.writeToParachain(ctx, extrinsicName, payload...)
	if err != nil {
		return err
	}

	wr.nonce = wr.nonce + 1

	defer sub.Unsubscribe()

	for {
		select {
		case status := <-sub.Chan():
			if status.IsDropped || status.IsInvalid || status.IsUsurped || status.IsFinalityTimeout {
				return fmt.Errorf("parachain write status was dropped, invalid, usurped or finality timed out")
			}
			if status.IsInBlock || status.IsFinalized {
				return nil
			}
		case err = <-sub.Err():
			return err
		case <-ctx.Done():
			return nil
		}
	}
}

func (wr *ParachainWriter) writeToParachain(ctx context.Context, extrinsicName string, payload ...interface{}) (*author.ExtrinsicStatusSubscription, error) {
	extI, err := wr.prepExtrinstic(ctx, extrinsicName, payload...)
	if err != nil {
		return nil, err
	}

	sub, err := wr.conn.API().RPC.Author.SubmitAndWatchExtrinsic(*extI)
	if err != nil {
		return nil, err
	}

	return sub, nil
}

func (wr *ParachainWriter) queryAccountNonce() (uint32, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "System", "Account", wr.conn.Keypair().PublicKey, nil)
	if err != nil {
		return 0, err
	}

	var accountInfo types.AccountInfo
	ok, err := wr.conn.API().RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil {
		return 0, err
	}
	if !ok {
		return 0, fmt.Errorf("no account info found for %s", wr.conn.Keypair().URI)
	}

	return uint32(accountInfo.Nonce), nil
}

func (wr *ParachainWriter) prepExtrinstic(ctx context.Context, extrinsicName string, payload ...interface{}) (*types.Extrinsic, error) {
	meta, err := wr.conn.API().RPC.State.GetMetadataLatest()
	if err != nil {
		return nil, err
	}

	c, err := types.NewCall(meta, extrinsicName, payload...)
	if err != nil {
		return nil, err
	}

	latestHash, err := wr.conn.API().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return nil, err
	}

	latestBlock, err := wr.conn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return nil, err
	}

	ext := types.NewExtrinsic(c)
	era := NewMortalEra(uint64(latestBlock.Block.Header.Number))

	genesisHash, err := wr.conn.API().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return nil, err
	}

	rv, err := wr.conn.API().RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return nil, err
	}

	o := types.SignatureOptions{
		BlockHash:          latestHash,
		Era:                era,
		GenesisHash:        genesisHash,
		Nonce:              types.NewUCompactFromUInt(uint64(wr.nonce)),
		SpecVersion:        rv.SpecVersion,
		Tip:                types.NewUCompactFromUInt(0),
		TransactionVersion: rv.TransactionVersion,
	}

	extI := ext

	err = extI.Sign(*wr.conn.Keypair(), o)
	if err != nil {
		return nil, err
	}

	return &extI, nil
}

func (wr *ParachainWriter) GetLastBasicChannelBlockNumber() (uint64, error) {
	return wr.getNumberFromParachain("BasicInboundQueue", "LatestVerifiedBlockNumber")
}

func (wr *ParachainWriter) GetLastBasicChannelNonceByAddresses(addresses []common.Address) (map[common.Address]uint64, error) {
	addressNonceMap := make(map[common.Address]uint64, len(addresses))

	for _, address := range addresses {
		nonce, err := wr.GetLastBasicChannelNonceByAddress(address)
		if err != nil {
			return addressNonceMap, fmt.Errorf("fetch basic channel nonce for address %s: %w", address, err)
		}

		addressNonceMap[address] = uint64(nonce)
	}

	return addressNonceMap, nil
}

func (wr *ParachainWriter) GetLastBasicChannelNonceByAddress(address common.Address) (uint64, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "BasicInboundQueue", "Nonce", address[:], nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key for basic channel nonces: %w", err)
	}

	var nonce types.U64
	_, err = wr.conn.API().RPC.State.GetStorageLatest(key, &nonce)
	if err != nil {
		return 0, fmt.Errorf("get storage for latest basic channel nonces (err): %w", err)
	}

	return uint64(nonce), nil
}

func (wr *ParachainWriter) GetLastExecutionHeaderState() (state.ExecutionHeader, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", "LatestExecutionHeader", nil, nil)
	if err != nil {
		return state.ExecutionHeader{}, fmt.Errorf("create storage key for LatestExecutionHeaderState: %w", err)
	}

	var storageState struct {
		BeaconBlockRoot types.H256
		BeaconSlot      types.U64
		BlockHash       types.H256
		BlockNumber     types.U64
	}
	_, err = wr.conn.API().RPC.State.GetStorageLatest(key, &storageState)
	if err != nil {
		return state.ExecutionHeader{}, fmt.Errorf("get storage for LatestExecutionHeaderState (err): %w", err)
	}

	return state.ExecutionHeader{
		BeaconBlockRoot: common.Hash(storageState.BeaconBlockRoot),
		BeaconSlot:      uint64(storageState.BeaconSlot),
		BlockHash:       common.Hash(storageState.BlockHash),
		BlockNumber:     uint64(storageState.BlockNumber),
	}, nil
}

func (wr *ParachainWriter) GetLastFinalizedHeaderState() (state.FinalizedHeader, error) {
	latestFinalizedBlockRootKey, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", "LatestFinalizedBlockRoot", nil, nil)
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("create storage key for LatestFinalizedBlockRoot: %w", err)
	}

	var latestFinalizedBlockRoot types.H256
	_, err = wr.conn.API().RPC.State.GetStorageLatest(latestFinalizedBlockRootKey, &latestFinalizedBlockRoot)
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("fetch LatestFinalizedBlockRoot: %w", err)
	}

	finalizedBeaconStateKey, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", "FinalizedBeaconState", latestFinalizedBlockRoot[:], nil)
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("create storage key for FinalizedBeaconState: %w", err)
	}
	var compactBeaconState scale.CompactBeaconState
	_, err = wr.conn.API().RPC.State.GetStorageLatest(finalizedBeaconStateKey, &compactBeaconState)
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("fetch FinalizedBeaconState: %w", err)
	}

	return state.FinalizedHeader{
		BeaconSlot:      uint64(compactBeaconState.Slot.Int64()),
		BeaconBlockRoot: common.Hash(latestFinalizedBlockRoot),
	}, nil
}

func (wr *ParachainWriter) GetFinalizedHeaderStateByBlockRoot(blockRoot types.H256) (state.FinalizedHeader, error) {
	finalizedBeaconStateKey, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", "FinalizedBeaconState", blockRoot[:], nil)
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("create storage key for FinalizedBeaconState: %w", err)
	}
	var compactBeaconState scale.CompactBeaconState
	_, err = wr.conn.API().RPC.State.GetStorageLatest(finalizedBeaconStateKey, &compactBeaconState)
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("fetch FinalizedBeaconState: %w", err)
	}
	if compactBeaconState.Slot.Int64() == 0 {
		return state.FinalizedHeader{}, fmt.Errorf("FinalizedBeaconState not exist at %s", blockRoot.Hex())
	}

	return state.FinalizedHeader{
		BeaconSlot:      uint64(compactBeaconState.Slot.Int64()),
		BeaconBlockRoot: common.Hash(blockRoot),
	}, nil
}

func (wr *ParachainWriter) getHashFromParachain(pallet, storage string) (common.Hash, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), pallet, storage, nil, nil)
	if err != nil {
		return common.Hash{}, fmt.Errorf("create storage key for %s:%s: %w", pallet, storage, err)
	}

	var hash types.H256
	_, err = wr.conn.API().RPC.State.GetStorageLatest(key, &hash)
	if err != nil {
		return common.Hash{}, fmt.Errorf("get storage for %s:%s (err): %w", pallet, storage, err)
	}

	return common.HexToHash(hash.Hex()), nil
}

func (wr *ParachainWriter) getNumberFromParachain(pallet, storage string) (uint64, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), pallet, storage, nil, nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key for %s:%s: %w", pallet, storage, err)
	}

	var number types.U64
	_, err = wr.conn.API().RPC.State.GetStorageLatest(key, &number)
	if err != nil {
		return 0, fmt.Errorf("get storage for %s:%s (err): %w", pallet, storage, err)
	}

	return uint64(number), nil
}
