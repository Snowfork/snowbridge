import { ChangeEvent, FormEvent, useState } from 'react';
import './App.css';
import { BrowserProvider, JsonRpcSigner, Network, assertArgumentCount, ethers } from 'ethers';
import { contextFactory, planSendToken, doSendToken, trackSendToken, Context } from '@snowbridge/api'

const ETHEREUM_WS_API = 'ws://127.0.0.1:8546'
const RELAY_CHAIN_WS_URL = 'ws://127.0.0.1:9944'
const ASSET_HUB_WS_URL = 'ws://127.0.0.1:12144'
const BRIDGE_HUB_WS_URL = 'ws://127.0.0.1:11144'
const GATEWAY_CONTRACT = '0xEDa338E4dC46038493b885327842fD3E301CaB39'
const BEEFY_CONTRACT = '0x992B9df075935E522EC7950F37eC8557e86f6fdb'

type WalletInfo = {
  isConnected: boolean,
  hasError: boolean,
  error?: string,
  signer?: JsonRpcSigner,
  provider?: BrowserProvider,
  network?: Network,
  context?: Context,
}

function MyForm() {
  let [walletInfo, setWalletInfo] = useState<WalletInfo>({
    isConnected: false,
    hasError: false,
  })
  let [transferInfo, setTransferInfo] = useState({
    tokenAddress: '',
    beneficiary: '',
    amount: BigInt(0),
  })

  const walletConnect = async () => {
    if (window.ethereum === undefined || window.ethereum === null) {
      console.log("MetaMask not installed; using read-only defaults")
      setWalletInfo({ isConnected: false, hasError: true, error: 'No wallet found.' })
    } else {
      const provider = new ethers.BrowserProvider(window.ethereum)
      const network = await provider?.getNetwork()
      const signer = await provider.getSigner();
      const context = await contextFactory({
        ethereum: { url: ETHEREUM_WS_API },
        polkadot: {
          url: {
            bridgeHub: BRIDGE_HUB_WS_URL,
            assetHub: ASSET_HUB_WS_URL,
            relaychain: RELAY_CHAIN_WS_URL,
          },
        },
        appContracts: {
          gateway: GATEWAY_CONTRACT,
          beefy: BEEFY_CONTRACT,
        },
      })
      const c = await context.ethereum.api.getNetwork();
      if (c.chainId === network.chainId) {
        setWalletInfo({ isConnected: true, hasError: false, signer, provider, network })
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
    if(walletInfo.isConnected && !walletInfo.hasError && walletInfo.context && walletInfo.signer) {
      const result = await planSendToken(walletInfo.context, 
        walletInfo.signer,
        transferInfo.beneficiary,
        transferInfo.tokenAddress,
        transferInfo.amount
      );
    }
    console.log('submit')
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
    </div>
  );
}

export default App;
