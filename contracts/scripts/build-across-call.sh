# !/bin/bash
set -eu

# Build calldata for depositing 0.001 WETH from Sepolia to Base via Across
# Input token: Sepolia WETH (0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14)
# Output token: Base WETH (0x4200000000000000000000000000000000000006)
# Origin chain ID: Sepolia (11155111)
# Destination chain ID: Base (84532)
# Amount: 0.001 WETH (1,000,000,000,000,000 in 18 decimal places)
# Depositor: 0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd

depositor="0x000000000000000000000000302f0b71b8ad3cf6dd90adb668e49b2168d652fd"
recipient="0x000000000000000000000000302f0b71b8ad3cf6dd90adb668e49b2168d652fd"
inputToken="0x000000000000000000000000fFf9976782d46CC05630D1f6eBAb18b2324d6B14"
outputToken="0x0000000000000000000000004200000000000000000000000000000000000006"
inputAmount=1100000000000000
minOutputAmount=1000000000000000
destinationChainId=84532
exclusiveRelayer="0x0000000000000000000000000000000000000000000000000000000000000000"
currentTimestamp=$(date +%s)
# quoteTimestamp is set to current time - 10 minutes to ensure validity
quoteTimestamp=$((currentTimestamp-600))
# fillDeadline is set to current time + 10 minutes to ensure sufficient time to fill
fillDeadline=$((currentTimestamp+600))
exclusivityParameter=0
message="0x"

calldata=$(cast calldata "deposit(bytes32,bytes32,bytes32,bytes32,uint256,uint256,uint256,bytes32,uint32,uint32,uint32,bytes)" \
$depositor $recipient $inputToken $outputToken $inputAmount $minOutputAmount $destinationChainId $exclusiveRelayer $quoteTimestamp $fillDeadline $exclusivityParameter $message
)

echo "Calldata for Across deposit:"
echo "$calldata"

recipient_raw="0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd"
inputToken_raw="0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14"
outputToken_raw="0x4200000000000000000000000000000000000006"
calldata=$(cast calldata "swapToken((address,address,uint256,uint256,uint256),address)" \
"($inputToken_raw,$outputToken_raw,$inputAmount,$minOutputAmount,$destinationChainId)" $recipient_raw
)
echo "Calldata for SnowbridgeL1Adaptor:"
echo "$calldata"
