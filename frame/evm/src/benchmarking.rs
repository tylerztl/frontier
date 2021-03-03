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
use sp_std::fmt;
use frame_benchmarking::{benchmarks};
use frame_support::ensure;

use pallet_evm_precompile_modexp::Modexp;
use num::{BigUint, Zero, One, FromPrimitive};
use core::convert::From;

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

		let b in 2 .. 40_000_000;
		let e in 101 .. 40_000_000;
		let m in 3 .. 40_000_000;

		frame_support::runtime_print!("=============================================================================");
		frame_support::runtime_print!("Running modexp, b = {}, e = {}, m = {}", b, e, m);

		// TODO: use benchmarking initialize fn
		let base_big = BigUint::from_u32(b).unwrap();
		let exp_big = BigUint::from_u32(e).unwrap();
		let mod_big = BigUint::from_u32(m).unwrap();

		frame_support::runtime_print!("base_big = {}, exp_big = {}, mod_big = {}",
									  base_big, exp_big, mod_big);

		let base_bytes = base_big.to_bytes_be();
		let exp_bytes = exp_big.to_bytes_be();
		let mod_bytes = mod_big.to_bytes_be();

		let mut input: [u8; (32 * 6)] = [0; (32 * 6)]; // room for 6 256 bit uints

		let base_size_in_bytes: U256 = U256::from(base_bytes.len());
		let exp_size_in_bytes: U256 = U256::from(exp_bytes.len());
		let mod_size_in_bytes: U256 = U256::from(mod_bytes.len());

		frame_support::runtime_print!("base_size = {}, exp_size = {}, mod_size = {}",
									  base_size_in_bytes, exp_size_in_bytes, mod_size_in_bytes);

		base_size_in_bytes.to_big_endian(&mut input[..32]);
		exp_size_in_bytes.to_big_endian(&mut input[32..64]);
		mod_size_in_bytes.to_big_endian(&mut input[64..96]);

		let base_start: usize = 96;
		let exp_start: usize = base_start + base_bytes.len();
		let mod_start: usize = exp_start + exp_bytes.len();

		input[base_start..(base_start + base_bytes.len())].copy_from_slice(&base_bytes);
		input[exp_start..(exp_start + exp_bytes.len())].copy_from_slice(&exp_bytes);
		input[mod_start..(mod_start + mod_bytes.len())].copy_from_slice(&mod_bytes);

		let cost: u64 = 1;

		// TODO: hack to have result in scope for verify block
		/*
		let mut result: core::result::Result<(ExitSucceed, Vec<u8>), ExitError>
			= Err(ExitError::Other("FIXME".into()));
			*/

	}:
	{
		let result = <Modexp as LinearCostPrecompile>::execute(&input, cost);

		let (_, response) = result.expect("Modexp::execute() errored");
		ensure!(
			U256::from(response.len()) == mod_size_in_bytes,
			"Unexpected response size");

		let value = if mod_big.is_zero() || mod_big.is_one() {
			BigUint::zero()
		} else {
			BigUint::from_bytes_be(&response[..])
		};

		frame_support::runtime_print!("calculating base.modpow. {} ^ {} % {} = {}",
									  base_big, exp_big, mod_big, base_big.modpow(&exp_big, &mod_big));

		let expected = base_big.modpow(&exp_big, &mod_big);
		if value != expected {
			frame_support::runtime_print!("Modexp produced unexpected value: {} vs. {}", value, expected);
		}
		ensure!(value == expected, "Modexp produced unexpected value");
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
