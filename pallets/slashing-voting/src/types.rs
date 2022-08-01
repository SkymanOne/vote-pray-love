use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};

use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Data {
	/// The data is stored directly.
	Raw(BoundedVec<u8, ConstU32<2048>>),
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct Proposal<AccountId, BlockNumber> {
	pub title: Data,
	pub proposer: AccountId,
	pub ayes: Vec<AccountId>,
	pub nays: Vec<AccountId>,
	pub end: BlockNumber,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum Vote {
	Yes,
	No,
}

/// To generate signature
///  ```
///  fn generate() -> String {
///     let pair: sp_core::sr25519::Pair = Pair::from_string("//Alice", None).unwrap();
///     let payload = (Vote::No, 10u32).encode();
///     let payload: [u8; 6] = payload.try_into().unwrap();
///     let signed = pair.sign(&payload).0;
///     format!("{:02x?}", signed)
/// }
/// ```

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct Commit<Signature> {
	pub signature: Signature,
	pub salt: u32,
}
