use crate as pallet_voting;
use crate::types::*;
use frame_system::EnsureRoot;
use frame_support::pallet_prelude::ConstU32;
use frame_support::traits::ConstU128;
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use frame_support::parameter_types;
use frame_support::PalletId;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, IdentifyAccount, Verify},
	MultiSignature,
};
use frame_support::pallet_prelude::*;
use sp_core::{sr25519, Pair, Public};

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

pub const UNIT: u128 = 1000000000000;
/// An index to a block.
pub type BlockNumber = u64;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// should be random, but we leave it const for simplicity
const SALT: u8 = 5u8;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		QuadraticVoting: pallet_voting::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

//let's make identity operations free-of-charge for testing purposes
parameter_types! {
	pub const BasicDeposit: Balance = 0;
	pub const FieldDeposit: Balance = 0;
	pub const SubAccountDeposit: Balance = 0;
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 1;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = ();
	type ForceOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Test>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<500>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

parameter_types! {
	pub const EntryFee: Balance = 30_000 * UNIT;
	pub const MaxProposals: u32 = 10u32;
	pub const RevealLength: BlockNumber = 50u64;
	pub const MinLength: BlockNumber = 100u64;
	pub const MaxTokens: u8 = 100u8;
	pub const VotingPalletId: PalletId = PalletId(*b"p/v8t1ng");
}

pub struct VotingIdentityProvider;
impl pallet_voting::IdentityProvider<AccountId> for VotingIdentityProvider {
	fn check_existence(account: &AccountId) -> bool {
		Identity::identity(account).is_some()
	}
}

impl pallet_voting::Config for Test {
	type Event = Event;
	type IdentityProvider = VotingIdentityProvider;
	type Currency = Balances;
	type BasicDeposit = EntryFee;
	type MaxProposals = MaxProposals;
	type Public = <Signature as Verify>::Signer;
	type Signature = MultiSignature;
	type RevealLength = RevealLength;
	type MinLength = MinLength;
	type MaxVotingTokens = MaxTokens;
	type PalletId = VotingPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(get_alice(), 1_000_000 * UNIT), (get_bob(), 1_000_000 * UNIT), (get_charlie(), 20_000 * UNIT)]
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}


pub fn get_alice() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("//Alice")
}

pub fn get_bob() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("//Bob")
}

pub fn get_charlie() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("//Charlie")
}

fn generate(key: &str, vote: Vote) -> String {
	let pair: sp_core::sr25519::Pair = Pair::from_string(key, None).unwrap();
	let payload = (vote, SALT).encode();
	let payload = payload.as_slice().to_owned();
	let signed = pair.sign(&payload);
	format!("{:02x?}", signed.0)
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}
