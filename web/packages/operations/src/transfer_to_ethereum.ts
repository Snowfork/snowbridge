import { Keyring } from "@polkadot/keyring";
import { Context, environment, toEthereumV2, assetsV2 } from "@snowbridge/api";
import { cryptoWaitReady } from "@polkadot/util-crypto";
import { formatUnits, Wallet } from "ethers";
import { fetchRegistry } from "./registry";

export const transferToEthereum = async (
	sourceParaId: number,
	symbol: string,
	amount: bigint,
) => {
	let env = "local_e2e";
	if (process.env.NODE_ENV !== undefined) {
		env = process.env.NODE_ENV;
	}
	const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env];
	if (snwobridgeEnv === undefined) {
		throw Error(`Unknown environment '${env}'`);
	}
	console.log(`Using environment '${env}'`);

	const { name, config, ethChainId } = snwobridgeEnv;
	await cryptoWaitReady();

	const ethApikey = process.env.REACT_APP_INFURA_KEY || "";
	const ethChains: { [ethChainId: string]: string } = {};
	Object.keys(config.ETHEREUM_CHAINS).forEach(
		(ethChainId) =>
			(ethChains[ethChainId.toString()] =
				config.ETHEREUM_CHAINS[ethChainId](ethApikey)),
	);
	const context = new Context({
		environment: name,
		ethereum: {
			ethChainId,
			ethChains,
			beacon_url: config.BEACON_HTTP_API,
		},
		polkadot: {
			assetHubParaId: config.ASSET_HUB_PARAID,
			bridgeHubParaId: config.BRIDGE_HUB_PARAID,
			relaychain: config.RELAY_CHAIN_URL,
			parachains: config.PARACHAINS,
		},
		appContracts: {
			gateway: config.GATEWAY_CONTRACT,
			beefy: config.BEEFY_CONTRACT,
		},
	});

	const polkadot_keyring = new Keyring({ type: "sr25519" });

	const ETHEREUM_ACCOUNT = new Wallet(
		process.env.ETHEREUM_KEY ?? "your key goes here",
		context.ethereum(),
	);
	const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress();
	const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(
		process.env.SUBSTRATE_KEY ?? "your key goes here",
	);
	const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address;

	console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC);

	const registry = await fetchRegistry(env, context);

	const assets = registry.ethereumChains[registry.ethChainId].assets;
	const TOKEN_CONTRACT = Object.keys(assets)
		.map((t) => assets[t])
		.find((asset) =>
			asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()),
		)?.token;

	console.log("Asset Hub to Ethereum");
	{
		// Step 1. Get the delivery fee for the transaction
		const fee = await toEthereumV2.getDeliveryFee(
			{
				assetHub: await context.assetHub(),
				source: await context.parachain(sourceParaId),
			},
			sourceParaId,
			registry,
			// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
			TOKEN_CONTRACT!,
		);

		// Step 2. Create a transfer tx
		const transfer = await toEthereumV2.createTransfer(
			await context.parachain(sourceParaId),
			registry,
			POLKADOT_ACCOUNT_PUBLIC,
			ETHEREUM_ACCOUNT_PUBLIC,
			TOKEN_CONTRACT!,
			amount,
			fee,
		);

		// Step 3. Estimate the cost of the execution cost of the transaction
		console.log("call: ", transfer.tx.inner.toHex());
		console.log("utx: ", transfer.tx.toHex());
		const feePayment = (
			await transfer.tx.paymentInfo(POLKADOT_ACCOUNT, {
				withSignedTransaction: true,
			})
		).toPrimitive() as any;
		console.log(
			`execution fee (${transfer.computed.sourceParachain.info.tokenSymbols}):`,
			formatUnits(
				feePayment.partialFee,
				transfer.computed.sourceParachain.info.tokenDecimals,
			),
		);
		console.log(
			`delivery fee (${registry.parachains[registry.assetHubParaId].info.tokenSymbols}): `,
			formatUnits(
				fee.totalFeeInDot,
				transfer.computed.sourceParachain.info.tokenDecimals,
			),
		);
		// console.log(
		//     "dryRun: ",
		//     (await transfer.tx.dryRun(POLKADOT_ACCOUNT, { withSignedTransaction: true })).toHuman()
		// )

		// Step 4. Validate the transaction.
		const validation = await toEthereumV2.validateTransfer(
			{
				sourceParachain: await context.parachain(sourceParaId),
				assetHub: await context.assetHub(),
				gateway: context.gateway(),
				bridgeHub: await context.bridgeHub(),
			},
			transfer,
		);
		console.log("validation result", validation);

		// Step 5. Check validation logs for errors
		if (
			validation.logs.find((l) => l.kind == toEthereumV2.ValidationKind.Error)
		) {
			throw Error(`validation has one of more errors.`);
		}
		if (process.env["DRY_RUN"] != "true") {
			// Step 6. Submit transaction and get receipt for tracking
			const response = await toEthereumV2.signAndSend(
				await context.parachain(sourceParaId),
				transfer,
				POLKADOT_ACCOUNT,
				{ withSignedTransaction: true },
			);
			if (!response) {
				throw Error(`Transaction ${response} not included.`);
			}
			console.log("Success message", response.messageId);
		}
	}
	await context.destroyContext();
};
