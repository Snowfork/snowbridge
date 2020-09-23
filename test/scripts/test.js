let { ApiPromise, WsProvider } = require('@polkadot/api');


let main = async () => {

    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({
        provider,
        types: {
            Address: 'AccountId',
            LookupSource: 'AccountId',
            AppId: '[u8; 20]',
            Message: {
                payload: 'Vec<u8>',
                verification: 'VerificationInput'
            },
            VerificationInput: {
                _enum: {
                    Basic: 'VerificationBasic',
                    None: null
                }
            },
            VerificationBasic: {
                blockNumber: 'u64',
                eventIndex: 'u32'
            },
            TokenId: 'H160',
            BridgedAssetId: 'H160',
            AssetAccountData: {
                free: 'U256'
            }
        }
    })

    api.query.system.events((events) => {
        console.log(`\nReceived ${events.length} events:`);

        events.forEach((record) => {
          const { event, phase } = record;
          const types = event.typeDef;

          console.log(`\t${event.section}:${event.method}:: (phase=${phase.toString()})`);
          console.log(`\t\t${event.meta.documentation.toString()}`);

          event.data.forEach((data, index) => {
            console.log(`\t\t\t${types[index].type}: ${data.toString()}`);
          });
        });
      });
}

main().catch((error) => {
    console.error(error)
    process.exit(-1)
})
