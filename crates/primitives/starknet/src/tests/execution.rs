use alloc::sync::Arc;

use blockifier::abi::abi_utils::selector_from_name;
use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::errors::{EntryPointExecutionError, VirtualMachineExecutionError};
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::errors::vm_exception::VmException;
use frame_support::{assert_err, assert_ok, bounded_vec};
use starknet_api::api_core::{ChainId, ClassHash, ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::Calldata;
use starknet_api::{patricia_key, stark_felt};
use thiserror_no_std::Error;

use crate::constants::INITIAL_GAS;
use crate::execution::call_entrypoint_wrapper::CallEntryPointWrapper;
use crate::execution::entrypoint_wrapper::{EntryPointExecutionErrorWrapper, EntryPointTypeWrapper};
use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::tests::utils::{create_test_state, TEST_CLASS_HASH, TEST_CONTRACT_ADDRESS};

#[test]
fn test_call_entry_point_execute_works() {
    let mut test_state = create_test_state();

    let class_hash = Felt252Wrapper::from_hex_be(TEST_CLASS_HASH).unwrap();
    let address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = selector_from_name("return_result").0.into();
    let calldata = bounded_vec![42_u128.into()];

    let entrypoint = CallEntryPointWrapper::new(
        Some(class_hash),
        EntryPointTypeWrapper::External,
        Some(selector),
        calldata,
        address,
        ContractAddressWrapper::default(),
        INITIAL_GAS.into(),
        None,
    );

    let block_context = BlockContext {
        chain_id: ChainId("0x1".to_string()),
        block_number: BlockNumber(0),
        block_timestamp: BlockTimestamp(0),
        sequencer_address: ContractAddress::default(),
        fee_token_address: ContractAddress::default(),
        vm_resource_fee_cost: Default::default(),
        gas_price: 0,
        invoke_tx_max_n_steps: 1000000,
        validate_max_n_steps: 1000000,
        max_recursion_depth: 50,
    };

    assert_ok!(entrypoint.execute(&mut test_state, block_context));
}

#[test]
fn test_call_entry_point_fails_insufficient_steps() {
    let mut test_state = create_test_state();

    let class_hash = Felt252Wrapper::from_hex_be(TEST_CLASS_HASH).unwrap();
    let address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = selector_from_name("return_result").0.into();
    let calldata = bounded_vec![42_u128.into()];

    let entrypoint = CallEntryPointWrapper::new(
        Some(class_hash),
        EntryPointTypeWrapper::External,
        Some(selector),
        calldata,
        address,
        ContractAddressWrapper::default(),
        Felt252Wrapper::default(),
        None,
    );

    let block_context = BlockContext {
        chain_id: ChainId("0x1".to_string()),
        block_number: BlockNumber(0),
        block_timestamp: BlockTimestamp(0),
        sequencer_address: ContractAddress::default(),
        fee_token_address: ContractAddress::default(),
        vm_resource_fee_cost: Default::default(),
        gas_price: 0,
        invoke_tx_max_n_steps: 0,
        validate_max_n_steps: 1000000,
        max_recursion_depth: 50,
    };

    match entrypoint.execute(&mut test_state, block_context) {
        Ok(_) => panic!("Expected an error"),
        Err(EntryPointExecutionErrorWrapper::EntryPointExecution(
            EntryPointExecutionError::VirtualMachineExecutionErrorWithTrace {
                trace: _,
                source:
                    VirtualMachineExecutionError::CairoRunError(CairoRunError::VmException(VmException {
                        pc: _,
                        inst_location: _,
                        inner_exc,
                        error_attr_value: _,
                        traceback: _,
                    })),
            },
        )) => {
            assert!(matches!(inner_exc, VirtualMachineError::UnfinishedExecution));
        }
        _ => panic!("Unexpected error type"),
    }
}

#[test]
fn test_call_entry_point_execute_fails_undeclared_class_hash() {
    let mut test_state = create_test_state();

    let address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = selector_from_name("return_result").0.into();
    let calldata = bounded_vec![42_u128.into()];

    let entrypoint = CallEntryPointWrapper::new(
        Some(Felt252Wrapper::ZERO),
        EntryPointTypeWrapper::External,
        Some(selector),
        calldata,
        address,
        ContractAddressWrapper::default(),
        INITIAL_GAS.into(),
        None,
    );

    let block_context = BlockContext {
        chain_id: ChainId("0x1".to_string()),
        block_number: BlockNumber(0),
        block_timestamp: BlockTimestamp(0),
        sequencer_address: ContractAddress::default(),
        fee_token_address: ContractAddress::default(),
        vm_resource_fee_cost: Default::default(),
        gas_price: 0,
        invoke_tx_max_n_steps: 0,
        validate_max_n_steps: 0,
        max_recursion_depth: 0,
    };

    assert!(entrypoint.execute(&mut test_state, block_context).is_err());
}

#[test]
fn test_try_into_entrypoint_default() {
    let entrypoint_wrapper = CallEntryPointWrapper::default();
    let entrypoint: CallEntryPoint = entrypoint_wrapper.try_into().unwrap();
    pretty_assertions::assert_eq!(entrypoint, CallEntryPoint::default());
}

#[test]
fn test_try_into_entrypoint_works() {
    let entrypoint_wrapper = CallEntryPointWrapper {
        class_hash: Some(Felt252Wrapper::from_hex_be("0x1").unwrap()),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec![Felt252Wrapper::ONE, Felt252Wrapper::TWO, Felt252Wrapper::THREE],
        storage_address: Felt252Wrapper::from_hex_be("0x1").unwrap(),
        caller_address: Felt252Wrapper::from_hex_be("0x2").unwrap(),
        initial_gas: INITIAL_GAS.into(),
        compiled_class_hash: None,
    };
    let entrypoint: CallEntryPoint = entrypoint_wrapper.try_into().unwrap();
    let expected_entrypoint = CallEntryPoint {
        call_type: CallType::Call,
        calldata: Calldata(Arc::new(vec![stark_felt!(1_u8), stark_felt!(2_u8), stark_felt!(3_u8)])),
        caller_address: ContractAddress(patricia_key!(2_u8)),
        storage_address: ContractAddress(patricia_key!(1_u8)),
        class_hash: Some(ClassHash(stark_felt!(1_u8))),
        code_address: None,
        entry_point_selector: EntryPointSelector(stark_felt!(0_u8)),
        entry_point_type: EntryPointType::External,
        initial_gas: INITIAL_GAS.into(),
    };

    pretty_assertions::assert_eq!(entrypoint, expected_entrypoint);
}
