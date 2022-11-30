use std::collections::BTreeMap;
use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::core::engine_state::{ExecuteRequest};
use casper_types::{runtime_args, ContractHash, RuntimeArgs, Key, account::AccountHash, CLValue};
use tempfile::TempDir;

use serde::{Deserialize, Serialize};

use clap::Parser;

pub(crate) const NFT_CONTRACT_WASM: &str = "contract.wasm";
pub(crate) const MINT_SESSION_WASM: &str = "mint_call.wasm";
pub(crate) const CONTRACT_NAME: &str = "nft_contract";
pub(crate) const NFT_TEST_COLLECTION: &str = "nft_test";
pub(crate) const NFT_TEST_SYMBOL: &str = "TEST";
pub(crate) const ARG_COLLECTION_NAME: &str = "collection_name";
pub(crate) const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
pub(crate) const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
pub(crate) const ARG_ALLOW_MINTING: &str = "allow_minting";
pub(crate) const ARG_MINTING_MODE: &str = "minting_mode";
pub(crate) const ARG_HOLDER_MODE: &str = "holder_mode";
pub(crate) const ARG_WHITELIST_MODE: &str = "whitelist_mode";
pub(crate) const ARG_CONTRACT_WHITELIST: &str = "contract_whitelist";
pub(crate) const ARG_TOKEN_META_DATA: &str = "token_meta_data";
pub(crate) const ARG_TOKEN_OWNER: &str = "token_owner";
pub(crate) const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
pub(crate) const ARG_JSON_SCHEMA: &str = "json_schema";
pub(crate) const ARG_NFT_METADATA_KIND: &str = "nft_metadata_kind";
pub(crate) const ARG_IDENTIFIER_MODE: &str = "identifier_mode";
pub(crate) const ARG_METADATA_MUTABILITY: &str = "metadata_mutability";
pub(crate) const ARG_BURN_MODE: &str = "burn_mode";
pub(crate) const ARG_OWNERSHIP_MODE: &str = "ownership_mode";
pub(crate) const ARG_NFT_KIND: &str = "nft_kind";
pub(crate) const TEST_PRETTY_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.barfoo.com"
}"#;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Token Supply
    #[arg(long, default_value_t = 1000000)]
    token_supply: u64,

    /// Contract Page Size
    #[arg(long, default_value_t = 256)]
    page_size: u64,

    /// Contract WASM
    #[arg(long, default_value_t = NFT_CONTRACT_WASM.to_string())]
    contract_wasm: String,


}


fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    let data_dir = TempDir::new().expect("should create temp dir");
    println!("global state storing in: {:?}", data_dir.path());

    let token_supply = args.token_supply;

    let mut builder = LmdbWasmTestBuilder::new(data_dir.as_ref());
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, &args.contract_wasm)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(token_supply)
        .with_page_size(args.page_size)
        .build();

    builder.exec(install_request).expect_success().commit();
    println!("token supply: {}  install gas: {}", token_supply, builder.last_exec_gas_cost());
    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let initial_size = builder.lmdb_on_disk_size().expect("expected lmdb size");
    let mint_per_block: u64 = 50;
    println!("mint_count,lmdb_bytes,gas_cost");
    for current_mint in 0..token_supply {
        let mint_session_call =
            ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_ADDR,
                MINT_SESSION_WASM,
                runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
                },
            ).build();

            builder.scratch_exec_and_commit(mint_session_call).expect_success();
            let last_gas = builder.last_exec_gas_cost();
            if current_mint % mint_per_block == 0 {
                // Write out to simulate block created.
                builder.write_scratch_to_db();
                builder.flush_environment();
            }
            println!("{},{},{}", current_mint, builder.lmdb_on_disk_size().expect("expected lmdb size") - initial_size, last_gas);
        }

    println!("Final Growth: {:?}", builder.lmdb_on_disk_size().unwrap() - initial_size);

}

 fn get_nft_contract_hash(
    builder: &LmdbWasmTestBuilder,
) -> ContractHash {
    let nft_hash_addr = builder
        .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have this entry in named keys")
        .into_hash()
        .expect("must get hash_addr");

    ContractHash::new(nft_hash_addr)
}

#[repr(u8)]
pub enum WhitelistMode {
    Unlocked = 0,
    Locked = 1,
}

#[repr(u8)]
pub enum NFTHolderMode {
    Accounts = 0,
    Contracts = 1,
    Mixed = 2,
}

#[repr(u8)]
pub enum MintingMode {
    /// The ability to mint NFTs is restricted to the installing account only.
    Installer = 0,
    /// The ability to mint NFTs is not restricted.
    Public = 1,
}

#[repr(u8)]
#[derive(Debug)]
pub enum OwnershipMode {
    Minter = 0,       // The minter owns it and can never transfer it.
    Assigned = 1,     // The minter assigns it to an address and can never be transferred.
    Transferable = 2, // The NFT can be transferred even to an recipient that does not exist.
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum NFTKind {
    Physical = 0,
    Digital = 1, // The minter assigns it to an address and can never be transferred.
    Virtual = 2, // The NFT can be transferred even to an recipient that does not exist
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MetadataSchemaProperty {
    name: String,
    description: String,
    required: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CustomMetadataSchema {
    properties: BTreeMap<String, MetadataSchemaProperty>,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    name: String,
    symbol: String,
    token_uri: String,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum NFTMetadataKind {
    CEP78 = 0,
    NFT721 = 1,
    Raw = 2,
    CustomValidated = 3,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum NFTIdentifierMode {
    Ordinal = 0,
    Hash = 1,
}

#[repr(u8)]
pub enum MetadataMutability {
    Immutable = 0,
    Mutable = 1,
}

#[repr(u8)]
pub enum BurnMode {
    Burnable = 0,
    NonBurnable = 1,
}

#[derive(Debug)]
pub(crate) struct InstallerRequestBuilder {
    account_hash: AccountHash,
    session_file: String,
    collection_name: CLValue,
    collection_symbol: CLValue,
    total_token_supply: CLValue,
    allow_minting: CLValue,
    minting_mode: CLValue,
    ownership_mode: CLValue,
    nft_kind: CLValue,
    holder_mode: CLValue,
    whitelist_mode: CLValue,
    contract_whitelist: CLValue,
    json_schema: CLValue,
    nft_metadata_kind: CLValue,
    identifier_mode: CLValue,
    metadata_mutability: CLValue,
    burn_mode: CLValue,
    page_size: CLValue,
}

impl InstallerRequestBuilder {
    pub(crate) fn new(account_hash: AccountHash, session_file: &str) -> Self {
        Self::default()
            .with_account_hash(account_hash)
            .with_session_file(session_file.to_string())
    }

    pub(crate) fn default() -> Self {
        InstallerRequestBuilder {
            account_hash: AccountHash::default(),
            session_file: String::default(),
            collection_name: CLValue::from_t("name".to_string()).expect("name is legit CLValue"),
            collection_symbol: CLValue::from_t("SYM").expect("collection_symbol is legit CLValue"),
            total_token_supply: CLValue::from_t(1u64).expect("total_token_supply is legit CLValue"),
            allow_minting: CLValue::from_t(true).unwrap(),
            minting_mode: CLValue::from_t(MintingMode::Installer as u8).unwrap(),
            ownership_mode: CLValue::from_t(OwnershipMode::Minter as u8).unwrap(),
            nft_kind: CLValue::from_t(NFTKind::Physical as u8).unwrap(),
            holder_mode: CLValue::from_t(NFTHolderMode::Mixed as u8).unwrap(),
            whitelist_mode: CLValue::from_t(WhitelistMode::Unlocked as u8).unwrap(),
            contract_whitelist: CLValue::from_t(Vec::<ContractHash>::new()).unwrap(),
            json_schema: CLValue::from_t("test".to_string())
                .expect("test_metadata was created from a concrete value"),
            nft_metadata_kind: CLValue::from_t(NFTMetadataKind::NFT721 as u8).unwrap(),
            identifier_mode: CLValue::from_t(NFTIdentifierMode::Ordinal as u8).unwrap(),
            metadata_mutability: CLValue::from_t(MetadataMutability::Mutable as u8).unwrap(),
            burn_mode: CLValue::from_t(BurnMode::Burnable as u8).unwrap(),
            page_size: CLValue::from_t(10u64).unwrap(),
        }
    }

    pub(crate) fn with_account_hash(mut self, account_hash: AccountHash) -> Self {
        self.account_hash = account_hash;
        self
    }

    pub(crate) fn with_session_file(mut self, session_file: String) -> Self {
        self.session_file = session_file;
        self
    }

    pub(crate) fn with_collection_name(mut self, collection_name: String) -> Self {
        self.collection_name =
            CLValue::from_t(collection_name).expect("collection_name is legit CLValue");
        self
    }

    pub(crate) fn with_collection_symbol(mut self, collection_symbol: String) -> Self {
        self.collection_symbol =
            CLValue::from_t(collection_symbol).expect("collection_symbol is legit CLValue");
        self
    }

    pub(crate) fn with_total_token_supply(mut self, total_token_supply: u64) -> Self {
        self.total_token_supply =
            CLValue::from_t(total_token_supply).expect("total_token_supply is legit CLValue");
        self
    }

    pub(crate) fn with_page_size(mut self, page_size: u64) -> Self {
        self.page_size = CLValue::from_t(page_size).expect("page_size is legit CLValue");
        self
    }

    pub(crate) fn build(self) -> ExecuteRequest {
        let mut runtime_args = RuntimeArgs::new();
        runtime_args.insert_cl_value(ARG_COLLECTION_NAME, self.collection_name);
        runtime_args.insert_cl_value(ARG_COLLECTION_SYMBOL, self.collection_symbol);
        runtime_args.insert_cl_value(ARG_TOTAL_TOKEN_SUPPLY, self.total_token_supply);
        runtime_args.insert_cl_value(ARG_ALLOW_MINTING, self.allow_minting);
        runtime_args.insert_cl_value(ARG_MINTING_MODE, self.minting_mode.clone());
        runtime_args.insert_cl_value(ARG_OWNERSHIP_MODE, self.ownership_mode);
        runtime_args.insert_cl_value(ARG_NFT_KIND, self.nft_kind);
        runtime_args.insert_cl_value(ARG_HOLDER_MODE, self.holder_mode);
        runtime_args.insert_cl_value(ARG_WHITELIST_MODE, self.whitelist_mode);
        runtime_args.insert_cl_value(ARG_CONTRACT_WHITELIST, self.contract_whitelist);
        runtime_args.insert_cl_value(ARG_JSON_SCHEMA, self.json_schema);
        runtime_args.insert_cl_value(ARG_NFT_METADATA_KIND, self.nft_metadata_kind);
        runtime_args.insert_cl_value(ARG_IDENTIFIER_MODE, self.identifier_mode);
        runtime_args.insert_cl_value(ARG_METADATA_MUTABILITY, self.metadata_mutability);
        runtime_args.insert_cl_value(ARG_BURN_MODE, self.burn_mode);
        ExecuteRequestBuilder::standard(self.account_hash, &self.session_file, runtime_args).build()
    }
}
