# Basic Channel

_Note: This component is obsolete, and has been replaced with a new design. Docs will follow._&#x20;

The basic channel offers guaranteed deliverability, but not guaranteed delivery. See [here](../overview.md#deliverability-and-delivery) for more explanation. Users or teams are responsible for relaying their messages at their own cost with the relaying software we provide. The channel enforces message ordering at the account level. For example, messages from user Alice will be delivered in the order they were sent.

This channel is primarily used to bootstrap the incentivized channel. However we can foresee it being used by third-party teams who wish to bypass our incentivized channel and build their own relaying fabric.

## Protocol Objects

### Messages inbound from Ethereum

```rust
pub struct Envelope {
    /// The address of the outbound channel on Ethereum that forwarded this message.
    pub channel: H160,
    /// The application on Ethereum where the message originated from.
    pub source: H160,
    /// A nonce for enforcing replay protection and ordering.
    pub nonce: u64,
    /// declared weight of payload. Used to calculate transaction fee.
    pub weight: u64,
    /// The inner payload generated from the source application.
    pub payload: Vec<u8>,
}
```

### Messages inbound from Polkadot

```rust
struct MessageBundle {
    // ID of app pallet on parachain.
    uint8 sourceChannelID;
    // SR-25519 public key of user who submitted messages 
    bytes32 account;
    // A nonce for enforcing replay protection and ordering.
    uint64 nonce;
    // All messages submitted by account in this commitment period. 
    Message[] messages;  
}

struct Message {
    // each message has a unique id distinct from the bundle nonce  
    uint64 id;
    // target smart contract
    address target;
    // payload to dispatch to target
    bytes payload;
}
```

##
