/* eslint-disable no-use-before-define */

let blockNumber = (await api.rpc.chain.getHeader()).number.toNumber();

let storage = {
  ForeignAssets: {
    Account: [
      [
        [
          {
            parents: 2,
            interior: {
              X2: [
                {
                  GlobalConsensus: {
                    Ethereum: {
                      chain_id: 1,
                    },
                  },
                },
                {
                  AccountKey20: {
                    key: "0x6B175474E89094C44Da98b954EedeAC495271d0F",
                  },
                },
              ],
            },
          },
          "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        ],
        { balance: "1000000000000" },
      ],
    ],
    Asset: [
      [
        [
          {
            parents: 2,
            interior: {
              X2: [
                {
                  GlobalConsensus: {
                    Ethereum: {
                      chain_id: 1,
                    },
                  },
                },
                {
                  AccountKey20: {
                    key: "0x6B175474E89094C44Da98b954EedeAC495271d0F",
                  },
                },
              ],
            },
          },
        ],
        {
          owner: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
          issuer: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
          admin: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
          freezer: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
          supply: "1000000000000",
          deposit: 0,
          minBalance: "1000000000",
          isSufficient: false,
          accounts: 1,
          sufficients: 1,
          approvals: 0,
          status: {
            Live: {},
          },
        },
      ],
    ],
  },
};

await api.rpc("dev_setStorage", storage);

await api.rpc("dev_newBlock", { count: 1 });
