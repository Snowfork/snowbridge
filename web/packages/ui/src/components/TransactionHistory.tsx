import { toEthereum, toPolkadot } from "@snowbridge/api"
import { AppProps, TransactionHistoryItem, TransactionHistoryStore, TransactionStatus } from "./Common"

interface TransactionHistoryProps extends AppProps {
    history: TransactionHistoryStore
    setHistory: React.Dispatch<React.SetStateAction<TransactionHistoryStore>>
}

interface TransactionProps {
    transaction: TransactionHistoryItem
}
const Transaction = (props: TransactionProps): JSX.Element => {
    let { type, when, status, result, messages } = props.transaction
    if (type === 'toEthereum') {
        let data = result as toEthereum.SendResult
        let plan = data.success?.plan.success
        return (<div className="transaction" >
            <h3>{TransactionStatus[status]} Transfer {data.success?.plan.success?.amount.toString()} to Ethereum on {when}</h3>
            <table>
                <tbody>
                    <tr>
                        <td>Token</td>
                        <td>{plan?.tokenAddress}</td>
                    </tr>
                    <tr>
                        <td>Source Address</td>
                        <td>{plan?.sourceAddress}</td>
                    </tr>
                    <tr>
                        <td>Beneficiary</td>
                        <td>{plan?.beneficiary}</td>
                    </tr>
                    <tr>
                        <td>Amount</td>
                        <td>{plan?.amount.toString()}</td>
                    </tr>
                </tbody>
            </table>
            <ul hidden={messages.length === 0} style={{ color: 'green', gridColumn: 'span 2' }}>
                {messages.map((update, index) => (<li key={index}>{update}</li>))}
            </ul>
        </div>)
    } else {
        let data = result as toPolkadot.SendResult;
        let plan = data.success?.plan.success
        return (<div className="transaction">
            <h3>{TransactionStatus[status]} Transfer {data.success?.plan.success?.amount.toString()} to Polkadot on {when}</h3>
            <table>
                <tbody>
                    <tr>
                        <td>Token</td>
                        <td>{plan?.token}</td>
                    </tr>
                    <tr>
                        <td>Source Address</td>
                        <td>{plan?.sourceAddress}</td>
                    </tr>
                    <tr>
                        <td>Beneficiary</td>
                        <td>{plan?.beneficiaryAddress}</td>
                    </tr>
                    <tr>
                        <td>Amount</td>
                        <td>{plan?.amount.toString()}</td>
                    </tr>
                </tbody>
            </table>
            <ul hidden={messages.length === 0} style={{ color: 'green', gridColumn: 'span 2' }}>
                {messages.map((update, index) => (<li key={index}>{update}</li>))}
            </ul>
        </div>)
    }
}

export const TransactionHistory = (props: TransactionHistoryProps): JSX.Element => {
    return (<div>
        <h2>Transaction History</h2>
        {
            props.history.items
                .sort((a, b) => new Date(b.when).getTime() - new Date(a.when).getTime())
                .map(transaction => (<Transaction key={transaction.when} transaction={transaction} />))
        }
    </div>)
}