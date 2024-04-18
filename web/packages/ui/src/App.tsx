import './App.css';
import { getConfig } from './Config';
import { BridgeStatus, EthWalletInfo, SubWalletInfo, TransactionHistoryItem, TransactionHistoryStore } from './components/Common';
import { Status } from './components/Status';
import { TransactionHistory } from './components/TransactionHistory';
import { TransferToEthereum } from './components/TransferToEthereum';
import { TransferToPolkadot } from './components/TransferToPolkadot';

import { Context, contextFactory } from '@snowbridge/api';
import { useEffect, useState } from 'react';
import { BrowserRouter, HashRouter, NavLink, Redirect, Route, Switch } from "react-router-dom";

const config = getConfig()

const connectContext = async (): Promise<Context> => {
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
  return context
}

function App() {
  const [context, setContext] = useState<Context>();
  const [error, setError] = useState<string>();
  const [ethWalletInfo, setEthWalletInfo] = useState<EthWalletInfo>({
    isConnected: false,
    hasError: false,
  })
  const [subWalletInfo, setSubWalletInfo] = useState<SubWalletInfo>({
    isConnected: false,
    hasError: false,
  })
  const [history, setHistory] = useState<TransactionHistoryStore>({
    items: []
  })
  const [bridgeStatus, setBridgeStatus] = useState<BridgeStatus>()
  const [updateTimerSeconds, setUpdateTimerSeconds] = useState<Date>(new Date())

  let connectionInProgress = false;
  const startConnect = async () => {
    if (context !== undefined || connectionInProgress) return;
    connectionInProgress = true;
    try {
      const c = await connectContext()
      connectionInProgress = false
      setContext(c)
    }
    catch (error) {
      connectionInProgress = false
      console.error(error);
      if (error instanceof Error) {
        setError(`${error.name}: ${error.message}`)
      }
    }
  }

  useEffect(() => { startConnect() })

  const snowbridgeHistoryKey = "snowbirdge-transaction-history"
  useEffect(() => {
    const transactionHistory = localStorage.getItem(snowbridgeHistoryKey)
    if (transactionHistory !== null) {
      setHistory(JSON.parse(transactionHistory) as TransactionHistoryStore);
    }
  }, [])
  const addTransactionHistory = (item: TransactionHistoryItem) => {
    history.items.push(item)
    localStorage.setItem(snowbridgeHistoryKey, JSON.stringify(history, (_, v) => typeof v === 'bigint' ? v.toString() : v))
    setHistory(history)
  }

  if (error !== undefined) {
    return (
      <div className='App'>
        <h1>Snowbridge</h1>
        <p>{error}</p>
      </div>
    )
  }

  return (
    <HashRouter>
      <div className='App'>
        <h1>Snowbridge</h1>
        <ul className='inline-list'>
          <li key="status"><NavLink to="status" activeClassName="selected">Status</NavLink></li>
          <li key="toPolkadot"><NavLink to="toPolkadot" activeClassName="selected">Transfer to Polkadot</NavLink></li>
          <li key="toEthereum"><NavLink to="toEthereum" activeClassName="selected">Transfer to Ethereum</NavLink></li>
          <li key="history"><NavLink to="history" activeClassName="selected">History</NavLink></li>
        </ul>

        <Switch>
          <Route exact path="/status">
            <Status config={config} context={context} diagnostic={false} bridgeStatus={bridgeStatus} setBridgeStatus={setBridgeStatus} updateDate={updateTimerSeconds} setUpdateDate={setUpdateTimerSeconds} />
          </Route>
          <Route exact path="/diagnostic">
            <Status config={config} context={context} diagnostic={true} bridgeStatus={bridgeStatus} setBridgeStatus={setBridgeStatus} updateDate={updateTimerSeconds} setUpdateDate={setUpdateTimerSeconds} />
          </Route>
          <Route exact path="/toPolkadot">
            <TransferToPolkadot config={config} context={context} wallet={ethWalletInfo} setWallet={setEthWalletInfo} addTransaction={addTransactionHistory} />
          </Route>
          <Route exact path="/toEthereum">
            <TransferToEthereum config={config} context={context} wallet={subWalletInfo} setWallet={setSubWalletInfo} addTransaction={addTransactionHistory} />
          </Route>
          <Route exact path="/history">
            <TransactionHistory config={config} context={context} history={history} setHistory={setHistory} />
          </Route>
          <Redirect from='*' to="status" />
        </Switch>

        <div style={{ textAlign: 'right' }}>
          <sub>Snowfork 2024 - {process.env.REACT_APP_NODE_ENV}</sub>
        </div>
      </div>
    </HashRouter>
  );
}

export default App;
