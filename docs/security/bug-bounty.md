---
description: This document describes the terms of our bug bounty.
---

# Bug Bounty

To report security issues, please email [security@snowfork.com](mailto:security@snowfork.com)

## Threat model&#x20;

For reference, the original audit and threat model for Snowbridge is here: [https://github.com/oak-security/audit-reports/blob/main/Snowbridge/2024-05-24%20Audit%20Report%20-%20Snowbridge%20v1.1.pdf](https://github.com/oak-security/audit-reports/blob/main/Snowbridge/2024-05-24%20Audit%20Report%20-%20Snowbridge%20v1.1.pdf).

## Bug Bounty Scope

### Snowbridge Git Repository

[https://github.com/Snowfork/snowbridge](https://github.com/Snowfork/snowbridge/tree/main/contracts) (Only the main branch)

#### Contracts

Ethereum Contracts which cover our Beefy Light Client, Parachain Light Client, Gateway, Agent, Messaging Protocol and Asset transfer functionality:&#x20;

[https://github.com/Snowfork/snowbridge/tree/main/contracts](https://github.com/Snowfork/snowbridge/tree/main/contracts)

#### Relayer

Offchain relayer code which submits messages to both sides of the bridge:

[https://github.com/Snowfork/snowbridge/tree/main/relayer](https://github.com/Snowfork/snowbridge/tree/main/relayer)

#### Web App

Javascript SDK used by web applications for interacting with the bridge:

[https://github.com/Snowfork/snowbridge/tree/main/web](https://github.com/Snowfork/snowbridge/tree/main/web)

### Snowbridge Web App

#### Web app (Only the main branch)

Web app for the bridge’s UI:

[https://github.com/Snowfork/snowbridge-app](https://github.com/Snowfork/snowbridge-app)

### Snowbridge Polkadot SDK Code

#### Pallets (Only the main branch)

Pallets and related code on Bridge Hub in the Polkadot SDK:

[https://github.com/Snowfork/polkadot-sdk/tree/snowbridge/bridges/snowbridge](https://github.com/Snowfork/polkadot-sdk/tree/snowbridge/bridges/snowbridge)

## Assets at risk in scope

### User Funds

#### What is this?

User funds in the Polkadot or Ethereum chain to be transferred through the bridge.

#### How should it work?

Applications implemented on top of the Bridge's Transport Layer involve transferring user assets. The key process for this type of asset is to freeze user funds on the source chain and create funds on the target chain using the bridge's functionality. Balances on both sides of the bridge transaction must match current asset availability and quantity.

**What to look for?**

Any bugs around:

* Mechanism to freeze and (un)lock non authorised funds
* Logic errors in pallets or smart contracts for bridges
* Consensus or light client exploitation
* Possibility of incorrect block generation
* XCM Bridge communication exploitation
* XCM Bridge or network DoS

### Relayer Rewards

#### What is this?

Relayer owner funds including compensation for transaction fees and rewards for running a relayer, as well as any funds stored in the relayer’s account.

#### How should it work?

Funds are defined by the operator and generated to compensate for weight transaction and infrastructure costs to make the bridge work.

#### What to look for?

Any bugs around:

* Mechanism to freeze and (un)lock non authorised funds
* Misbehaving of the system that may lead to chain penalise relayers without justification.
* Re-routing rewards to a different owner address without consent

## Bug Bounty Submission Evaluation Process

### Evaluation

There are six stages in the process:

* **Submission:** Reporter sends submission to [security@snowbridge.com](mailto:security@snowbridge.com).
* **Initial Sanity Check**
  * Check for completeness, collect missing or incomplete information
  * Confirm submission, assign first reviewer
* **Verify bug and evaluate severity**
  * If needed, collect missing information or incomplete information to reproduce
  * Reproduce and provide regular status updates
* **Technical discussion**
  * Involve technical teams working the bridges components
  * Define complexity to patch the report
  * Assign an engineer to work in the fix and expedite a timeline
* **Provide feedback**
  * Provide comprehensive feedback on criticality and remediation process
  * Announce Payout Award or explain reasons of ineligibility
* **Remediation and bug disclosure**
  * Create a PR with the fix
  * Get the PR audited before merge
  * Release a new version of bridges components with the changes
  * Publish a disclosure with report details (including technical and security assessments)

### Disclosure timeframe

The bug bounty program mandates a specific remediation period, detailed in the policy, for all confirmed and in-scope bug submissions. The disclosure of these bugs is scheduled for a set period after the release of the fix, and is based on the Parity disclosure policy. However, disclosure will not occur without a completed fix, audit, and release; the disclosure timeline will be adjusted as necessary. For detailed information, please refer to the Parity [disclosure policy](https://forum.polkadot.network/t/improving-the-substrate-ecosystem-vulnerabilities-disclosure/38/18).

### Bug Bounty Severity Classification

The Bug Bounty’s approach to risk severity reflects the importance of measuring risk from both a technical and organisational perspective. Criticality ratings also take into account an attack’s likelihood and its prerequisites for successful exploitation. As in typical risk assessments, attacks more likely to occur are prioritised over less-likely attacks and will be classified accordingly. Following is a semi specific classification of severities for reports to keep in mind while participating and sending submissions:

**Critical (up to $50,000)**

* Governance compromise
* Onchain issues that can cause significant loss of end-user assets
* Unauthorised assets minting
* Unauthorised assets burning
* Double spending
* Direct loss of bridged funds due to onchain issues
* Transaction/consensus manipulation
* Extended shutdown of core or full network functionality

**High (up to $15,000)**

* Extended shutdown of partial network functionality
* Extended blocking or modifying governance process
* Extended blocking users from accessing their funds

**Medium (up to $5,000)**

* Blocking stuffing / Storage bloating leading to performance or cost issues on the bridge
* Putting on-chain data into a unexpected state without interrupting the system or users from performing their tasks
* Bugs in the Javascript SDK or web UI that cause loss of end-user assets

**Low (up to $1,000)**

* Incorrectly priced weights or transaction fees
* Bugs in the off chain relayer that could result in failed relaying or loss of relayer funds

Bugs in the web app, Javascript SDK and off chain relayers will only qualify for Low and Medium rewards, besides for exceptional circumstances at our discretion.

Bugs in the web app, Javascript SDK and off chain relayers that do not involve loss of funds or blocked access to the bridge will only qualify for the bug bounty at our own discretion - this bounty is primarily focused on security and protection of funds, rather than UX/UI bugs.

## Bug Bounty Code of Conduct, Legal and Privacy

### Rules Of The Road

This bug bounty campaign will run starting from the confirmation of the referenda

* Submissions outside the official start date will not be considered for this campaign. If you suspect that the flaw you found may be fatal for the items in the scope, please do NOT take further actions. Instead, describe your assumptions as detailed as possible in the report.
* If you’re able to compromise something significant, please stop at the point of recognition, collect the small evidence (enough to understand where you are and what you can do), and report the vulnerability.
* Duplicate submissions made within 72 hours of each other will split the bounty between reporters. If duplicate submissions are of unequal quality, the split will be at the level of the lesser report, and the greater report will receive a prorated additional bounty on top of the split. Despite striving to be transparent as much as possible, we do not disclose other participant’s names in such cases.
* If you inadvertently access, modify, delete, or store user data, we ask that you notify us immediately at security@snowbridge.com and delete any stored data after notifying us.

Our Security team will investigate and level up the bounty if it has a greater impact than you were able to determine without breaking our stuff. Please do not break our (or anyone’s) stuff as during the bounty program period more people will be using the same resources.

### What Is A Good Submission?

If there is no impact, then we aren’t really interested. Purely-theoretical findings are sometimes entertaining to investigate, so feel free to send us any. However, if there’s no way it can be used to break our systems in practice, it won’t be eligible. Read carefully and avoid submissions being discarded:

* Provide a working proof-of-concept (or equivalent evidence) — assuming that your research didn’t produce unrecoverable changes. This helps us to evaluate whether your submission is within the program’s scope and usable in possible attacks.
* Include your vision of the potential impact and potential attack scenario, including required attack conditions.
* The bug must be original and previously unreported (no traces of reporting in public issues or internal audits), however, include links to issues or PR where conversations lead you to the discovery or introduction of the vulnerability.

### Reward Eligibility

As a main rule, a reward will ONLY be made once the patch for the vulnerability has landed and been released and you are NOT allowed to share any part of the security issue with any third party, without our written consent first. In addition, consider the following:

* You must not have written the buggy code or otherwise been involved in contributing the buggy code to the Polkadot/Kusama project.
* You must be old enough to be eligible to participate in and receive payment from this program in your jurisdiction, or otherwise qualify to receive payment, whether through consent from your parent or guardian or some other way.
* We might be prevented by law from paying you. For example, if you happen to live in a country on a sanctions list that applies to us. In this case, if we can, we’re happy to make a donation to a well-established charity.
* You must NOT either directly or indirectly exploit the security vulnerability for your own gain/incite, or encourage/assist anyone else in doing so.
* Each bug will only be considered for a reward once.

Do not threaten or attempt to extort members of the Polkadot/Kusama/Snowbridge ecosystem. We reserve the right to disqualify individuals if they threaten to withhold the security issue from us or threaten to release the vulnerability, or any exposed data, to the public or any third party — or otherwise act in a malicious, disrespectful, or disruptive manner.

The reward mechanism is articulated over following areas:

* A Hall of Fame of the Bug reporter will be published and regularly updated based on new reports and associated criticality (if they wish to preserve their anonymity, this can be skipped)
* Based on the criticality a financial reward will be awarded

### How You Get Paid

* Bounty eligible bug hunters will be asked to do a KYC to prove their identity.
* Bug hunters have to sign a reward letter.
  * Details about payment timeframe and more will be detailed in the letter.
* We will request a USDC address to send you the reward.

### Legal And Privacy

The Bug Bounty Program is a discretionary rewards program for our active community to encourage and reward those who are helping to improve the systems we build. It is not a competition. We can cancel the program at any time and awards are at our sole discretion.

All Bug Bounty awards are subject to compliance with local laws, rules, and regulations. We will not issue awards to individuals who are on sanctions lists or who are in countries on sanctions lists. Please be advised that we might conduct background checks via our screening tool in order to verify this. You are responsible for all taxes payable in connection with the receipt of any rewards. Finally, your testing must not violate any law or compromise any IP rights, data — or funds — that are not yours.

**Privacy and Data Protection**

As part of participating in the Bug Bounty Program, you will need to share personal data including your name, email address, ID information and photos, and a blockchain address. Snowbridge is committed to protecting and respecting your privacy. Your data will not be shared outside of Snowbridge without your explicit permission.

**Legal Safe Harbour**

This program strongly supports and encourages security research into Polkadot/Kusama/Snowbridge. If you conduct genuine, in-scope, bug-hunting research in good faith and in accordance with this policy, we will consider your actions to be legitimate and not seek prosecution. But for the avoidance of doubt, this does not give you permission to act in any manner that is inconsistent with the law or might cause us to be in breach of any of our legal obligations.

## Curators

The curators are responsible for evaluating the effectiveness of the different phases and for the attribution of the reward to the bug bounty reporter. The bounties will be curated by the Snowbridge team directly.

\
