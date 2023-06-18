use crate as omni_swap;
use core::ops::AddAssign;
use frame_support::{
	assert_ok,
	dispatch::DispatchError,
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, UnixTime},
};
use pallet_assets::{traits::OmniverseTokenFactoryHandler, FactoryResult};
use pallet_omniverse_protocol::{
	traits::OmniverseAccounts, OmniverseTransactionData, OmniverseTx, VerifyError, VerifyResult,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		OmniSwap: omni_swap,
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	static Frozen: HashMap<(u32, u64), u128> = Default::default();
	static Hooks: Vec<Hook> = Default::default();
}
pub struct TestFreezer;
impl pallet_assets::FrozenBalance<u32, u64, u128> for TestFreezer {
	fn frozen_balance(asset: u32, who: &u64) -> Option<u128> {
		Frozen::get().get(&(asset, *who)).cloned()
	}

	fn died(asset: u32, who: &u64) {
		Hooks::mutate(|v| v.push(Hook::Died(asset, *who)));

		// Sanity check: dead accounts have no balance.
		assert!(Assets::balance(asset, *who).is_zero());
	}
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
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
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	// type OmniverseToken = OmniverseTokenFactoryHandler;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = TestFreezer;
	type WeightInfo = ();
	type Extra = ();
	type OmniverseProtocol = OmniverseProtocol;
	type Timestamp = Timestamp;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Hook {
	Died(u32, u64),
}

#[derive(Default)]
pub struct OmniverseToken();

impl OmniverseTokenFactoryHandler for OmniverseToken {
	fn send_transaction_external(
		token_id: Vec<u8>,
		data: &OmniverseTransactionData,
	) -> Result<FactoryResult, DispatchError> {
		assert_ok!(Assets::send_transaction(
			RuntimeOrigin::signed(1),
			token_id.to_vec(),
			data.clone()
		));
		OmniverseProtocol::set_transaction_data(Some(OmniverseTx::new(
			data.clone(),
			Timestamp::now().as_secs(),
		)));
		Ok(FactoryResult::Success)
	}
}

impl omni_swap::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	// type OmniverseToken = Type;
	type OmniverseToken = OmniverseToken;
	type OmniverseProtocol = OmniverseProtocol;
}

// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	pallet_balances::GenesisConfig::<Test> { balances: vec![(1, 1024), (2, 10000)] }
		.assimilate_storage(&mut storage)
		.unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	// Clear thread local vars for https://github.com/paritytech/substrate/issues/10479.
	ext.execute_with(|| take_hooks());
	ext.execute_with(|| System::set_block_number(1));
	ext
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
		0
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

	fn execute(_pk: [u8; 64], _pallet_name: Vec<u8>, _token_id: Vec<u8>, _nonce: u128) {
		unsafe {
			match TRANSACTION_DATA.as_mut() {
				Some(tx_data) => tx_data.executed = true,
				None => {},
			}
		}
	}
}

pub(crate) fn take_hooks() -> Vec<Hook> {
	Hooks::take()
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
