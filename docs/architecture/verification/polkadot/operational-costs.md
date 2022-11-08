# Operational Costs

To remain operational, the BEEFY light client must be updated with new BEEFY commitments. These commitments are emitted periodically by the relay chain, roughly every minute. A mandatory commitment is emitted at the start of every validator [session](https://wiki.polkadot.network/docs/maintain-polkadot-parameters#periods-of-common-actions-and-attributes), and must be provided to the light client.

It will be prohibitively expensive to submit updates very minute. So we envision that the rate of updates will be dynamic and influenced by user demand. Assuming current gas prices, the cost of operating the the BEEFY client should be between $50,000 and $1,500,000 per year. For detailed calculations, see our [Cost Predictions](https://docs.google.com/spreadsheets/d/1QtxNtG4GE1IUaH204QFO6lObyAqLV9WCbmSYEopU18Q/edit?usp=sharing).

Our current client implementation is it not very optimized, as we have focused foremost on correctness and readability. We have identified several optimizations which can bring down this cost by around 20% or more.

Once BEEFY and Ethereum support BLS signatures, costs should fall dramatically  (ETA 2024).
