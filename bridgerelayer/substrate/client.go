package substrate

import (
	gsrpc "github.com/Snowfork/go-substrate-rpc-client"
	"github.com/Snowfork/go-substrate-rpc-client/config"
	"github.com/Snowfork/go-substrate-rpc-client/signature"
	"github.com/Snowfork/go-substrate-rpc-client/types"

	etypes "github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// Client struct
type Client struct {
	api      *gsrpc.SubstrateAPI
	metadata *types.Metadata
}

// NewClient is a new Substrate client
func NewClient() (*Client, error) {

	api, err := gsrpc.NewSubstrateAPI(config.Default().RPCURL)
	if err != nil {
		return nil, err
	}

	metadata, err := api.RPC.State.GetMetadataLatest()
	if err != nil {
		panic(err)
	}

	client := Client{
		api, metadata,
	}

	return &client, nil
}

// SubmitPacket submits a packet, it returns true
func (client *Client) SubmitPacket(appID [32]byte, packet etypes.PacketV2) error {

	from, err := signature.KeyringPairFromSecret("//Alice", "")
	if err != nil {
		return err
	}

	meta, err := client.api.RPC.State.GetMetadataLatest()
	if err != nil {
		return err
	}

	appid := types.NewBytes32(appID)

	payload, err := types.EncodeToBytes(packet)
	if err != nil {
		return err
	}

	message := types.NewBytes(payload)

	c, err := types.NewCall(meta, "Bridge.send", appid, message)
	if err != nil {
		return err
	}

	ext := types.NewExtrinsic(c)

	era := types.ExtrinsicEra{IsMortalEra: false}

	genesisHash, err := client.api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}

	rv, err := client.api.RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return err
	}

	key, err := types.CreateStorageKey(meta, "System", "Account", from.PublicKey, nil)
	if err != nil {
		return err
	}

	var accountInfo types.AccountInfo
	ok, err := client.api.RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil || !ok {
		return err
	}

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		BlockHash:   genesisHash,
		Era:         era,
		GenesisHash: genesisHash,
		Nonce:       types.NewUCompactFromUInt(uint64(nonce)),
		SpecVersion: rv.SpecVersion,
		TxVersion:   1,
		Tip:         types.NewUCompactFromUInt(0),
	}

	extI := ext

	err = extI.Sign(from, o)
	if err != nil {
		return err
	}

	_, err = client.api.RPC.Author.SubmitExtrinsic(extI)
	if err != nil {
		return err
	}

	return nil
}
