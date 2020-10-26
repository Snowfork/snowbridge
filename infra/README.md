# Infrastructure

This subproject contains the tooling and configuration necessary to provision our testnets

## Testnets

### Rococo

RPC Endpoint: wss://parachain-rpc.snowfork.network

Our parachain testnet is hosted in the AWS eu-west-1 region. It currently has 3 nodes, spread out across all availability zones for redundancy.

To interact with the parachain, visit https://polkaeth-substrate.netlify.app, and select the _Ethereum Bridge_ testnet.

_Note: We are running our own relay chain until we switch over to the one hosted by Parity/W3F_

## Provisioning

We use Ansible for provisioning and maintaining our infrastructure.

### Getting Started

A recent Ubuntu environment is recommended to run Ansible.

The following packages are required:

- Python3
- Boto3
- Ansible 2.10

To install them, run the following:

```bash
# pip
apt install python3-pip

# boto3
python3 -m pip install --user boto3

# ansible
python3 -m pip install --user ansible
```

### AWS Access

Some playbooks require authenticated access to AWS. Follow this guide [Configuration](https://boto3.amazonaws.com/v1/documentation/api/latest/guide/configuration.html#guide-configuration) to setup credentials for boto3.

Now set the configuration profile:

```
export AWS_PROFILE=<profile>
```

### SSH Access

Ansible connects to our EC2 instances using public key authentication. Make sure you have an `ssh-agent` running. Request the EC2 keypair from a team member and add it your agent keyring

Example:
```
ssh-add ~/.ssh/<key>
```

### Secrets

Various secrets are encrypted at rest in our repository using [Ansible Vault](https://docs.ansible.com/ansible/latest/user_guide/vault.html).

In order to have Ansible decrypt these secrets, acquire the encryption key and write it to `ansible/.vault-password`.

### Deployments

Change to the ansible directory
```
cd ansible
```

Create or update EC2 resources:

```bash
ansible-playbook aws.yml
```

Create or update the parachain:

```bash
ansible-playbook parachain.yml
```

Create or update the relay:

```bash
ansible-playbook relay.yml
```
