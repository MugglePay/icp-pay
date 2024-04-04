# Feedback

## Task

The objective was to develop a smart contract, or in Internet Computer Protocol (ICP) terminology, a canister, capable of deducting a specified amount from incoming ICP token transfers and refunding the remainder to the sender's account.

For instance, upon receiving 100 ICP tokens from a user, the contract would refund 99.9 ICP back to the sender and forward 0.1 ICP to another designated address.

```
Coins Support - ICP Tokens on ICP blockchain
```

## Research

We utilized an example provided by the Internet Computer community to create a canister. The example, available at [GitHub - ICP Transfer Example](https://github.com/dfinity/examples/tree/master/motoko/icp_transfer), served as a foundation for understanding ledger smart contracts. Following the example, we successfully deployed the smart contract on our local environment, leveraging tools such as dfx and mops cli as instructed.

Thus far, we have obtained the canister address to which ICP tokens are sent. Subsequently, we need to convert these tokens into cycles, the currency the smart contract utilizes to maintain its operation.

## Issues

The primary obstacle encountered was determining how to trigger the canister's functionality upon receiving token transfers. Unlike traditional smart contract platforms like Ethereum, where token transfers often automatically execute code, ICP requires further exploration into mechanisms like outside canister calls or asynchronous calls to achieve the desired behavior.

The complexity of the learning curve, especially compared to platforms like Ethereum, posed a notable obstacle. Additionally, the fact that many development aspects are still in beta phase contributed to the difficulty.

A crucial decision point involves selecting the appropriate programming language for canister development. ICP offers two main options: Rust and Motoko for production. 

## Community Resources for Learning

For those navigating the learning curve, several community resources can provide valuable insights:

- [101 Teaching Lessons](https://dacade.org/communities/icp) - Offers educational materials tailored to learning about the Internet Computer Protocol.
- [Ledger Transfer Documentation](https://internetcomputer.org/docs/current/references/samples/motoko/ledger-transfer/) - Official documentation providing guidance on ledger transfers.
- [ICP Samples](https://internetcomputer.org/samples?selectedDomains=Asynchronous+DeFi) - Collection of sample projects showcasing various capabilities of the Internet Computer Protocol.
