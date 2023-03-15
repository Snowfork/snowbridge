


struct WoopMessage {
    dest: MultiLocation,
    action: Action,
}

enum Action {
    NativeTokens(NativeTokensAction),
}

enum NativeTokensAction {
    Create {
        token: H160,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8
    },
    Mint {
        token: H160,
        recipient: MultiLocation, // Recipient of funds on final destination
        amount: u128,
        forward: Option<MultiLocation> // Optional location of a final parachain to forward funds
    } 
}
