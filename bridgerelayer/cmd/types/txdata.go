package types

import "fmt"

type TxData interface {
	GetID() []byte
	String() string
}

type EthTxData struct {
	ChainID   []byte
	BlockHash []byte
	TxHash    []byte
	Data      []interface{}
	Reciept   []interface{}
}

func NewEthTxData(chainID, blockHash, txHash, data, reciept []byte) EthTxData {
	return EthTxData{
		ChainID:   chainID,
		BlockHash: blockHash,
		TxHash:    txHash,
		Data:      []interface{}{data},
		Reciept:   []interface{}{reciept},
	}
}

func (etd EthTxData) GetID() []byte {
	return append(etd.ChainID[:], etd.TxHash[:]...)
}

func (etd EthTxData) String() string {
	return fmt.Sprintf(`Eth Tx %d:
	ChainID:            %s
	BlockHash:        %s
	TxHash:             %s
	Data:                 %s
	Reciept:           %s`,
		etd.GetID(), etd.ChainID, etd.BlockHash,
		etd.TxHash, etd.Data, etd.Reciept,
	)
}
