echo "Rebuilding and killing relay services"

configdir=/tmp/snowbridge

address_for()
{
    jq -r .contracts."${1}".address $configdir/contracts.json
}

# Build relay services
mage -d ../relayer build

# Configure beefy relay
jq \
    --arg k1 "$(address_for BeefyLightClient)" \
'
    .sink.contracts.BeefyLightClient = $k1
' \
config/beefy-relay.json > $configdir/beefy-relay.json

# Configure parachain relay
jq \
    --arg k1 "$(address_for BasicInboundChannel)" \
    --arg k2 "$(address_for IncentivizedInboundChannel)" \
    --arg k3 "$(address_for BeefyLightClient)" \
'
    .source.contracts.BasicInboundChannel = $k1
| .source.contracts.IncentivizedInboundChannel = $k2
| .source.contracts.BeefyLightClient = $k3
| .sink.contracts.BasicInboundChannel = $k1
| .sink.contracts.IncentivizedInboundChannel = $k2
' \
config/parachain-relay.json > $configdir/parachain-relay.json

# Configure ethereum relay
jq \
    --arg k1 "$(address_for BasicOutboundChannel)" \
    --arg k2 "$(address_for IncentivizedOutboundChannel)" \
'
    .source.contracts.BasicOutboundChannel = $k1
| .source.contracts.IncentivizedOutboundChannel = $k2
' \
config/ethereum-relay.json > $configdir/ethereum-relay.json

kill $(ps -aux | grep -e snowbridge-relay | awk '{print $2}') -9 || true
