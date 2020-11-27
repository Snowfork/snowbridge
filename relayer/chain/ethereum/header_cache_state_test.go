package ethereum_test

import (
	"testing"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/tranvictor/ethashproof"
	"golang.org/x/sync/errgroup"
)

type TestCacheLoader struct {
	mock.Mock
}

func (tl *TestCacheLoader) MakeCache(epoch uint64) (*ethashproof.DatasetMerkleTreeCache, error) {
	args := tl.Called(epoch)
	return args.Get(0).(*ethashproof.DatasetMerkleTreeCache), args.Error(1)
}

func TestHeaderCacheState(t *testing.T) {
	eg := &errgroup.Group{}
	cacheLoader := TestCacheLoader{}
	cacheLoader.On("MakeCache", uint64(0)).Return(&ethashproof.DatasetMerkleTreeCache{Epoch: 0}, nil)
	cacheLoader.On("MakeCache", uint64(1)).Return(&ethashproof.DatasetMerkleTreeCache{Epoch: 1}, nil)
	cacheLoader.On("MakeCache", uint64(2)).Return(&ethashproof.DatasetMerkleTreeCache{Epoch: 2}, nil)
	cacheLoader.On("MakeCache", uint64(3)).Return(&ethashproof.DatasetMerkleTreeCache{Epoch: 3}, nil)

	// Should load epoch 0 and 1 caches
	hcs, err := ethereum.NewHeaderCacheState(eg, 0, &cacheLoader)
	if err != nil {
		panic(err)
	}
	err = eg.Wait()
	if err != nil {
		panic(err)
	}

	cacheLoader.AssertCalled(t, "MakeCache", uint64(0))
	cacheLoader.AssertCalled(t, "MakeCache", uint64(1))
	cacheLoader.AssertNumberOfCalls(t, "MakeCache", 2)

	// No new cache data needs to be loaded
	cache := getCacheAndWait(eg, hcs, 29999)
	assert.Equal(t, cache.Epoch, uint64(0))
	cacheLoader.AssertNumberOfCalls(t, "MakeCache", 2)

	// Should trigger epoch 2 to be loaded
	cache = getCacheAndWait(eg, hcs, 30000)
	assert.Equal(t, cache.Epoch, uint64(1))
	cacheLoader.AssertCalled(t, "MakeCache", uint64(2))
	cacheLoader.AssertNumberOfCalls(t, "MakeCache", 3)

	// Should trigger epoch 0 to be loaded again
	cache = getCacheAndWait(eg, hcs, 29999)
	assert.Equal(t, cache.Epoch, uint64(0))
	cacheLoader.AssertNumberOfCalls(t, "MakeCache", 4)

	// Should trigger epoch 2 and 3 to be loaded
	cache = getCacheAndWait(eg, hcs, 60000)
	assert.Equal(t, cache.Epoch, uint64(2))
	cacheLoader.AssertCalled(t, "MakeCache", uint64(3))
	cacheLoader.AssertNumberOfCalls(t, "MakeCache", 6)
}

func getCacheAndWait(
	eg *errgroup.Group,
	hcs *ethereum.HeaderCacheState,
	blockNumber uint64,
) *ethashproof.DatasetMerkleTreeCache {
	cache, err := hcs.GetEthashproofCache(blockNumber)
	if err != nil {
		panic(err)
	}

	err = eg.Wait()
	if err != nil {
		panic(err)
	}

	return cache
}
