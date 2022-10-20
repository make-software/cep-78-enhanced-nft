#!/bin/bash

#rm cep78-cost-benchmarking-output

# Filename annotation
#ANNOTATION="normal"
ANNOTATION="zero-storage-cost"

# NCTL config
NETWORK_NAME=casper-net-1
NODE_1_RPC_PORT=11101
NODE_1_ADDRESS=http://localhost:$NODE_1_RPC_PORT
USER_1_SECRET_KEY=$NCTL/assets/net-1/users/user-1/secret_key.pem
USER_2_SECRET_KEY=$NCTL/assets/net-1/users/user-2/secret_key.pem
GAS_LIMIT=1000000000000

# Token installation args
TOKEN_WASM=contract/target/wasm32-unknown-unknown/release/contract.wasm
TOKEN_NAME="TestToken"
TOKEN_SYMBOL="TST"
TOKEN_SUPPLY=10000000
TOKEN_OWNERSHIP=2
TOKEN_KIND=2
METADATA_KIND=0
METADATA_SCHEMA=""
TOKEN_IDENTIFIER=0
METADATA_MUTABILITY=0

# Client code paths
MINT_WASM=client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm
TRANSFER_WASM=client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm

# Make sure our token wasm exists
cd contract &&\
cargo build --release --target wasm32-unknown-unknown -p contract &&\
cd ..

# Make sure client session code also exists
cd client/mint_session &&\
cargo build --release --target wasm32-unknown-unknown -p mint_session &&\
cd ../..

cd client/transfer_session &&\
cargo build --release --target wasm32-unknown-unknown -p transfer_session &&\
cd ../..

# Install the token
TOKEN_INSTALL_DEPLOY=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_1_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-path $TOKEN_WASM\
  --session-arg "collection_name:string='$TOKEN_NAME'"\
  --session-arg "collection_symbol:string='$TOKEN_SYMBOL'"\
  --session-arg "total_token_supply:u64='$TOKEN_SUPPLY'"\
  --session-arg "ownership_mode:u8='$TOKEN_OWNERSHIP'"\
  --session-arg "nft_kind:u8='$TOKEN_KIND'"\
  --session-arg "json_schema:string='$METADATA_SCHEMA'"\
  --session-arg "nft_metadata_kind:u8='$METADATA_KIND'"\
  --session-arg "identifier_mode:u8='$TOKEN_IDENTIFIER'"\
  --session-arg "metadata_mutability:u8='$METADATA_MUTABILITY'"\
  | jq .result.deploy_hash\
  | tr -d '"')

sleep 90

# Recover contract hash
TOKEN_CONTRACT_HASH=$(nctl-view-user-account user=1\
  | tr -d "\n"\
  | grep -o  "{.*"\
  | jq '.stored_value.Account.named_keys[] | select(.name == "nft_contract") | .key'\
  | tr -d '"')

# Recover install cost
INSTALL_COST=$(nctl-view-chain-deploy deploy=$TOKEN_INSTALL_DEPLOY\
                | jq .execution_results[0].result.Success.cost\
                | tr -d '"')

echo INSTALLATION, $ANNOTATION, $INSTALL_COST >> cep78-cost-benchmarking-output