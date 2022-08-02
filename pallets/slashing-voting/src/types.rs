use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};

use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::prelude::*;

pub type VoteToken = u8;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Data {
	/// The data is stored directly.
	Raw(BoundedVec<u8, ConstU32<2048>>),
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Proposal<AccountId, BlockNumber, Balance> {
	/// The title of proposal
	pub title: Data,
	/// Who proposed
	pub proposer: AccountId,
	/// Total votes for proposal to pass
	pub ayes: u32,
	/// Total votes for proposal to get rejected
	pub nays: u32,
	/// The hard end of voting phase
	pub poll_end: BlockNumber,
	/// The hard end of reveal phase
	pub reveal_end: Option<BlockNumber>,
	/// The number of votes each voter gave
	pub votes: Vec<(AccountId, u8, Vote)>,
	/// Users who revealed their choices.
	/// Allows to verify who did not reveal on time.
	/// This may look as data duplication, but it will reduce runtime
	/// otherwise we need to parse `votes` vector and compose vector of required format
	pub revealed: Vec<AccountId>,
	/// The amount that was slashed and distributed
	pub payout: Balance,
	/// Is proposal closed
	pub closed: bool
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum Vote {
	Yes,
	No,
}


#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct Commit<Signature> {
	/// The signed choice of a voter
	pub signature: Signature,
	/// The number of votes the voter gives to their choice.
	/// Must be exposed and unencrypted to allow double spend of votes
	pub number: u8,
	/// Salt which comes with the choice to ensure the security
	pub salt: u32,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, Default)]
pub struct VoterBalance<Balance> {
	/// The number of votes the voter gives to their choice.
	/// Must be exposed and unencrypted to allow double spend of votes
	pub voting_tokens: VoteToken,
	/// Salt which comes with the choice to ensure the security
	pub reserved_balance: Balance,
}
