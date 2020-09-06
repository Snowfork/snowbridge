package substrate

import (
	"github.com/ethereum/go-ethereum/log"
	"github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"

	etypes "github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

type Writer struct {
	conn *Connection
	stop <-chan int
}

func NewWriter(conn *Connection, stop <-chan int) (*Writer, error) {
	return &Writer{
		conn: conn,
		stop: stop,
	}, nil
}

func (wr *Writer) Start() error {
	log.Debug("Starting writer")
	return nil
}

func (wr *Writer) Resolve(_ *chain.Message) {

}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) write(_ string, _ []byte) error {
	return nil
}

// SubmitPacket submits a packet, it returns true
func (wr *Writer) SubmitPacket(appID [32]byte, packet etypes.PacketV2) error {

	appid := types.NewBytes32(appID)

	payload, err := types.EncodeToBytes(packet)
	if err != nil {
		return err
	}

	message := types.NewBytes(payload)

	c, err := types.NewCall(&wr.conn.metadata, "Bridge.send", appid, message)
	if err != nil {
		return err
	}

	ext := types.NewExtrinsic(c)

	era := types.ExtrinsicEra{IsMortalEra: false}

	genesisHash, err := wr.conn.api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}

	rv, err := wr.conn.api.RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return err
	}

	key, err := types.CreateStorageKey(&wr.conn.metadata, "System", "Account", wr.conn.kp.PublicKey, nil)
	if err != nil {
		return err
	}

	var accountInfo types.AccountInfo
	ok, err := wr.conn.api.RPC.State.GetStorageLatest(key, &accountInfo)
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

	err = extI.Sign(*wr.conn.kp, o)
	if err != nil {
		return err
	}

	_, err = wr.conn.api.RPC.Author.SubmitExtrinsic(extI)
	if err != nil {
		return err
	}

	return nil
}
