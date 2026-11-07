package executionV3

import "testing"

func TestV1Enabled(t *testing.T) {
	var c Config
	if c.V1Enabled() {
		t.Fatal("empty channel-id should disable V1")
	}
	c.Source.ChannelID[0] = 1
	if !c.V1Enabled() {
		t.Fatal("non-zero channel-id should enable V1")
	}
}
