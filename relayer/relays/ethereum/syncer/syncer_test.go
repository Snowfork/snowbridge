package syncer_test

import (
	"context"
	"math/big"
	"math/rand"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/snowfork/snowbridge/relayer/relays/ethereum/syncer"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"golang.org/x/sync/errgroup"
)

func makeHeaderChain(num int, seed int64) []*types.Header {
	r := rand.New(rand.NewSource(seed))
	headers := make([]*types.Header, num)

	for i := 0; i < num; i++ {
		header := types.Header{
			Number: big.NewInt(int64(i)),
			Nonce:  types.EncodeNonce(r.Uint64()),
		}
		if i > 0 {
			header.ParentHash = headers[i-1].Hash()
		}
		headers[i] = &header
	}

	return headers
}

type TestSubscription struct{}

func (tes *TestSubscription) Unsubscribe()      {}
func (tes *TestSubscription) Err() <-chan error { return make(chan error) }

type TestHeaderLoader struct {
	mock.Mock
	NewHeaders chan<- *types.Header
}

func (thl *TestHeaderLoader) HeaderByHash(ctx context.Context, hash common.Hash) (*types.Header, error) {
	args := thl.Called(hash)
	return args.Get(0).(*types.Header), args.Error(1)
}

func (thl *TestHeaderLoader) HeaderByNumber(ctx context.Context, number *big.Int) (*types.Header, error) {
	var args mock.Arguments
	if number == nil {
		args = thl.Called(nil)
	} else {
		args = thl.Called(*number)
	}
	return args.Get(0).(*types.Header), args.Error(1)
}

func (thl *TestHeaderLoader) SubscribeNewHead(ctx context.Context, ch chan<- *types.Header) (ethereum.Subscription, error) {
	thl.NewHeaders = ch
	return &TestSubscription{}, nil
}

func Test_HeaderCache(t *testing.T) {
	cache := syncer.NewHeaderCache(3)
	headers := makeHeaderChain(5, 0)

	assert.True(t, cache.Insert(headers[0]))
	assert.True(t, cache.Insert(headers[1]))
	assert.True(t, cache.Insert(headers[2]))

	// Oldest header should still be in cache
	header0Item, exists := cache.Get(headers[0].Hash())
	assert.True(t, exists)
	assert.Equal(t, header0Item.Header, headers[0])
	assert.False(t, header0Item.Forwarded)

	// Re-insert an existing header
	assert.True(t, cache.Insert(headers[0]))

	// This should prune header 0 and only header 0
	assert.True(t, cache.Insert(headers[3]))
	header0Item, exists = cache.Get(headers[0].Hash())
	assert.Nil(t, header0Item)
	assert.False(t, exists)
	header1Item, _ := cache.Get(headers[1].Hash())
	assert.NotNil(t, header1Item)

	// Cannot re-insert a pruned header
	assert.False(t, cache.Insert(headers[0]))
	header0Item, _ = cache.Get(headers[0].Hash())
	assert.Nil(t, header0Item)

	// Insert sibling headers
	var header1Sib = &types.Header{
		Number:     headers[1].Number,
		Nonce:      types.EncodeNonce(432),
		ParentHash: headers[1].ParentHash,
	}
	assert.True(t, cache.Insert(header1Sib))

	// Prune multiple headers
	assert.True(t, cache.Insert(headers[4]))
	header1Item, _ = cache.Get(headers[1].Hash())
	header1SibItem, _ := cache.Get(header1Sib.Hash())
	assert.Nil(t, header1Item)
	assert.Nil(t, header1SibItem)
}

func Test_SyncFromFinalizedHeaderWithCacheGaps(t *testing.T) {
	eg, ctx := errgroup.WithContext(context.Background())
	headers := makeHeaderChain(6, 0)
	headerLoader := TestHeaderLoader{}
	// Sets the latest header
	headerLoader.On("HeaderByNumber", nil).Return(headers[3], nil)
	for _, header := range headers {
		headerLoader.On("HeaderByNumber", *header.Number).Return(header, nil)
		headerLoader.On("HeaderByHash", header.Hash()).Return(header, nil)
	}

	syncer := syncer.NewSyncer(2, &headerLoader)
	headerChannel, _ := syncer.StartSync(ctx, eg, 0)

	// header2 is finalized and should be forwarded first
	header2 := <-headerChannel
	assert.Equal(t, header2, headers[1])
	headerLoader.AssertNumberOfCalls(t, "HeaderByNumber", 2)
	headerLoader.AssertCalled(t, "HeaderByNumber", *big.NewInt(1))

	// This should trigger header 3, 4 and 5 to be forwarded. 3 and 4
	// will be missing from cache and thus fetched using HeaderByHash
	headerLoader.NewHeaders <- headers[4]
	header3 := <-headerChannel
	header4 := <-headerChannel
	header5 := <-headerChannel
	assert.Equal(t, header3, headers[2])
	assert.Equal(t, header4, headers[3])
	assert.Equal(t, header5, headers[4])
	headerLoader.AssertNumberOfCalls(t, "HeaderByHash", 2)
	headerLoader.AssertCalled(t, "HeaderByHash", header3.Hash())
	headerLoader.AssertCalled(t, "HeaderByHash", header4.Hash())

	// This should only forward header 6 since ancestors were
	// forwarded above
	headerLoader.NewHeaders <- headers[5]
	header6 := <-headerChannel
	assert.Equal(t, header6, headers[5])
	headerLoader.AssertNumberOfCalls(t, "HeaderByHash", 2)
}

func Test_SyncFromUnfinalizedHeader(t *testing.T) {
	eg, ctx := errgroup.WithContext(context.Background())
	headers := makeHeaderChain(5, 0)
	headerLoader := TestHeaderLoader{}
	// Sets the latest header
	headerLoader.On("HeaderByNumber", nil).Return(headers[3], nil)
	for _, header := range headers {
		headerLoader.On("HeaderByHash", header.Hash()).Return(header, nil)
	}

	syncer := syncer.NewSyncer(2, &headerLoader)
	headerChannel, _ := syncer.StartSync(ctx, eg, 1)

	// Give syncer time to determine that there aren't any finalized
	// headers to forward
	time.Sleep(100 * time.Millisecond)

	// This should trigger header 2, 3, 4 to be forwarded (header 2 is at
	// the syncer init height, but it must be forwarded again since
	// the height isn't final until header 4)
	headerLoader.NewHeaders <- headers[3]
	header2 := <-headerChannel
	header3 := <-headerChannel
	header4 := <-headerChannel
	assert.Equal(t, header2, headers[1])
	assert.Equal(t, header3, headers[2])
	assert.Equal(t, header4, headers[3])
}

func Test_SyncForwardsMultipleForks(t *testing.T) {
	eg, ctx := errgroup.WithContext(context.Background())
	headersChain1 := makeHeaderChain(5, 0)
	headersChain2 := makeHeaderChain(5, 1)
	headerLoader := TestHeaderLoader{}
	// Sets the latest header
	headerLoader.On("HeaderByNumber", nil).Return(headersChain1[3], nil)
	headerLoader.On("HeaderByNumber", *big.NewInt(1)).Return(headersChain1[1], nil)
	for i, _ := range headersChain1 {
		headerLoader.On("HeaderByHash", headersChain1[i].Hash()).Return(headersChain1[i], nil)
		headerLoader.On("HeaderByHash", headersChain2[i].Hash()).Return(headersChain2[i], nil)
	}

	syncer := syncer.NewSyncer(2, &headerLoader)
	headerChannel, _ := syncer.StartSync(ctx, eg, 0)
	<-headerChannel

	// Forward a fork
	headerLoader.NewHeaders <- headersChain1[4]
	time.Sleep(100 * time.Millisecond)
	assert.Equal(t, 3, len(headerChannel))

	// Forward a different fork
	headerLoader.NewHeaders <- headersChain2[4]
	time.Sleep(100 * time.Millisecond)
	assert.Equal(t, 5, len(headerChannel))
}
