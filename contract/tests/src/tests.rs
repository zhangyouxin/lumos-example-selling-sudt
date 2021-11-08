use super::*;
use ckb_testtool::ckb_error::Error;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
const MAX_CYCLES: u64 = 10_000_000;
const SELF_CAPACITY: u64 = 1_000;

#[derive(PartialEq, Eq)]
enum UnlockMode {
    Owner,
    Purchase,
}

fn create_selling_lock_args(owner_lock: &Script,  price: u64) -> Bytes {
    let owner_lock_code_hash: Byte32 = owner_lock.code_hash();
    let owner_lock_hash_type: Byte = owner_lock.hash_type();
    let owner_lock_args: Bytes = owner_lock.args().unpack();
    let price_vec = price.to_le_bytes().to_vec();

    Bytes::from(
        owner_lock_code_hash.as_slice()
            .iter()
            .chain(owner_lock_hash_type.as_slice())
            .chain(owner_lock_args.as_ref())
            .chain(price_vec.as_slice())
            .cloned()
            .collect::<Vec<u8>>(),
    )
}

fn build_test_context(
    inputs_price: Vec<u64>,
    pay_price: u64,
    unlock_mode: UnlockMode,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("selling-lock");
    let out_point = context.deploy_cell(contract_bin);
    let lock_script_dep = CellDep::new_builder()
        .out_point(out_point.clone())
        .build();

    // deploy always_success script
    let always_success_out_point: OutPoint = context.deploy_cell(ALWAYS_SUCCESS.clone());
    
    // prepare scripts
    let a_s_lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let a_s_lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();

    let owner_lock = context
        .build_script(&always_success_out_point, Bytes::from(vec![8]))
        .expect("script");

    let input_selling_locks = inputs_price
        .iter()
        .map(|price| {
            let args = create_selling_lock_args(&owner_lock, *price);
            context
                .build_script(&out_point, args)
                .expect("script")
        })
        .collect::<Vec<Script>>();
    // prepare inputs and assign 1000 Bytes to per input
    let mut payment_cell_lock = a_s_lock_script.clone();
    if unlock_mode == UnlockMode::Owner {
        payment_cell_lock = owner_lock.clone();
    }
    let input_pay_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(pay_price.pack())
            .lock(payment_cell_lock)
            .build(),
        Bytes::new(),
    );
    let inputs = input_selling_locks.iter().map(|selling_lock| {
        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(SELF_CAPACITY.pack())
                .lock(selling_lock.clone())
                .build(),
            Bytes::new(),
        );
        let input = CellInput::new_builder()
            .previous_output(input_out_point)
            .build();
        input
    });

    let input_pay_cell = CellInput::new_builder()
        .previous_output(input_pay_cell_out_point)
        .build();

    let payment_output = CellOutput::new_builder()
        .capacity(pay_price.pack())
        .lock(owner_lock.clone())
        .build();

    let outputs_data = vec![Bytes::new(); 1];
    // build transaction
    let tx = TransactionBuilder::default()
    .input(input_pay_cell)
    .inputs(inputs)
    .output(payment_output)
    .outputs_data(outputs_data.pack())
    .cell_dep(lock_script_dep)
    .cell_dep(a_s_lock_script_dep)
    .build();

    (context, tx)
}

#[test]
fn test_unlock_by_owner() {
    let (mut context, tx) = build_test_context(vec![1000], 100, UnlockMode::Owner);
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("cycles: {}", cycles);
}

#[test]
fn test_unlock_by_purchase() {
    let (mut context, tx) = build_test_context(vec![1000], 2000, UnlockMode::Purchase);
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("cycles: {}", cycles);
}

#[test]
fn test_unlock_by_purchase_multiple() {
    let (mut context, tx) = build_test_context(vec![500, 600], 3000, UnlockMode::Purchase);
    let tx = context.complete_tx(tx);

    // should fail cause minimal pay price is 3100
    let error: Error = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();
    println!("error: {:?}", error.kind());
}
