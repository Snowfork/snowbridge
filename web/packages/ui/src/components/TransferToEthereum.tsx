import { web3Accounts, web3Enable, web3FromAddress } from '@polkadot/extension-dapp';
import { toEthereum } from "@snowbridge/api";
import { ChangeEvent, FormEvent, useState } from "react";
import { AppProps, SubWalletInfo, TransactionHistoryItem, TransactionStatus } from "./Common";

type TransferInfo = {
    sourceAccount: string,
    sourceParachain: number,
    tokenAddress: string,
    beneficiary: string,
    amount: bigint,
    transferInProgress: boolean,
    result?: toEthereum.SendResult,
}

interface TransferToEthereumProps extends AppProps {
    wallet: SubWalletInfo
    setWallet: React.Dispatch<React.SetStateAction<SubWalletInfo>>
    addTransaction: (item: TransactionHistoryItem) => void
}

export const TransferToEthereum = (props: TransferToEthereumProps): JSX.Element => {
    let { context, config, wallet, setWallet } = props
    let [transferInfo, setTransferInfo] = useState<TransferInfo>({
        sourceAccount: '0',
        sourceParachain: config.ASSET_HUB_PARAID,
        tokenAddress: '',
        beneficiary: '',
        amount: BigInt(0),
        transferInProgress: false,
    })
    let [errors, setErrors] = useState<string[]>([]);
    let [statusUpdates, setStatusUpdates] = useState<string[]>([]);

    const walletConnect = async () => {
        const injectedExtensions = await web3Enable('Snowbridge');
        const accounts = await web3Accounts();
        setWallet({ isConnected: true, hasError: false, accounts, injectedExtensions })
    }

    const handleSubmit = async (e: FormEvent) => {
        e.preventDefault();
        if (wallet.isConnected && !wallet.hasError && context !== undefined && wallet.accounts !== undefined && wallet.accounts.length > 0) {
            setErrors([])
            statusUpdates = []
            setStatusUpdates(statusUpdates)
            setTransferInfo({ ...transferInfo, result: undefined, transferInProgress: false })

            try {
                const account = wallet.accounts[Number(transferInfo.sourceAccount)]
                const injector = await web3FromAddress(account.address);
                let walletSigner = { address: account.address, signer: injector.signer }
                const plan = await toEthereum.validateSend(context,
                    walletSigner,
                    transferInfo.sourceParachain,
                    transferInfo.beneficiary,
                    transferInfo.tokenAddress,
                    transferInfo.amount,
                );
                if (plan.failure) {
                    let errors: string[] = []
                    if (!plan.failure.bridgeOperational) errors.push('Bridge halted.')
                    if (!plan.failure.tokenIsValidERC20) errors.push(`Token '${transferInfo.tokenAddress}' not a valid ERC20 token.`)
                    if (!plan.failure.tokenIsRegistered) errors.push(`Token '${transferInfo.tokenAddress}' not registered with the Snowbridge gateway.`)
                    if (!plan.failure.foreignAssetExists) errors.push(`Token '${transferInfo.tokenAddress}' not registered on Asset Hub.`)
                    if (!plan.failure.lightClientLatencyIsAcceptable) errors.push('Light client is too far behind.')
                    if (!plan.failure.canPayFee) errors.push('Cannot pay fee.')
                    if (!plan.failure.hrmpChannelSetup) errors.push('HRMP channel is not set up.')
                    if (!plan.failure.parachainHasPalletXcm) errors.push('Source parachain does not have pallet-xcm.')
                    if (!plan.failure.parachainKnownToContext) errors.push('Source parachain is not known to context.')
                    if (!plan.failure.hasAsset) errors.push('Source account does not have enough asset.')
                    setErrors(errors)
                    return;
                }
                setTransferInfo({ ...transferInfo, result: undefined, transferInProgress: true })
                statusUpdates.push('Submitting...')
                setStatusUpdates(statusUpdates)
                const result = await toEthereum.send(context, walletSigner, plan)
                props.addTransaction({
                    when: new Date().toISOString(),
                    status: result.failure !== undefined ? TransactionStatus.Failed : TransactionStatus.InProgress,
                    type: 'toEthereum',
                    messages: [],
                    result: result
                })
                if (result.failure) {
                    setErrors([`Transaction failed at block ${result.failure.assetHub?.blockHash}`])
                } else {
                    setTransferInfo({ ...transferInfo, result, transferInProgress: true })
                    statusUpdates[0] = `Transaction submitted at block ${result.success?.assetHub.blockHash}.`
                        + `Waiting for block to be included by the light client.`
                    setStatusUpdates(statusUpdates)
                    for await (const update of toEthereum.trackSendProgress(context, result)) {
                        statusUpdates.push(update)
                        setStatusUpdates(statusUpdates)
                    }
                }
            } catch (error: any) {
                setErrors([error.message])
            }
            setTransferInfo({ ...transferInfo, transferInProgress: false })
        }
        else {
            setErrors(['Wallet not connected.'])
        }
    }

    const handleChange = (e: ChangeEvent<HTMLElement>) => {
        if (!(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLSelectElement)) return
        let { name, value } = e.target
        setTransferInfo({
            ...transferInfo,
            [name]: value
        })
    }

    if (context === undefined) {
        return (<div>Loading...</div>)
    }
    if (wallet.hasError) {
        return (<p style={{ color: 'red' }}>Error: {wallet.error || 'Unknown'}</p>)
    }
    else if (wallet.isConnected) {
        return (
            <div>
                <h2>Transfer asset from Asset Hub to Ethereum.</h2>
                <form onSubmit={handleSubmit}>
                    <label htmlFor="sourceParachain">Source Account:</label>
                    <select name="sourceAccount" onChange={handleChange}>
                        {wallet.accounts?.map((account, index) => (<option key={index} value={index}>{account.address} ({account.meta.source})</option>))}
                    </select>
                    <label>Source Parachain:</label>
                    <input type='number'
                        placeholder='1000'
                        required
                        name='sourceParachain'
                        value={transferInfo.sourceParachain.toString()}
                        onChange={handleChange}
                        disabled />
                    <label htmlFor='tokenAddress'>Token Address:</label>
                    <input type='text'
                        placeholder='0x0000000000000000000000000000000000000000'
                        required
                        name='tokenAddress'
                        value={transferInfo.tokenAddress}
                        onChange={handleChange} />
                    <label htmlFor='beneficiary'>Beneficiary:</label>
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
                    <button disabled={transferInfo.transferInProgress} type='submit'>Send</button>
                    <p hidden={transferInfo.result === undefined} style={{ gridColumn: 'span 2' }}>TxHash: {transferInfo.result?.success?.assetHub.txHash}</p>
                    <ul hidden={errors.length === 0} style={{ color: 'red', gridColumn: 'span 2' }}>
                        {errors.map((error, index) => (<li key={index}>{error}</li>))}
                    </ul>
                    <ul hidden={statusUpdates.length === 0} style={{ color: 'green', gridColumn: 'span 2' }}>
                        {statusUpdates.map((update, index) => (<li key={index}>{update}</li>))}
                    </ul>
                </form>
            </div>
        )
    } else {
        return (
            <button onClick={walletConnect}>Connect Wallet</button>
        )
    }
}