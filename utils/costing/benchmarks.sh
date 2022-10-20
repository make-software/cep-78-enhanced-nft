#!/bin/bash

USER_1_ACCOUNT=$(nctl-view-user-account user=1\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z|A-Z]{64}")
USER_2_ACCOUNT=$(nctl-view-user-account user=2\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z|A-Z]{64}")

# Transfers from user 1 to user 2

TOKEN_MINT_DEPLOY=$(casper-client put-deploy\
        --chain-name $NETWORK_NAME\
        --node-address $NODE_1_ADDRESS\
        --secret-key $USER_1_SECRET_KEY\
        --payment-amount $GAS_LIMIT\
        --session-path $MINT_WASM\
        --session-arg "nft_contract_hash:key='$TOKEN_CONTRACT_HASH'"\
        --session-arg "token_owner:key='$USER_1_ACCOUNT'"\
        --session-arg "token_meta_data:string=''"\
        | jq .result.deploy_hash\
        | tr -d '"')

sleep 120

TOKEN_TRANSFER_DEPLOY=$(casper-client put-deploy\
        --chain-name $NETWORK_NAME\
        --node-address $NODE_1_ADDRESS\
        --secret-key $USER_1_SECRET_KEY\
        --payment-amount $GAS_LIMIT\
        --session-path $TRANSFER_WASM\
        --session-arg "nft_contract_hash:key='$TOKEN_CONTRACT_HASH'"\
        --session-arg "token_id:u64='0'"\
        --session-arg "target_key:key='$USER_2_ACCOUNT'"\
        --session-arg "source_key:key='$USER_1_ACCOUNT'"\
        | jq .result.deploy_hash\
        | tr -d '"')

sleep 120

# Write the data
DEPLOY_TYPES=(MINT TRANSFER)
for deploy_type in ${DEPLOY_TYPES[@]}; do
  name=\$TOKEN_$deploy_type\_DEPLOY
  cost=$(nctl-view-chain-deploy deploy=$(eval "echo $name")\
          | jq .execution_results[0].result.Success.cost\
          | tr -d '"')

  echo $deploy_type, $ANNOTATION, $cost >> cep78-cost-benchmarking-output
done