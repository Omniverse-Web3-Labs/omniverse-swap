// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test environment for Uniques pallet.

use super::*;
use crate as pallet_uniques;
use std::ops::AddAssign;
use std::time::{Duration, SystemTime};

use frame_support::{
	construct_runtime,
	traits::{AsEnsureOriginWithArg, ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use pallet_omniverse_protocol::OmniverseTx;
use pallet_omniverse_protocol::{
	traits::OmniverseAccounts, OmniverseTransactionData, VerifyError, VerifyResult,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OmniverseProtocol = OmniverseProtocol;
	type Timestamp = Timestamp;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type Locker = ();
	type CollectionDeposit = ConstU64<2>;
	type ItemDeposit = ConstU64<1>;
	type MetadataDepositBase = ConstU64<1>;
	type AttributeDepositBase = ConstU64<1>;
	type DepositPerByte = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type KeyLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub static mut TIME_PAST: u64 = 0;

pub struct Timestamp {}

impl Timestamp {
	pub fn past(t: u64) {
		unsafe {
			TIME_PAST = TIME_PAST + t;
		}
	}
}

impl UnixTime for Timestamp {
	fn now() -> core::time::Duration {
		unsafe {
			let mut now = SystemTime::now();
			let dur = Duration::from_secs(TIME_PAST);
			now.add_assign(dur);
			now.duration_since(SystemTime::UNIX_EPOCH).unwrap()
		}
	}
}

pub static mut TRANSACTION_DATA: Option<OmniverseTx> = None;

#[derive(Default)]
pub struct OmniverseProtocol();

impl OmniverseProtocol {
	pub fn set_transaction_data(tx_data: Option<OmniverseTx>) {
		unsafe {
			TRANSACTION_DATA = tx_data;
		}
	}
}

impl OmniverseAccounts for OmniverseProtocol {
	fn verify_transaction(
		_pallet_name: &[u8],
		_token_id: &[u8],
		data: &OmniverseTransactionData,
		_with_ethereum: bool,
	) -> Result<VerifyResult, VerifyError> {
		if data.signature == [0; 65] {
			return Err(VerifyError::SignatureError);
		}

		Ok(VerifyResult::Success)
	}

	fn get_transaction_count(_pk: [u8; 64], _pallet_name: Vec<u8>, _token_id: Vec<u8>) -> u128 {
		0u128
	}

	fn is_malicious(_pk: [u8; 64]) -> bool {
		false
	}

	fn get_chain_id() -> u32 {
		1
	}

	fn get_transaction_data(
		_pk: [u8; 64],
		_pallet_name: Vec<u8>,
		_token_id: Vec<u8>,
		_nonce: u128,
	) -> Option<OmniverseTx> {
		unsafe { TRANSACTION_DATA.clone() }
	}
}
