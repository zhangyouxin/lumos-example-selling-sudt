// Import from `core` instead of from `std` since we are in no-std mode
use core::{result::Result};

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    debug,
    ckb_constants::Source,
    high_level::{load_script, load_cell_lock, load_cell, QueryIter},
    ckb_types::{bytes::Bytes, prelude::*, packed::*},
};
use crate::error::Error;

// the data type is u64, so that we need 8 bytes to store it
const MINIMAL_CAPACITY_LEN: usize = 8;

fn selling_lock_args_same_as_script(args: &Bytes, lock: &Script) ->  bool {
    let minimal_capacity_start_point = args.len() - MINIMAL_CAPACITY_LEN;
    let raw_args = args.as_ref();
    debug!("selling_lock_args_same_as_script?");
    debug!("lock.code_hash().as_reader().raw_data()[..] is {:?}", lock.code_hash().as_reader().raw_data());
    debug!("raw_args is {:?}", &raw_args[..]);
    lock.code_hash().as_reader().raw_data()[..] == raw_args[0..32] &&
    lock.hash_type().as_reader().as_slice()[0] == raw_args[32] &&
    lock.args().as_reader().raw_data()[..] == raw_args[33..minimal_capacity_start_point]
}

fn selling_lock_args_code_hash_same_as_script(selling_lock_code_hash: &[u8], selling_lock_hash_type: &u8, lock: &Script) ->  bool {
    let result = lock.code_hash().as_reader().raw_data()[..] == selling_lock_code_hash[..] &&
        lock.hash_type().as_reader().as_slice()[0] == *selling_lock_hash_type;
    debug!("reslt is {:?}", result);
    return result;
}

fn check_is_owner(args: &Bytes) -> Result<bool, Error> {
    let is_owner = QueryIter::new(load_cell_lock, Source::Input)
        .find(|lock: &Script| {
            debug!("check_is_owner:");
            selling_lock_args_same_as_script(args, lock)
        }).is_some();
    Ok(is_owner)
}

fn outputs_contains_owner_cell_with_no_type(args: &Bytes) -> Result<bool, Error> {
    let contains_owner_cell = QueryIter::new(load_cell, Source::Output)
        .find(|output: &CellOutput| {
            debug!("outputs_contains_owner_cell:");
            selling_lock_args_same_as_script(args, &output.lock()) && output.type_().as_reader().is_none()
        }).is_some();
    Ok(contains_owner_cell)
}

fn collect_outputs_owner_amount(args: &Bytes) -> Result<u64, Error> {
    debug!("enter collect_outputs_owner_amount:");
    let mut buf = [0u8; MINIMAL_CAPACITY_LEN];
    let capacity_list = QueryIter::new(load_cell, Source::Output)
        .map(|cell: CellOutput|{
            debug!("now collect_outputs_owner_amount:");
            debug!("selling_lock_args_same_as_script: {:?}",selling_lock_args_same_as_script(args, &cell.lock()));
            debug!("cell.type_().as_reader().is_none(): {:?}",cell.type_().as_reader().is_none());
            if selling_lock_args_same_as_script(args, &cell.lock()) && cell.type_().as_reader().is_none() {
                debug!("&cell.capacity().raw_data(): {:?}",&cell.capacity().raw_data());
                buf.copy_from_slice(&cell.capacity().raw_data());
                return Ok(u64::from_le_bytes(buf));
            } else {
                return Ok(0u64)
            }
        }).collect::<Result<Vec<_>, Error>>()?;
    Ok(capacity_list.into_iter().sum::<u64>())
}

fn get_self_capacity(selling_lock_code_hash: &[u8], selling_lock_hash_type: &u8) -> Result<u64, Error> {
    let mut buf = [0u8; MINIMAL_CAPACITY_LEN];

    let capacity_list = QueryIter::new(load_cell, Source::Input)
        .map(|cell|{
            debug!("now get_self_capacity:");
            if selling_lock_args_code_hash_same_as_script(selling_lock_code_hash, selling_lock_hash_type, &cell.lock()) {
                debug!("into the selling lock capacity:");
                buf.copy_from_slice(cell.as_reader().capacity().as_slice());
                Ok(u64::from_le_bytes(buf))
            } else {
                debug!("not the selling lock capacity:");
                Ok(0u64)
            }
        }).collect::<Result<Vec<_>, Error>>()?;
    debug!("self capacity_list is: {:?}", capacity_list);
    Ok(capacity_list.into_iter().sum::<u64>())
}

fn get_price(selling_lock_code_hash: &[u8], selling_lock_hash_type: &u8) -> Result<u64, Error> {
    let capacity_list = QueryIter::new(load_cell, Source::Input)
        .map(|cell|{
            debug!("now get_price:");
            if selling_lock_args_code_hash_same_as_script(selling_lock_code_hash, selling_lock_hash_type, &cell.lock()) {
                debug!("into the selling lock price:");
                let cell_args = cell.as_reader().lock().args().raw_data();
                let data_len = cell_args.len();
                let start_point = data_len - MINIMAL_CAPACITY_LEN;
                let mut buf = [0u8; MINIMAL_CAPACITY_LEN];
                buf.copy_from_slice(&cell_args[start_point..]);
                Ok(u64::from_le_bytes(buf))
            } else {
                debug!("not the selling lock capacity:");
                Ok(0u64)
            }
        }).collect::<Result<Vec<_>, Error>>()?;
    debug!("self capacity_list is: {:?}", capacity_list);
    Ok(capacity_list.into_iter().sum::<u64>())
}
pub fn main() -> Result<(), Error> {
    let script: Script = load_script()?;
    let args: Bytes = script.args().unpack();
    debug!("script args is {:?}", &args);
    let script_code_hash = script.code_hash();
    let script_hash_type = script.hash_type();
    let selling_lock_code_hash = &script_code_hash.as_reader().raw_data()[..];
    let selling_lock_hash_type = &script_hash_type.as_reader().as_slice()[0];
    debug!("my code hash is {:?}", &selling_lock_code_hash);
    debug!("my hash type is {:?}", &selling_lock_hash_type);

    let self_capacity = get_self_capacity(&selling_lock_code_hash, &selling_lock_hash_type)?;
    debug!("self capacity is {:?}", self_capacity);

    if check_is_owner(&args)? {
        debug!("unlock by owner!");
        return Ok(());
    } else{
        debug!("unlock by purchase!");
        /*
         * outputs.contains(owner_lock) && 
         * output_owner_cell.capacity >= minimal_capaicty + self.capacity && 
         * output_owner_cell.type_script == null
         */
        let sell_price = get_price(&selling_lock_code_hash, &selling_lock_hash_type)?;
        let paid_price = collect_outputs_owner_amount(&args)?;
        debug!("sell price is {:?}", sell_price);
        debug!("paid price is {:?}", paid_price);
        if outputs_contains_owner_cell_with_no_type(&args)? && paid_price >= sell_price + self_capacity {
            return Ok(());
        } else {
            return Err(Error::MyError);
        } 
    }
}

