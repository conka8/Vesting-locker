# vesting-locker
A smart contracts that gradually unlocks the Team's preminted tokens over a period of 5 years

## Testing

`cargo test`

## Deploying and upgrading

Enter your address and keyfile path in *owner_interactions.sh*. Change the proxy and chainId if necessary (devnet by default).

Run:
`source owner_interactions.sh`
then
`deploy`
(Add the flag `--metadata-not-upgradeable` line 11, before --send of *owner_interactions.sh* if this is a mainnet deployment)

Copy the contract address given by the previous command and enter it in *owner_interactions.sh*

Run again:
`source owner_interactions.sh`

To upgrade the contract, use
`upgrade`
(should only be used for development purposes, the final contract on mainnet should be deployed as non-upgradeable)

# Endpoints (All endpoints are owner_only)

**lock_tokens**()
Locks the tokens sent. Tokens must be sent for it to work. Can only be called once. Each year for 5 years, 1/5 of the locked tokens are un-lockable.

**With the script:**
`lockTokens 1000` if you want to lock 1000 tokens. Floating point numbers are parsed, 0.01 can be entered as is.

**unlock_tokens**()
If at least a year has passed since the last unlock (or the first lock), unlocks 1/5 of the locked tokens and sends them to the caller.

**With the script:**
`unlockTokens`
