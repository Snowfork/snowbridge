import { ChangeEvent, FormEvent, useState } from 'react';
import './App.css';
import { BrowserProvider, JsonRpcSigner, Network, ethers } from 'ethers';
import { contextFactory, planSendToken, doSendToken, trackSendToken, Context } from '@snowbridge/api'

let config = {
  ETHEREUM_WS_API: 'ws://127.0.0.1:8546',
  RELAY_CHAIN_WS_URL: 'ws://127.0.0.1:9944',
  ASSET_HUB_WS_URL: 'ws://127.0.0.1:12144',
  BRIDGE_HUB_WS_URL: 'ws://127.0.0.1:11144',
  GATEWAY_CONTRACT: '0xEDa338E4dC46038493b885327842fD3E301CaB39',
  BEEFY_CONTRACT: '0x992B9df075935E522EC7950F37eC8557e86f6fdb',
}
if (process.env.NODE_ENV === 'production') {
  config = {
    ETHEREUM_WS_API: `wss://sepolia.infura.io/ws/v3/${process.env.REACT_APP_INFURA_KEY}`,
    RELAY_CHAIN_WS_URL: 'wss://rococo-rpc.polkadot.io',
    ASSET_HUB_WS_URL: 'wss://rococo-asset-hub-rpc.polkadot.io',
    BRIDGE_HUB_WS_URL: 'wss://rococo-bridge-hub-rpc.polkadot.io',
    GATEWAY_CONTRACT: '0x5b4909ce6ca82d2ce23bd46738953c7959e710cd',
    BEEFY_CONTRACT: '0x27e5e17ac995d3d720c311e1e9560e28f5855fb1',
  }
}

type WalletInfo = {
  isConnected: boolean,
  hasError: boolean,
  error?: string,
  signer?: JsonRpcSigner,
  provider?: BrowserProvider,
  network?: Network,
  context?: Context,
}

type TransferInfo = {
    tokenAddress: string,
    beneficiary: string,
    amount: bigint,
}

function MyForm() {
  let [walletInfo, setWalletInfo] = useState<WalletInfo>({
    isConnected: false,
    hasError: false,
  })
  let [transferInfo, setTransferInfo] = useState<TransferInfo>({
    tokenAddress: '',
    beneficiary: '',
    amount: BigInt(0),
  })
  let [errors, setErrors] = useState<string[]>([]);
  let [statusUpdates, setStatusUpdates] = useState<string[]>([]);

  const walletConnect = async () => {
    if (window.ethereum === undefined || window.ethereum === null) {
      console.log("MetaMask not installed; using read-only defaults")
      setWalletInfo({ isConnected: false, hasError: true, error: 'No wallet found.' })
    } else {
      const provider = new ethers.BrowserProvider(window.ethereum)
      const network = await provider?.getNetwork()
      const signer = await provider.getSigner();
      const context = await contextFactory({
        ethereum: { url: config.ETHEREUM_WS_API },
        polkadot: {
          url: {
            bridgeHub: config.BRIDGE_HUB_WS_URL,
            assetHub: config.ASSET_HUB_WS_URL,
            relaychain: config.RELAY_CHAIN_WS_URL,
          },
        },
        appContracts: {
          gateway: config.GATEWAY_CONTRACT,
          beefy: config.BEEFY_CONTRACT,
        },
      })
      const c = await context.ethereum.api.getNetwork();
      if (c.chainId === network.chainId) {
        setWalletInfo({ isConnected: true, hasError: false, signer, provider, network, context })
      } else {
        let error = `Connected chainId is '${network.chainId.toString()}'. Bridged chainId is '${c.chainId.toString()}'.`
        setWalletInfo({ isConnected: true, hasError: true, signer, provider, network, error })
      }
    }
  }

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setTransferInfo({
      ...transferInfo,
      [name]: value
    });
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    console.log(walletInfo.context !== undefined, walletInfo.signer !== undefined)
    if(walletInfo.isConnected && !walletInfo.hasError && walletInfo.context !== undefined && walletInfo.signer !== undefined) {
      setErrors([])
      setStatusUpdates([])

      const plan = await planSendToken(walletInfo.context, 
        walletInfo.signer,
        transferInfo.beneficiary,
        transferInfo.tokenAddress,
        transferInfo.amount
      );
      if(plan.failure) {
        let errors: string[] = []
        if (!plan.failure.bridgeOperational) errors.push('Bridge halted.')
        if (!plan.failure.channelOperational) errors.push('Channel to destination halted.')
        if (!plan.failure.destinationAccountExists) errors.push(`'${transferInfo.beneficiary}' does not exist on destination.`)
        if (!plan.failure.tokenIsValidERC20) errors.push(`Token '${transferInfo.tokenAddress}' not a valid ERC20 token.`)
        if (!plan.failure.tokenIsRegistered) errors.push(`Token '${transferInfo.tokenAddress}' not registered with the Snowbridge gateway.`)
        if (!plan.failure.foreignAssetExists) errors.push(`Token '${transferInfo.tokenAddress}' not registered on Asset Hub.`)
        if (!plan.failure.hasToken) errors.push(`Source address '${await walletInfo.signer?.getAddress()}' does not own token '${transferInfo.tokenAddress}'.`)
        if (!plan.failure.tokenSpendApproved) errors.push(`Source address '${await walletInfo.signer?.getAddress()}' has not allowed Snowbridge gateway '${config.GATEWAY_CONTRACT}' to spend token '${transferInfo.tokenAddress}'.`)
        if (!plan.failure.lightClientLatencyIsAcceptable) errors.push('Light client is too far behind.')
        setErrors(errors)
        return;
      }
      try {
        const result = await doSendToken(walletInfo.context, walletInfo.signer, plan)
        for await (const update of trackSendToken(walletInfo.context, result)) {
          setStatusUpdates([update, ...statusUpdates])
        }
      } catch(error: any) {
        setErrors(error.message)
        return;
      }
    } else {
      setErrors(['Wallet not connected.'])
    }
  }

  if (walletInfo.hasError) {
    return (<p style={{ color: 'red' }}>Error: {walletInfo.error || 'Unknown'}</p>)
  }
  else if (walletInfo.isConnected) {
    return (
      <div>
        Connected to <span>{walletInfo.network?.name}</span> network(<span>{walletInfo.network?.chainId.toString()}</span>) with address:
        <pre>{walletInfo.signer?.address}</pre>
        <form onSubmit={handleSubmit}>
          <label>Token Address:</label>
          <input type='text'
            placeholder='0x0000000000000000000000000000000000000000'
            required
            name='tokenAddress'
            value={transferInfo.tokenAddress}
            onChange={handleChange}/>
          <label>beneficiary:</label>
          <input type='text'
            placeholder='SS58 or Raw Address'
            required
            name='beneficiary'
            value={transferInfo.beneficiary}
            onChange={handleChange}/>
          <label>Amount:</label>
          <input type='number'
            placeholder='0'
            required
            name='amount'
            value={transferInfo.amount.toString()}
            onChange={handleChange}/>
          <button type='submit'>Send</button>
          <ul style={{color: 'red', gridColumn: 'span 2'}}>
            {errors.map(error => (<li>{error}</li>))}
          </ul>
          <ul style={{color: 'green', gridColumn: 'span 2'}}>
            {statusUpdates.map(update => (<li>{update}</li>))}
          </ul>
        </form>
      </div>
    )
  }
  else {
    return (
      <button onClick={walletConnect}>Connect Wallet</button>
    )
  }
}

function App() {
  return (
    <div className='App'>
      <h1>Snowbridge</h1>
      <p>Transfer asset from Ethereum to Asset Hub</p>
      <MyForm />
      <div style={{textAlign:'right'}}>
          <sub>Snowfork 2024 - {process.env.NODE_ENV} build</sub>
      </div>
    </div>
  );
}

export default App;
