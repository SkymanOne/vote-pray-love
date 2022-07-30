# Vote Pray Love

Polkadot Blockchain Academy 2022 Cohort Final Exam Project.

## Milestones
- [ ] Basic quadratic voting system
- [ ] Anonymous quadratic system (commit-reveal)
- [ ] Staking
- [ ] Slashing

## Potential idea for anonymous voting

1. Author creates a proposal and generates a keypair for this proposal
2. The public key of the proposer is distributed to voters
3. Voters encrypt their vote using the public key and sign transaction with their private key
4. Once timeout and enough votes are collected, the proposer is ready to reveal the results
5. The proposer uses their private key to decrypt votes and calculate the outcome of vote

If the proposer reveals results before the timeout -> slashing
If the proposer tries to inside-trade the intermediate vote results -> no solution, might be worth using nominating random voters
to generate keypairs and use multi-sigs to collectively reveal the results

## Potential idea for anonymous voting 2
1. Author publishes a proposal
2. The voters commit their decision
3. Once timeout is out, the voters have some time to reveal their choices
4. If the voters does not reveal their results -> slashing

## Resources

[Commit - reveal](https://karl.tech/learning-solidity-part-2-voting/)
