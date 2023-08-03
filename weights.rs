
//! Autogenerated weights for `pallet_starknet`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-08-03, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/madara
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_starknet
// --extrinsic=infinite_loop
// --output=weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_starknet`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_starknet::WeightInfo for WeightInfo<T> {
	/// Storage: Starknet ContractClassHashes (r:2 w:0)
	/// Proof: Starknet ContractClassHashes (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Starknet FeeTokenAddress (r:1 w:0)
	/// Proof: Starknet FeeTokenAddress (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	/// Storage: Starknet SequencerAddress (r:1 w:0)
	/// Proof: Starknet SequencerAddress (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	/// Storage: Starknet Nonces (r:1 w:0)
	/// Proof: Starknet Nonces (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: Starknet ContractClasses (r:2 w:0)
	/// Proof: Starknet ContractClasses (max_values: None, max_size: Some(20971552), added: 20974027, mode: MaxEncodedLen)
	fn infinite_loop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `34008`
		//  Estimated: `41949044`
		// Minimum execution time: 4_500_824_000_000 picoseconds.
		Weight::from_parts(4_599_155_000_000, 0)
			.saturating_add(Weight::from_parts(0, 41949044))
			.saturating_add(T::DbWeight::get().reads(8))
	}
}
