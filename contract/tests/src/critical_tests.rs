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

#[test]
fn test_diffrent_seller_in_single_tx() {
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

    let owner_lock_a = context
        .build_script(&always_success_out_point, Bytes::from(vec![8]))
        .expect("script");
    let owner_lock_b = context
        .build_script(&always_success_out_point, Bytes::from(vec![9]))
        .expect("script");

    let selling_lock_a = context
        .build_script(&out_point, create_selling_lock_args(&owner_lock_a, 500u64))
        .expect("script");
    let selling_lock_b = context
        .build_script(&out_point, create_selling_lock_args(&owner_lock_b, 600u64))
        .expect("script");
    let input_selling_locks = vec![&selling_lock_a, &selling_lock_b];
    // prepare inputs and assign 1000 Bytes to per input
    // alice has enough capacity to pay
    let input_pay_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(3000u64.pack())
            .lock(owner_lock_a.clone())
            .build(),
        Bytes::new(),
    );
    // selling_lock_a and selling_lock_b
    let inputs = input_selling_locks.iter().map(|selling_lock| {
        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(SELF_CAPACITY.pack())
                .lock((*selling_lock).clone())
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

    let payment_output_b = CellOutput::new_builder()
        .capacity(600u64.pack())
        .lock(owner_lock_b.clone())
        .build();

    let outputs_data = vec![Bytes::new(); 1];
    // build transaction
    let tx = TransactionBuilder::default()
    .input(input_pay_cell)
    .inputs(inputs)
    .output(payment_output_b)
    .outputs_data(outputs_data.pack())
    .cell_dep(lock_script_dep)
    .cell_dep(a_s_lock_script_dep)
    .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("cycles: {}", cycles);
}
