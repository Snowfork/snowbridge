# Infrastructure Tooling

## Requirements

- Python3
- Boto3
- Ansible 2.10

```bash
apt install python3-pip

# boto3
python3 -m pip install --user boto3

# ansible
python3 -m pip install --user ansible
```

## Secrets

### AWS

Some playbooks require authenticated access to AWS. Follow this guide [Configuration](https://boto3.amazonaws.com/v1/documentation/api/latest/guide/configuration.html#guide-configuration) to setup credentials for boto3.

### SSH

Ansible connects to EC2 instances using authenticated public-keys. Make sure you have an `ssh-agent` running. Request the EC2 keypair from a team member and add it your agent keyring

## Deployments

Deploy relay chain:

```bash
ansible-playbook relaychain.yml
```

Deploy parachain:

```bash
ansible-playbook parachain.yml
```
