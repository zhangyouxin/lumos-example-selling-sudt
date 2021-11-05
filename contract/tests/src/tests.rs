use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder},
    packed::*,
    prelude::*,
};

const MAX_CYCLES: u64 = 10_000_000;

#[test]
fn test_unlock_by_purchase() {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("selling-lock");
    let out_point = context.deploy_cell(contract_bin);

    // deploy always_success script
    let always_success_out_point: OutPoint = context.deploy_cell(ALWAYS_SUCCESS.clone());

    static ARGS_BYTES: &'static [u8] = &[48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 49, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 244, 1, 0, 0, 0, 0, 0, 0 ];
    static OWNER_LOCK_CODE_HASH: &'static [u8] = &[48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49];
    static OWNER_LOCK_ARGS: &'static [u8] = &[48, 49, 50, 51, 52, 53, 54, 55, 56, 57];
    static OWNER_LOCK_HASH_TYPE: &'static [u8] = &[49];
     // prepare scripts
    let lock_script = context
        .build_script(&out_point, Bytes::from(ARGS_BYTES))
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(out_point)
        .build();

    let a_s_lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let a_s_lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let owner_lock = ScriptBuilder::default()
                            .code_hash(Byte32::from_slice(OWNER_LOCK_CODE_HASH).unwrap())
                            .args(Bytes::from(OWNER_LOCK_ARGS).pack())
                            .hash_type(Byte::from_slice(OWNER_LOCK_HASH_TYPE).unwrap());
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
            .lock(owner_lock.build())
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

    static ARGS_BYTES: &'static [u8] = &[
        230, 131, 176, 65, 57, 52, 71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131, 31, 54, 96, 215,
        2,
        8,
        244, 1, 0, 0, 0, 0, 0, 0 
        ];
    static OWNER_LOCK_CODE_HASH: &'static [u8] = &[230, 131, 176, 65, 57, 52, 71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131, 31, 54, 96, 215];
    static OWNER_LOCK_ARGS: &'static [u8] = &[8];
    static OWNER_LOCK_HASH_TYPE: &'static [u8] = &[2];
     // prepare scripts
    let lock_script = context
        .build_script(&out_point, Bytes::from(ARGS_BYTES))
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(out_point)
        .build();

    let a_s_lock_script = context
        .build_script(&always_success_out_point, Bytes::from(vec![8]))
        .expect("script");
    let a_s_lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let owner_lock = ScriptBuilder::default()
                            .code_hash(Byte32::from_slice(OWNER_LOCK_CODE_HASH).unwrap())
                            .args(Bytes::from(OWNER_LOCK_ARGS).pack())
                            .hash_type(Byte::from_slice(OWNER_LOCK_HASH_TYPE).unwrap());
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
            .lock(owner_lock.build())
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
fn test_sell_by_owner() {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("selling-lock");
    let out_point = context.deploy_cell(contract_bin);

    // deploy always_success script
    let always_success_out_point: OutPoint = context.deploy_cell(ALWAYS_SUCCESS.clone());

    static ARGS_BYTES: &'static [u8] = &[
        230, 131, 176, 65, 57, 52, 71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131, 31, 54, 96, 215,
        2,
        8,
        244, 1, 0, 0, 0, 0, 0, 0 
        ];
    static OWNER_LOCK_CODE_HASH: &'static [u8] = &[230, 131, 176, 65, 57, 52, 71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131, 31, 54, 96, 215];
    static OWNER_LOCK_ARGS: &'static [u8] = &[8];
    static OWNER_LOCK_HASH_TYPE: &'static [u8] = &[2];
     // prepare scripts
    let lock_script = context
        .build_script(&out_point, Bytes::from(ARGS_BYTES))
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(out_point)
        .build();

    let a_s_lock_script = context
        .build_script(&always_success_out_point, Bytes::from(vec![8]))
        .expect("script");
    let a_s_lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let owner_lock = ScriptBuilder::default()
                            .code_hash(Byte32::from_slice(OWNER_LOCK_CODE_HASH).unwrap())
                            .args(Bytes::from(OWNER_LOCK_ARGS).pack())
                            .hash_type(Byte::from_slice(OWNER_LOCK_HASH_TYPE).unwrap());
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
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(1500u64.pack())
            .lock(owner_lock.build())
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