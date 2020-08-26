package ethereum

import (
	"bytes"
	"context"
	"math/big"

	"github.com/spf13/viper"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	log "github.com/sirupsen/logrus"
	"golang.org/x/crypto/sha3"

	keybase "github.com/snowfork/polkadot-ethereum/bridgerelayer/keybase/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/substrate"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
	"github.com/snowfork/polkadot-ethereum/prover"
)

// Router packages raw event data as Packets and relays them to the bridge
type Router struct {
	keybase  *keybase.Keypair
	sc       *substrate.Client
	ec       *ethclient.Client
	verifier common.Address
}

// NewRouter initializes a new instance of Router
func NewRouter(websocketURL string, keybase *keybase.Keypair, verifier common.Address) (*Router, error) {
	substrateClient, err := substrate.NewClient()
	if err != nil {
		return nil, err
	}

	ethereumClient, err := ethclient.Dial(websocketURL)
	if err != nil {
		return nil, err
	}

	return &Router{
		keybase:  keybase,
		sc:       substrateClient,
		ec:       ethereumClient,
		verifier: verifier,
	}, nil
}

// Route packages tx data as a packet and relays it to the bridge
func (er Router) Route(eventData types.EventData) error {

	appAddress := eventData.Contract.Bytes()
	var appID [32]byte
	copy(appID[:], appAddress)

	packet, err := er.buildPacket(eventData.Contract, eventData.Data)
	if err != nil {
		return err
	}

	err = er.sendPacket(appID, packet)
	if err != nil {
		return err
	}

	return nil
}

// BuildPacket builds a data packet from tx data
func (er Router) buildPacket(id common.Address, eLog ctypes.Log) (types.PacketV2, error) {
	// RLP encode event log's Address, Topics, and Data
	var buff bytes.Buffer
	err := eLog.EncodeRLP(&buff)
	if err != nil {
		return types.PacketV2{}, err
	}

	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(buff.Bytes(), er.keybase.PrivateKey())
	if err != nil {
		return types.PacketV2{}, err
	}

	packet := types.PacketV2{
		Data:      buff.Bytes(),
		Signature: proof.Signature,
	}
	return packet, nil
}

// SendPacket sends a tx data packet to the bridge
func (er Router) sendPacket(appID [32]byte, packet types.PacketV2) error {
	log.Info("Sending packet:\n", packet)
	er.sc.SubmitPacket(appID, packet)
	return nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (er Router) Submit(appName string, data []byte) error {

	log.Info("Submitting ", appName, " message to Ethereum")

	// Get address of ethereum app
	appHexAddr := viper.GetString(strings.Join([]string{"ethereum", "apps", appName}, "."))
	appAddress := common.HexToAddress(appHexAddr)
	log.Info("App Address: ", appHexAddr)

	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(data, er.keybase.PrivateKey())
	if err != nil {
		return err
	}

	nonce, err := er.ec.PendingNonceAt(context.Background(), er.keybase.CommonAddress())
	if err != nil {
		return err
	}

	value := big.NewInt(0)      // in wei (0 eth)
	gasLimit := uint64(2000000) // in units
	gasPrice, err := er.ec.SuggestGasPrice(context.Background())
	if err != nil {
		return err
	}

	// Calculate the method ID of our function using crypto.sha3
	submitFnSignature := []byte("submit(data,sig)")
	hash := sha3.NewLegacyKeccak256()
	hash.Write(submitFnSignature)
	methodID := hash.Sum(nil)[:4]

	var txData []byte
	txData = append(txData, methodID...)
	txData = append(txData, data...) // TODO: consider padding bytes with common.LeftPadBytes(data.Bytes(), 32)
	txData = append(txData, proof.Signature...)

	tx := ctypes.NewTransaction(nonce, appAddress, value, gasLimit, gasPrice, data)
	signedTx, err := ctypes.SignTx(tx, ctypes.HomesteadSigner{}, er.keybase.PrivateKey())
	if err != nil {
		return err
	}

	err = er.ec.SendTransaction(context.Background(), signedTx)
	if err != nil {
		return err
	}

	log.Info("tx sent: ", signedTx.Hash().Hex())
	return nil
}
