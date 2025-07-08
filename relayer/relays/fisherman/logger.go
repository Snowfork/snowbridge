package fisherman

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

func Hex(b []byte) string {
	return types.HexEncodeToString(b)
}
