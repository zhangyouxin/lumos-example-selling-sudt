use core::{result::Result};
use alloc::{vec::Vec};
use ckb_std::{
    debug,
    ckb_constants::Source,
    high_level::{load_script, load_cell_lock, load_cell, QueryIter},
    ckb_types::{bytes::Bytes, prelude::*, packed::*},
};
use crate::error::Error;

const PRICE_LEN: usize = 8;
struct SellingInfo {
    // The owner of the selling lock
    owner_lock: Script,
    // minimal capacity should pay to the owner
    #[allow(dead_code)]
    price: u64,
}

fn build_lock_from_args(args: &Bytes) -> Script {
    let lock_start_point = args.len() - PRICE_LEN;
    let lock_buf = args.as_ref()[33..lock_start_point].to_vec();
    let lock = ScriptBuilder::default()
        .code_hash(Byte32::new_unchecked(Bytes::from(args[0..32].to_vec())))
        .hash_type(Byte::from(args[32]))
        .args(Bytes::from(lock_buf).pack())
        .build();
    lock
}

fn load_selling_info() -> Result<SellingInfo, Error> {
    let script: Script = load_script()?;
    let args: Bytes = script.args().unpack();
    let mut price_buf = [0u8; PRICE_LEN];
    let price_start_point = args.len() - PRICE_LEN;
    price_buf.copy_from_slice(&args.as_ref()[price_start_point..].as_ref());
    let price = u64::from_le_bytes(price_buf);
    let owner_lock = build_lock_from_args(&args);
    Ok(SellingInfo{owner_lock, price})
}

fn is_owner_of_selling_info(other_lock: &Script, selling_info: &SellingInfo) ->  bool {
    other_lock.as_slice() == selling_info.owner_lock.as_slice()
}

fn is_selling_lock(lock: &Script) -> bool {
    let script: Script = load_script().unwrap();
    lock.code_hash().as_reader().raw_data()[..] == script.code_hash().as_reader().raw_data()[..] &&
    lock.hash_type().as_reader().as_slice()[0] == script.hash_type().as_reader().as_slice()[0]
}

fn load_owner_lock_from_selling_lock(selling_lock: &Script) -> Script {
    let args: Bytes = selling_lock.args().unpack();
    let owner_lock = build_lock_from_args(&args);
    owner_lock
}

fn is_selling_lock_from_same_owner(selling_info: &SellingInfo, other_lock: &Script) -> bool {
    if !is_selling_lock(other_lock) {
        return false;
    } else {
        let owner_lock: &Script = &load_owner_lock_from_selling_lock(other_lock);
        return is_owner_of_selling_info(owner_lock, selling_info);
    }
}

fn load_selling_price(selling_info: &SellingInfo) -> Result<u64, Error> {
    let price_list = QueryIter::new(load_cell, Source::Input)
        .map(|cell|{
            if is_selling_lock_from_same_owner(selling_info, &cell.lock()) {
                let cell_args = cell.as_reader().lock().args().raw_data();
                let data_len = cell_args.len();
                let start_point = data_len - PRICE_LEN;
                let mut buf = [0u8; PRICE_LEN];
                buf.copy_from_slice(&cell_args[start_point..]);
                Ok(u64::from_le_bytes(buf))
            } else {
                Ok(0u64)
            }
        }).collect::<Result<Vec<_>, Error>>()?;
    Ok(price_list.into_iter().sum::<u64>())
}

fn load_paying_price(selling_info: &SellingInfo) -> Result<u64, Error> {
    let mut buf = [0u8; PRICE_LEN];
    let capacity_list = QueryIter::new(load_cell, Source::Output)
        .map(|cell: CellOutput|{
            if is_owner_of_selling_info( &cell.lock(), selling_info) {
                buf.copy_from_slice(&cell.capacity().raw_data());
                return Ok(u64::from_le_bytes(buf));
            } else {
                return Ok(0u64)
            }
        }).collect::<Result<Vec<_>, Error>>()?;
    Ok(capacity_list.into_iter().sum::<u64>())
}

fn unlock_by_owner(selling_info: &SellingInfo) -> Result<bool, Error> {
    let is_owner_present = QueryIter::new(load_cell_lock, Source::Input)
        .find(|lock: &Script| {
            is_owner_of_selling_info(lock, selling_info)
        }).is_some();
    Ok(is_owner_present)
}

fn unlock_by_purchase(selling_info: &SellingInfo) -> Result<(), Error> {
    debug!("unlock by purchase");
    if validate_paying_price(selling_info)? && validate_no_additional_type(selling_info)? {
        return Ok(())
    }
    Err(Error::InvalidUnlock)
}

fn validate_paying_price(selling_info:&SellingInfo) -> Result<bool, Error> {
    let selling_price = load_selling_price(selling_info)?;
    debug!("selling price: {}", selling_price);
    let paying_price = load_paying_price(selling_info)?;
    debug!("paying price price: {}", paying_price);
    Ok(paying_price >= selling_price)
}

fn validate_no_additional_type(selling_info: &SellingInfo) -> Result<bool, Error> {
    let has_additional_type = QueryIter::new(load_cell, Source::Output)
        .find(|cell: &CellOutput| {
            is_selling_lock_from_same_owner(selling_info, &cell.lock()) && cell.type_().as_reader().is_some()
        }).is_some();
    if has_additional_type {
        return Err(Error::InvalidUnlock);
    }
    Ok(!has_additional_type)
}

pub fn main() -> Result<(), Error> {
    let selling_info: SellingInfo = load_selling_info()?;
    if unlock_by_owner(&selling_info)? {
        debug!("unlock by owner");
        return Ok(())
    }
    unlock_by_purchase(&selling_info)
}

