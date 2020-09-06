package substrate

import (
	"encoding/hex"
	log "github.com/sirupsen/logrus"

	"github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
)

type Writer struct {
	conn *Connection
	messages <-chan chain.Message
	stop <-chan int
}

func NewWriter(conn *Connection, messages <-chan chain.Message, stop <-chan int) (*Writer, error) {
	return &Writer{
		conn: conn,
		messages: messages,
		stop: stop,
	}, nil
}

func (wr *Writer) Start() error {
	log.Debug("Starting writer")

	go func() {
		wr.writeLoop()
	}()

	return nil
}

func (wr *Writer) writeLoop() {
	for {
		select {
		case <-wr.stop:
			return
		case msg := <-wr.messages:
			err := wr.Write(&msg)
			if err != nil {
				log.WithFields(log.Fields{
					"appid": hex.EncodeToString(msg.AppID[:]),
					"error": err,
				}).Error("Failure submitting message to substrate")
			}
		}
	}
}

// SubmitPacket submits a packet, it returns true
func (wr *Writer) Write(msg *chain.Message) error {

	appid := types.NewBytes32(msg.AppID)
	message := types.NewBytes(msg.Payload)

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

	log.WithFields(log.Fields{
		"appid": hex.EncodeToString(msg.AppID[:]),
	}).Info("Submitted message to Substrate")

	return nil
}
