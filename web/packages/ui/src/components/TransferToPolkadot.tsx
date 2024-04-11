import { AppProps, EthWalletInfo, TransactionHistoryItem, TransactionStatus } from './Common';

import { toPolkadot } from '@snowbridge/api';
import { ethers } from 'ethers';
import { ChangeEvent, FormEvent, useState } from 'react';

type TransferInfo = {
  tokenAddress: string,
  beneficiary: string,
  amount: bigint,
  transferInProgress: boolean,
  destinationChain: number,
  destinationFee: bigint
  result?: toPolkadot.SendResult,
}

interface TransferToPolkadotProps extends AppProps {
  wallet: EthWalletInfo
  setWallet: React.Dispatch<React.SetStateAction<EthWalletInfo>>
  addTransaction: (item: TransactionHistoryItem) => void
}

export function TransferToPolkadot(props: TransferToPolkadotProps): JSX.Element {
  let { context, config, wallet, setWallet } = props
  let [transferInfo, setTransferInfo] = useState<TransferInfo>({
    tokenAddress: '',
    beneficiary: '',
    amount: BigInt(0),
    transferInProgress: false,
    destinationFee: BigInt(0),
    destinationChain: config.ASSET_HUB_PARAID
  })
  let [errors, setErrors] = useState<string[]>([]);
  let [statusUpdates, setStatusUpdates] = useState<string[]>([]);

  if (context === undefined) {
    return (<div>Loading...</div>)
  }

  const walletConnect = async () => {
    if (window.ethereum === undefined || window.ethereum === null) {
      console.log("MetaMask not installed; using read-only defaults")
      setWallet({ isConnected: false, hasError: true, error: 'No wallet found.' })
    } else {
      const provider = new ethers.BrowserProvider(window.ethereum)
      const network = await provider?.getNetwork()
      const signer = await provider.getSigner();
      if (context === undefined) {
        let error = 'Context disconnected.'
        setWallet({ isConnected: true, hasError: true, signer, provider, network, error })
        return
      }
      const c = await context.ethereum.api.getNetwork();
      if (c.chainId === network.chainId) {
        setWallet({ isConnected: true, hasError: false, signer, provider, network })
      } else {
        let error = `Connected chainId is '${network.chainId.toString()}'. Bridged chainId is '${c.chainId.toString()}'.`
        setWallet({ isConnected: true, hasError: true, signer, provider, network, error })
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
    if (wallet.isConnected && !wallet.hasError && context !== undefined && wallet.signer !== undefined) {
      setErrors([])
      statusUpdates = []
      setStatusUpdates(statusUpdates)
      setTransferInfo({ ...transferInfo, result: undefined, transferInProgress: false })

      try {
        const plan = await toPolkadot.validateSend(context,
          wallet.signer,
          transferInfo.beneficiary,
          transferInfo.tokenAddress,
          transferInfo.destinationChain,
          transferInfo.amount,
          transferInfo.destinationFee
        );
        if (plan.failure) {
          let errors: string[] = []
          if (!plan.failure.bridgeOperational) errors.push('Bridge halted.')
          if (!plan.failure.channelOperational) errors.push('Channel to destination halted.')
          if (!plan.failure.beneficiaryAccountExists) errors.push(`'${transferInfo.beneficiary}' does not exist on destination.`)
          if (!plan.failure.tokenIsValidERC20) errors.push(`Token '${transferInfo.tokenAddress}' not a valid ERC20 token.`)
          if (!plan.failure.tokenIsRegistered) errors.push(`Token '${transferInfo.tokenAddress}' not registered with the Snowbridge gateway.`)
          if (!plan.failure.foreignAssetExists) errors.push(`Token '${transferInfo.tokenAddress}' not registered on Asset Hub.`)
          if (!plan.failure.hasToken) errors.push(`Source address '${await wallet.signer?.getAddress()}' does not own token '${transferInfo.tokenAddress}'.`)
          if (!plan.failure.tokenSpendApproved) errors.push(`Source address '${await wallet.signer?.getAddress()}' has not allowed Snowbridge gateway '${config.GATEWAY_CONTRACT}' to spend token '${transferInfo.tokenAddress}'.`)
          if (!plan.failure.lightClientLatencyIsAcceptable) errors.push('Light client is too far behind.')
          if (!plan.failure.canPayFee) errors.push('Cannot pay fee.')
          if (!plan.failure.destinationChainExists) errors.push('Destination chain does not exist.')
          if (!plan.failure.hrmpChannelSetup) errors.push('HRMP channel is not set uo.')
          setErrors(errors)
          return;
        }
        setTransferInfo({ ...transferInfo, result: undefined, transferInProgress: true })
        statusUpdates.push('Submitting...')
        setStatusUpdates(statusUpdates)
        const result = await toPolkadot.send(context, wallet.signer, plan)
        props.addTransaction({
          when: new Date().toISOString(),
          type: 'toPolkadot',
          status: result.failure !== undefined ? TransactionStatus.Failed : TransactionStatus.InProgress,
          messages: [],
          result: result
        })
        if (result.failure) {
          setErrors(['Transaction failed ' + result.failure.receipt])
        } else {
          setTransferInfo({ ...transferInfo, result, transferInProgress: true })
          statusUpdates[0] = `Transaction submitted ${result.success?.ethereum.transactionHash}.`
            + `Waiting for block ${result.success?.ethereum.blockNumber.toString()} to be included by the light client.`
          setStatusUpdates(statusUpdates)
          for await (const update of toPolkadot.trackSendProgress(context, result)) {
            statusUpdates.push(update)
            setStatusUpdates(statusUpdates)
          }
        }
      } catch (error: any) {
        setErrors([error.message])
      }
      setTransferInfo({ ...transferInfo, transferInProgress: false })
    } else {
      setErrors(['Wallet not connected.'])
    }
  }

  if (wallet.hasError) {
    return (<p style={{ color: 'red' }}>Error: {wallet.error || 'Unknown'}</p>)
  }
  else if (wallet.isConnected) {
    return (
      <div>
        <h2>Transfer asset from Ethereum to Asset Hub.</h2>
        Connected to <span>{wallet.network?.name}</span> network(<span>{wallet.network?.chainId.toString()}</span>) with address:
        <pre>{wallet.signer?.address}</pre>
        <form onSubmit={handleSubmit}>
          <label htmlFor='tokenAddress'>Token Address:</label>
          <input type='text'
            placeholder='0x0000000000000000000000000000000000000000'
            required
            name='tokenAddress'
            value={transferInfo.tokenAddress}
            onChange={handleChange} />
          <label htmlFor='benificiary'>Beneficiary:</label>
          <input type='text'
            placeholder='SS58 or Raw Address'
            required
            name='beneficiary'
            value={transferInfo.beneficiary}
            onChange={handleChange} />
          <label htmlFor='amount'>Amount:</label>
          <input type='number'
            placeholder='0'
            required
            name='amount'
            value={transferInfo.amount.toString()}
            onChange={handleChange} />
          <label htmlFor='destinationChain'>Destination Parachain:</label>
          <input type='number'
            placeholder='1000'
            required
            name='destinationChain'
            value={transferInfo.destinationChain.toString()}
            onChange={handleChange} />
          <label htmlFor='destinationFee'>Destination Fee:</label>
          <input
            type='number'
            placeholder='0'
            required
            name='destinationFee'
            value={transferInfo.destinationFee.toString()}
            onChange={handleChange} />
          <button disabled={transferInfo.transferInProgress} type='submit'>Send</button>
          <p hidden={transferInfo.result === undefined} style={{ gridColumn: 'span 2' }}>TxHash: {transferInfo.result?.success?.ethereum.transactionHash}</p>
          <ul hidden={errors.length === 0} style={{ color: 'red', gridColumn: 'span 2' }}>
            {errors.map((error, index) => (<li key={index}>{error}</li>))}
          </ul>
          <ul hidden={statusUpdates.length === 0} style={{ color: 'green', gridColumn: 'span 2' }}>
            {statusUpdates.map((update, index) => (<li key={index}>{update}</li>))}
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