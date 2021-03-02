// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Config;

use sp_std::prelude::*;
use frame_benchmarking::{benchmarks};

use pallet_evm_precompile_modexp::Modexp;
use num::{BigUint, FromPrimitive};

extern crate hex;

benchmarks! {
	/*
	call {
		let caller: T::AccountId = whitelisted_caller();
	}: _(
			caller.clone(),
			H160::default(),
			H160::from_str("1000000000000000000000000000000000000001").unwrap(),
			Vec::new(),
			U256::default(),
			1000000,
			U256::default(),
			None
		)
	verify {
	}
	*/



	modexp {

		let b in 0 .. 10_000;
		let m in 0 .. 10_000;
		let e in 0 .. 10_000;

		// first 3 args declare the size of the following 3 args. we hard code these
		// and then build the following 3 args
		// TODO: don't hard code
		// TODO: see if large values impact runtime
		let size_args = hex::decode(
			"0000000000000000000000000000000000000000000000000000000000000100\
			0000000000000000000000000000000000000000000000000000000000000100\
			0000000000000000000000000000000000000000000000000000000000000100")
			.expect("Decode failed");

		assert!(size_args.len() == 96);

		// TODO: use benchmarking initialize fn
		let base_big = BigUint::from_u32(b).unwrap();
		let mod_big = BigUint::from_u32(m).unwrap();
		let exp_big = BigUint::from_u32(e).unwrap();

		let base_bytes = base_big.to_bytes_be();
		let mod_bytes = mod_big.to_bytes_be();
		let exp_bytes = exp_big.to_bytes_be();

		let mut input: [u8; (32 * 6)] = [0; (32 * 6)]; // room for 6 256 bit uints

		input[..96].copy_from_slice(&size_args);
		input[(96+(32*1))..].copy_from_slice(&base_bytes);
		input[(96+(32*2))..].copy_from_slice(&mod_bytes);
		input[(96+(32*3))..].copy_from_slice(&exp_bytes);

		let cost: u64 = 1;

	}:
	{
		<Modexp as LinearCostPrecompile>::execute(&input, cost)
			.expect("Modexp::execute() failed");
	} verify {
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{assert_ok};
	use crate::tests::{new_test_ext, Test};

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_modexp::<Test>());
		});
	}
}
