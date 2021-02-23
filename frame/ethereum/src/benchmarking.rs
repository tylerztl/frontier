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
use mock::*;
use sp_std::prelude::*;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use ethereum::{Transaction};

benchmarks! {
	transact {
		let caller: T::AccountId = whitelisted_caller();
		let txn: UnsignedTransaction = UnsignedTransaction {
			nonce: 0,
			gas_price: 0,
			gas_limit: 0,
			action: TransactionAction::Call,
			value: 0,
			input: vec![],
		};
	}: _(RawOrigin::Signed(caller.clone()), txn.sign(caller))
	verify {
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{new_test_ext, Test};
	use frame_support::{assert_ok, assert_err};

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_transact::<Test>());
		});
	}
}
