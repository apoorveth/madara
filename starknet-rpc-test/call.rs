#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use rstest::{fixture, rstest};
use starknet_accounts::Account;
use starknet_contract::ContractFactory;
use starknet_core::types::contract::SierraClass;
use starknet_core::types::{BlockId, BlockTag, FunctionCall, StarknetError};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::utils::create_account;
use starknet_rpc_test::{ExecutionStrategy, MadaraClient, Transaction};

#[fixture]
async fn madara() -> MadaraClient {
    MadaraClient::new(ExecutionStrategy::Native).await
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert_matches!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("name").unwrap(),
                calldata: vec![]
            },
            BlockId::Hash(FieldElement::ZERO)
        )
        .await
        .err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::BlockNotFound)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_entrypoint(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert_matches!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: FieldElement::from_hex_be("0x0").unwrap(),
                calldata: vec![]
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::ContractError)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_incorrect_calldata(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert_matches!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("name").unwrap(),
                calldata: vec![FieldElement::ONE] // name function has no calldata
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::ContractError)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_on_correct_call_no_calldata(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert_eq!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("name").unwrap(),
                calldata: vec![] // name function has no calldata
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .unwrap(),
        vec![FieldElement::ZERO]
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_on_correct_call_with_calldata(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("balanceOf").unwrap(),
                calldata: vec![FieldElement::TWO] // name function has no calldata
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .unwrap()[0]
            .gt(&FieldElement::ZERO)
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_on_mutable_call_without_modifying_storage(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;
    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS);

    let contract_artifact: SierraClass = serde_json::from_reader(
        std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/contracts/HelloStarknet.sierra.json").unwrap(),
    )
    .unwrap();

    let declaration = account.declare(
        contract_artifact.clone().flatten().unwrap().into(),
        FieldElement::from_hex_be("0xdf4d3042eec107abe704619f13d92bbe01a58029311b7a1886b23dcbb4ea87").unwrap(), // compiled class hash
    );
    let contract_factory = ContractFactory::new(contract_artifact.class_hash().unwrap(), account.clone());

    let deployment = contract_factory.deploy(vec![], FieldElement::ZERO, true);

    // declare and deploy contract
    madara.create_block_with_txs(vec![Transaction::Declaration(declaration)]).await?;
    madara.create_block_with_txs(vec![Transaction::Execution(deployment)]).await?;

    // address of deployed contract (will always be the same for 0 salt)
    let contract_address =
        FieldElement::from_hex_be("0x335e244fc6f5752ab93c1c86e9bd714b413d378dc4860732644bc20626c6c51").unwrap();

    let read_balance = || async {
        rpc.call(
            FunctionCall {
                contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap()
    };

    let initial_balance = read_balance().await[0];
    // call increase_balance and verify it returns a result
    assert!(
        rpc.call(
            FunctionCall {
                contract_address,
                entry_point_selector: get_selector_from_name("increase_balance").unwrap(),
                calldata: vec![FieldElement::ONE]
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .is_ok()
    );
    let final_balance = read_balance().await[0];

    assert_eq!(initial_balance, final_balance);

    Ok(())
}