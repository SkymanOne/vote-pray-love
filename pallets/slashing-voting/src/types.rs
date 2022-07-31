use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{ConstU32},
	BoundedVec,
};

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
	title: Data,
	proposer: AccountId,
	ayes: Vec<AccountId>,
	nays: Vec<AccountId>,
	end: BlockNumber,
}
