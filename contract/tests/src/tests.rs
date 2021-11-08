use super::*;
use ckb_testtool::ckb_error::Error;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder},
    packed::*,
    prelude::*,
};
const MAX_CYCLES: u64 = 10_000_000;

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
fn test_unlock_by_purchase() {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("selling-lock");
    let out_point = context.deploy_cell(contract_bin);

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
    let selling_lock_args = create_selling_lock_args(&owner_lock, 500u64);

    let lock_script = context
        .build_script(&out_point, Bytes::from(selling_lock_args))
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(out_point)
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input_pay_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(2000u64.pack())
            .lock(a_s_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let pay_cell_input = CellInput::new_builder()
        .previous_output(input_pay_cell_out_point)
        .build();
    let inputs = vec![input, pay_cell_input];
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(400u64.pack())
            .lock(a_s_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(1500u64.pack())
            .lock(owner_lock)
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(a_s_lock_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_unlock_by_owner() {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("selling-lock");
    let out_point = context.deploy_cell(contract_bin);

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
    let selling_lock_args = create_selling_lock_args(&owner_lock, 500u64);

    let lock_script = context
        .build_script(&out_point, Bytes::from(selling_lock_args))
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(out_point)
        .build();
    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input_pay_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(2000u64.pack())
            .lock(a_s_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let pay_cell_input = CellInput::new_builder()
        .previous_output(input_pay_cell_out_point)
        .build();
    let inputs = vec![input, pay_cell_input];
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(400u64.pack())
            .lock(a_s_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(1500u64.pack())
            .lock(owner_lock)
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(a_s_lock_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]

/*
 * build a tx with two selling lock inputs: 
 * inputs[0]: self_capacity=1000, price: 500:
 * inputs[1]: self_capacity=1000, price: 600
 * so that minimal pay price should be 1000 + 500 + 1000 + 600 = 3100
 */

fn verify_purchase_multiple_should_fail() {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("selling-lock");
    let out_point: OutPoint = context.deploy_cell(contract_bin);

    // deploy always_success script
    let always_success_out_point: OutPoint = context.deploy_cell(ALWAYS_SUCCESS.clone());

     // prepare scripts
   let lock_script_dep = CellDep::new_builder()
        .out_point(out_point.clone())
        .build();
    let a_s_lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let a_s_lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();

    let owner_lock =  context
        .build_script(&always_success_out_point, Bytes::from(vec![8]))
        .expect("script");
    let selling_lock_args_with_price_500 = create_selling_lock_args(&owner_lock, 500u64);
    let selling_lock_args_with_price_600 = create_selling_lock_args(&owner_lock, 600u64);

    let lock_script = context
        .build_script(&out_point, Bytes::from(selling_lock_args_with_price_500))
        .expect("script");
    let lock_script2 = context
        .build_script(&out_point, Bytes::from(selling_lock_args_with_price_600))
        .expect("script");

    // prepare cells
    let sell_lock_out_point1 = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let sell_lock_out_point2 = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script2.clone())
            .build(),
        Bytes::new(),
    );
    let input_pay_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(5000u64.pack())
            .lock(a_s_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input1 = CellInput::new_builder()
        .previous_output(sell_lock_out_point1)
        .build();
    let input2 = CellInput::new_builder()
        .previous_output(sell_lock_out_point2)
        .build();
    let pay_cell_input = CellInput::new_builder()
        .previous_output(input_pay_cell_out_point)
        .build();
    let inputs = vec![input1, input2, pay_cell_input];
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(200u64.pack())
            .lock(a_s_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(3000u64.pack())
            .lock(owner_lock)
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(a_s_lock_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let error: Error = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();
    println!("error: {:?}", error.kind());
}