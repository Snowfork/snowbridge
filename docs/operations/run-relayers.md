---
description: Steps to set up your own Snowbridge message relayers.
---

# Run Relayers

## AWS Account

The first thing you will need is an AWS account. [Register](https://signin.aws.amazon.com/signup?request\_type=register) if you do not have an account yet.

## Clone infra Repo

Clone the [infrastructure repository](https://github.com/Snowfork/snowbrige-relayers-infra):

```sh
git clone https://github.com/Snowfork/snowbrige-relayers-infra.git
```

## Install Ansible & dependencies

Install Ansible and its dependencies:

```sh
brew install pipx
pipx install --include-deps ansible boto3 botocore
pipx ensurepath
```

## Create AWS Key Pair

On the AWS console, under the EC2 section, create an ED25519 key pair called `snowbridge-relayers-key`.

### Set AWS Env variables

In the `snowbridge-relayers-infra` directory, create a .envrc file with the following values:

```
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
export AWS_ACCOUNT_ID=
export AWS_DEFAULT_REGION=eu-central-1
```

Add your AWS access key ID, secret access key and account ID.

## Create EC2 instance

Run command from inside the `snowbridge-relayers-infra` directory:

`ansible-playbook -i inventory/message-relayers/aws_ec2.yml infra.yml`

It will create an EC2 instance to run the relayers on.

## Add Secrets

Add the following plaintext secrets to AWS secrets manager:

```
snowbridge/dwellir-eth-node-api-key
snowbridge/dwellir-polkadot-node-api-key
snowbridge/chainalysis-api-key
snowbridge/asset-hub-ethereum-relay
snowbridge/asset-hub-parachain-relay
```

<figure><img src="../.gitbook/assets/Screenshot 2024-10-22 at 19.44.27.png" alt="" width="563"><figcaption><p>Example of how to add an AWS secret.</p></figcaption></figure>

### Lodestar, Polkadot Nodes & Chainalysis Key

Ask for API keys for `dwellir-eth-node-api-key`, `dwellir-polkadot-node-api-key` and `chainalysis-api-key` in Snowbridge Relayer Telegram group: [https://t.me/+I8Iel-Eaxcw3NjU0](https://t.me/+I8Iel-Eaxcw3NjU0) (keys will be DM'ed to you).

### Ethereum Relay Key

The `asset-hub-ethereum-relay` is a private key for an prefunded account on Polkadot BridgeHub. To retrieve the private key from an account on Polkadot with seedphrase "cat cow milk...", use [subkey](https://docs.substrate.io/reference/command-line-tools/subkey/):

```
./target/release/subkey inspect "cat cow milk..."
```

Use the secret seed hash as the `snowbridge/asset-hub-ethereum-relay` secret.

### Parachain Relay Key

The `asset-hub-parachain-relay` is a private key for a funded account on Ethereum.

## Set Relayer Number

Once you have set up all of the above, ask for a relayer ID and relayer count in Snowbridge Relayer Telegram group: [https://t.me/+I8Iel-Eaxcw3NjU0](https://t.me/+I8Iel-Eaxcw3NjU0). Add the key and ID in your `.envrc` file. [Example .envrc file](https://github.com/Snowfork/snowbrige-relayers-infra/blob/main/.envrc-example#L5-L6).

## Install Relayers

Once you have added all the secrets, you can deploy your relayers:

```
ssh-agent bash
ssh-add /path/to/snowbridge-relayers-key.pem
ansible-playbook -i inventory/message-relayers/aws_ec2.yml relayers.yml
```

Once it has completed, ssh into your instance.

```
ssh -i message-relayers-key.pem ubuntu@xxx.eu-central-1.compute.amazonaws.com
```

Check that you see no relayer errors for each relayer:

```
sudo journalctl -fu snowbridge-asset-hub-ethereum-relay --since today
sudo journalctl -fu snowbridge-asset-hub-parachain-relay --since today
```

### Increment Relayer count

Once the relayer has started up successfully, all relaying parties should increment their [relayer count config and redeploy](https://github.com/Snowfork/snowbrige-relayers-infra/blob/main/.envrc-example#L5-L6) their relayer config. This action will be prompted in the TG group.

### Monitoring

TODO

