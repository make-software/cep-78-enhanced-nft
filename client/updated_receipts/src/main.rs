#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use casper_contract::contract_api::runtime;
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types;
use casper_types::{ContractPackageHash, Key, runtime_args, RuntimeArgs};

const ENTRY_POINT_UPDATE_RECEIPTS: &str = "updated_receipts";
const ARG_COLLECTION_NAME: &str = "collection_name";
const ARG_NFT_PACKAGE_HASH: &str = "nft_package_hash";

#[no_mangle]
pub extern "C" fn call() {
    let nft_package_hash: ContractPackageHash = runtime::get_named_arg::<Key>(ARG_NFT_PACKAGE_HASH)
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap_or_revert();


    // This value represents a list of receipt names and addresses corresponding
    // to the pages marking ownership of NFTs owned by the account.
    let updated_receipt_data = runtime::call_versioned_contract::<Vec<(String, Key)>>(
        nft_package_hash,
        None,
        ENTRY_POINT_UPDATE_RECEIPTS,
        runtime_args! {}
    );
    for (receipt_name, dictionary_address) in updated_receipt_data.into_iter() {
        runtime::put_key(&receipt_name, dictionary_address)
    }
}


