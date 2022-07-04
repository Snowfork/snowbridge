package beacon

import (
	"context"
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"golang.org/x/sync/errgroup"
)

type ParachainWriter struct {
	conn        *parachain.Connection
	nonce       uint32
	pool        *parachain.ExtrinsicPool
	genesisHash types.Hash
}

func NewParachainWriter(
	conn *parachain.Connection,
) *ParachainWriter {
	return &ParachainWriter{
		conn: conn,
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

	wr.pool = parachain.NewExtrinsicPool(eg, wr.conn)

	return nil
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

func (wr *ParachainWriter) WriteToParachain(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	meta, err := wr.conn.API().RPC.State.GetMetadataLatest()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, extrinsicName, payload...)
	if err != nil {
		return err
	}

	latestHash, err := wr.conn.API().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return err
	}

	latestBlock, err := wr.conn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return err
	}

	ext := types.NewExtrinsic(c)
	era := parachain.NewMortalEra(uint64(latestBlock.Block.Header.Number))

	genesisHash, err := wr.conn.API().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}

	rv, err := wr.conn.API().RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return err
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
		return err
	}

	_, err = wr.conn.API().RPC.Author.SubmitAndWatchExtrinsic(extI)
	if err != nil {
		return err
	}

	wr.nonce = wr.nonce + 1

	return nil
}
