package parachain

import (
	"context"
	"fmt"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
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
	maxBatchCallSize     int64
	mu                   sync.Mutex
}

func NewParachainWriter(
	conn *Connection,
	maxWatchedExtrinsics int64,
	maxBatchCallSize int64,
) *ParachainWriter {
	return &ParachainWriter{
		conn:                 conn,
		maxWatchedExtrinsics: maxWatchedExtrinsics,
		maxBatchCallSize:     maxBatchCallSize,
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

func (wr *ParachainWriter) BatchCall(ctx context.Context, extrinsic string, calls []interface{}) error {
	batchSize := int(wr.maxBatchCallSize)
	var j int
	for i := 0; i < len(calls); i += batchSize {
		j += batchSize
		if j > len(calls) {
			j = len(calls)
		}
		slicedCalls := append([]interface{}{}, calls[i:j]...)
		encodedCalls := make([]types.Call, len(slicedCalls))
		for k := range slicedCalls {
			call, err := wr.prepCall(extrinsic, slicedCalls[k])
			if err != nil {
				return err
			}
			encodedCalls[k] = *call
		}
		err := wr.WriteToParachainAndRateLimit(ctx, "Utility.batch_all", encodedCalls)
		if err != nil {
			return fmt.Errorf("batch call failed: %w", err)
		}
	}
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
			if status.IsFinalized {
				log.WithFields(log.Fields{
					"extrinsic": extrinsicName, "block": status.AsFinalized}).Debug("extrinsic finalized")
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

func (wr *ParachainWriter) prepCall(extrinsicName string, payload ...interface{}) (*types.Call, error) {
	meta, err := wr.conn.API().RPC.State.GetMetadataLatest()
	if err != nil {
		return nil, err
	}

	c, err := types.NewCall(meta, extrinsicName, payload...)
	if err != nil {
		return nil, err
	}
	return &c, nil
}

func (wr *ParachainWriter) GetLastBasicChannelBlockNumber() (uint64, error) {
	return wr.getNumberFromParachain("EthereumInboundQueue", "LatestVerifiedBlockNumber")
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
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumInboundQueue", "Nonce", address[:], nil)
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

func (wr *ParachainWriter) GetLastFinalizedHeaderState() (state.FinalizedHeader, error) {
	finalizedState, err := wr.GetFinalizedStateByStorageKey("LatestFinalizedBlockRoot")
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("fetch FinalizedBeaconState: %w", err)
	}
	initialCheckpointState, err := wr.GetFinalizedStateByStorageKey("InitialCheckpointRoot")
	if err != nil {
		return state.FinalizedHeader{}, fmt.Errorf("fetch InitialBeaconState: %w", err)
	}

	return state.FinalizedHeader{
		BeaconSlot:            uint64(finalizedState.Slot.Int64()),
		BeaconBlockRoot:       common.Hash(finalizedState.BlockRoot),
		InitialCheckpointSlot: uint64(initialCheckpointState.Slot.Int64()),
		InitialCheckpointRoot: common.Hash(initialCheckpointState.BlockRoot),
	}, nil
}

func (wr *ParachainWriter) GetFinalizedStateByStorageKey(key string) (scale.BeaconState, error) {
	storageRootKey, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", key, nil, nil)
	if err != nil {
		return scale.BeaconState{}, fmt.Errorf("create storage key: %w", err)
	}

	var storageRoot types.H256
	_, err = wr.conn.API().RPC.State.GetStorageLatest(storageRootKey, &storageRoot)
	if err != nil {
		return scale.BeaconState{}, fmt.Errorf("fetch storage root: %w", err)
	}

	storageStateKey, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", "FinalizedBeaconState", storageRoot[:], nil)
	if err != nil {
		return scale.BeaconState{}, fmt.Errorf("create storage key for FinalizedBeaconState: %w", err)
	}
	var compactBeaconState scale.CompactBeaconState
	_, err = wr.conn.API().RPC.State.GetStorageLatest(storageStateKey, &compactBeaconState)
	if err != nil {
		return scale.BeaconState{}, fmt.Errorf("fetch FinalizedBeaconState: %w", err)
	}
	return scale.BeaconState{BlockRoot: storageRoot, CompactBeaconState: scale.CompactBeaconState{
		Slot:           compactBeaconState.Slot,
		BlockRootsRoot: compactBeaconState.BlockRootsRoot,
	}}, nil
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

func (wr *ParachainWriter) GetLastFinalizedStateIndex() (types.U32, error) {
	var index types.U32
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", "FinalizedBeaconStateIndex", nil, nil)
	if err != nil {
		return index, fmt.Errorf("create storage key for FinalizedBeaconStateIndex: %w", err)
	}

	_, err = wr.conn.API().RPC.State.GetStorageLatest(key, &index)
	if err != nil {
		return index, fmt.Errorf("get storage for FinalizedBeaconStateIndex (err): %w", err)
	}

	return index, nil
}

func (wr *ParachainWriter) GetFinalizedBeaconRootByIndex(index uint32) (types.H256, error) {
	var beaconRoot types.H256
	encodedIndex, err := types.EncodeToBytes(types.NewU32(index))
	if err != nil {
		return beaconRoot, fmt.Errorf("get finalized beacon root encode index error: %w", err)
	}
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconClient", "FinalizedBeaconStateMapping", encodedIndex, nil)
	if err != nil {
		return beaconRoot, fmt.Errorf("create storage key for FinalizedBeaconStateMapping: %w", err)
	}

	_, err = wr.conn.API().RPC.State.GetStorageLatest(key, &beaconRoot)
	if err != nil {
		return beaconRoot, fmt.Errorf("get storage for FinalizedBeaconStateMapping (err): %w", err)
	}

	return beaconRoot, nil
}

func (wr *ParachainWriter) FindCheckPointBackward(slot uint64) (state.FinalizedHeader, error) {
	var beaconState state.FinalizedHeader
	lastIndex, err := wr.GetLastFinalizedStateIndex()
	if err != nil {
		return beaconState, fmt.Errorf("GetLastFinalizedStateIndex error: %w", err)
	}
	startIndex := uint32(lastIndex)
	endIndex := uint32(0)
	if lastIndex > 256 {
		endIndex = endIndex - 256
	}
	for index := startIndex; index >= endIndex; index-- {
		beaconRoot, err := wr.GetFinalizedBeaconRootByIndex(index)
		if err != nil {
			return beaconState, fmt.Errorf("GetFinalizedBeaconRootByIndex %d, error: %w", index, err)
		}
		beaconState, err = wr.GetFinalizedHeaderStateByBlockRoot(beaconRoot)
		if err != nil {
			return beaconState, fmt.Errorf("GetFinalizedHeaderStateByBlockRoot %s, error: %w", beaconRoot.Hex(), err)
		}
		if beaconState.BeaconSlot < slot {
			break
		}
		if beaconState.BeaconSlot > slot && beaconState.BeaconSlot < slot+8192 {
			break
		}
	}
	if beaconState.BeaconSlot > slot && beaconState.BeaconSlot < slot+8192 {
		return beaconState, nil
	}
	return beaconState, fmt.Errorf("Can't find checkpoint on chain for slot %d", slot)
}
