#--------TECHNICAL--------

ADDRESS="erd1xxx"
OWNER_ADDRESS="erd1xxx"
PRIVATE_KEY=(--keyfile=erd1xxx.json --passfile=.passfile)
PASSFILE=--passfile=.passfile # Ignore that
PROXY=https://devnet-api.elrond.com
CHAIN_ID=D

TOKEN_IDENTIFIER="EVILLE-a7976c" # Token identifier
TOKEN_NONCE=0 # 0 for fungible tokens

deploy() {
    erdpy --verbose contract deploy --bytecode output/vesting-locker.wasm --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --metadata-not-upgradeable --send --outfile="deploy.interaction.json" || return

    TRANSACTION=$(erdpy data parse --file="deploy.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=vesting-locker-address-devnet --value=${ADDRESS}
    erdpy data store --key=vesting-locker-deployTransaction-devnet --value=${TRANSACTION}

}

upgrade() {
   echo "Upgrading Smart Contract address: ${ADDRESS}"
   erdpy --verbose contract upgrade ${ADDRESS} --bytecode output/vesting-locker.wasm --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --send

}

lockTokens() {
    token_name="0x$(echo -n ${TOKEN_IDENTIFIER} | xxd -p -u | tr -d '\n')"
    nonce=${TOKEN_NONCE}
    amount=$(echo "scale=0; (${1}*10^18)/1" | bc -l) # Lets you enter it as 0.05 instead of 50000000000000000
    sc_function="0x$(echo -n 'lockTokens' | xxd -p -u | tr -d '\n')"
    sc_address="0x$(erdpy wallet bech32 --decode ${ADDRESS})"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce ${PRIVATE_KEY} \
            --gas-limit=50000000 \
            --proxy=${PROXY} --chain=${CHAIN_ID} \
            --function="ESDTTransfer" \
            --arguments ${token_name} ${amount} ${sc_function}\
            --send
    echo $?
}

unlockTokens() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce ${PRIVATE_KEY}\
        --gas-limit=50000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --function "unlockTokens" \
        --send

    echo $?
}
