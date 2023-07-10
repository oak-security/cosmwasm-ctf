# Awesomwasm 2023 CTF

## Challenge 05: *Draupnir*

Simplified vault where users can deposit and withdraw their tokens which will be internally accounted. The vault's `owner` can perform arbitrary actions through the `OwnerAction` entry point. In addition, a two step address transfer is implemented for the `owner` role.

### Execute entry points:
```rust
pub enum ExecuteMsg {
    Deposit {},
    Withdraw { amount: Uint128 },
    OwnerAction { msg: CosmosMsg },
    ProposeNewOwner { new_owner: String },
    AcceptOwnership {},
    DropOwnershipProposal {},
}
```

Please check the challenge's [integration_tests](./src/integration_test.rs) for expected usage examples. You can use these tests as a base to create your exploit Proof of Concept.

**:house: Base scenario:**
- The contract has been instantiated with zero funds.
- `USER1` and `USER2` deposit `10_000` tokens each.
- The owner role is assigned to the `ADMIN` address.

**:star: Goal for the challenge:**
- Demonstrate how an unprivileged user can drain all the funds inside the contract.

## Scoring

This challenge has been assigned a total of **150** points: 
- **40** points will be awarded for a proper description of the finding that allows you to achieve the **Goal** above.
- **35** points will be awarded for a proper recommendation that fixes the issue.
- If the report is deemed valid, the remaining **75** additional points will be awarded for a working Proof of Concept exploit of the vulnerability.


:exclamation: The usage of [`cw-multi-test`](https://github.com/CosmWasm/cw-multi-test) is **mandatory** for the PoC, please take the approach of the provided integration tests as a suggestion.

:exclamation: Remember that insider threats and centralization concerns are out of the scope of the CTF.

## Any questions?

If you are unsure about the contract's logic or expected behavior, drop your question on the [official Telegram channel](https://t.me/+8ilY7qeG4stlYzJi) and one of our team members will reply to you as soon as possible. 

Please remember that only questions about the functionality from the point of view of a standard user will be answered. Potential solutions, vulnerabilities, threat analysis or any other "attacker-minded" questions should never be discussed publicly in the channel and will not be answered.
