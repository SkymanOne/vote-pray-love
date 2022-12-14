#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::traits::Get;
use frame_support::traits::ReservableCurrency;
use frame_support::BoundedVec;
pub use pallet::*;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::CheckedDiv;
use sp_runtime::traits::Saturating;
use sp_runtime::DispatchError;
use sp_std::borrow::ToOwned;
use sp_std::vec::Vec;
pub mod types;

#[frame_support::pallet]
pub mod pallet {

	use core::cmp::Ordering;

	use crate::types::{Commit, Data, Proposal, Vote, VoteToken, VoterBalance};
	use frame_support::dispatch::DispatchResult;
	use frame_support::ensure;
	use frame_support::pallet_prelude::CountedStorageMap;
	use frame_support::pallet_prelude::StorageDoubleMap;
	use frame_support::pallet_prelude::StorageMap;
	use frame_support::sp_runtime::traits::Hash;
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_support::{
		pallet_prelude::{ValueQuery, *},
		Identity, PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_runtime::traits::{IdentifyAccount, Member, Verify};
	use sp_std::boxed::Box;
	use sp_std::vec;
	use sp_std::vec::Vec;

	pub type MemberCount = u32;
	pub type ProposalIndex = u32;

	/// Shorted type for extracting current balance of a user
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub trait IdentityProvider<AccountId> {
		fn check_existence(account: &AccountId) -> bool;
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// general event that happens in the system
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// glueing trait that provides bridge to identity pallet
		/// In other words, allows to interact with Identity component
		type IdentityProvider: IdentityProvider<Self::AccountId>;
		/// Currency type, required to manipulate voters balances and deposits
		type Currency: ReservableCurrency<Self::AccountId>;
		/// The amount of funds that is required to have skin in a game
		#[pallet::constant]
		type BasicDeposit: Get<BalanceOf<Self>>;
		/// The length of reveal phase
		#[pallet::constant]
		type RevealLength: Get<Self::BlockNumber>;
		/// Maximum number of proposals allowed to be active in parallel.
		#[pallet::constant]
		type MaxProposals: Get<ProposalIndex>;
		/// Minimum length of proposal
		#[pallet::constant]
		type MinLength: Get<Self::BlockNumber>;
		/// Minimum length of proposal
		#[pallet::constant]
		type MaxVotingTokens: Get<u8>;
		// Public ket type to identify accounts and verify signatures
		type Public: IdentifyAccount<AccountId = Self::AccountId>;
		// Signature type to verify signed votes
		type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;
		/// The council's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Some identity joined the voting committee
		Joined(T::AccountId),
		/// Some identity left the voting committee
		Left { account: T::AccountId, cashout: BalanceOf<T> },
		/// A motion (given hash) has been proposed (by given account)
		Proposed { account: T::AccountId, proposal_hash: T::Hash },
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Voted { account: T::AccountId, proposal_hash: T::Hash },
		/// A motion (given hash) has been committed on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Committed { account: T::AccountId, proposal_hash: T::Hash },
		/// The reveal phase was finished
		ClosedCommit(T::Hash),
		/// Proposal has been approved
		Approved(T::Hash),
		/// Proposal has nit been approved
		Disapproved(T::Hash),
		/// No consensus has been reached in motion
		Tie(T::Hash),
		/// The voting phase was closed
		ClosedReveal {
			proposal_hash: T::Hash,
			yes: MemberCount,
			no: MemberCount,
			revealed: MemberCount,
			payout: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is not a member
		NotMember,
		/// Account is a already a member
		AlreadyMember,
		/// Account does not have an identity
		NoIdentity,
		/// Duplicate proposals not allowed
		DuplicateProposal,
		/// Proposal must exist
		ProposalMissing,
		/// Invalid Argument was supplied
		InvalidArgument,
		/// Duplicate vote ignored
		DuplicateVote,
		/// Members are already initialized!
		AlreadyInitialized,
		/// There can only be a maximum of `MaxProposals` active proposals.
		TooManyProposals,
		/// The given length bound for the proposal was too low.
		WrongProposalLength,
		/// Not enough funds to join the voting council
		NotEnoughFunds,
		/// Voter does not have enough voting tokens to submit a vote
		NotEnoughVotingTokens,
		/// Voting phase ended
		VoteEnded,
		/// Vote has already ended when trying to close it
		VoteAlreadyEnded,
		/// Too early to do action
		TooEarly,
		/// Reveal phase has not yet started
		RevealNotStarted,
		/// Reveal phase ended, proposal finished
		RevealEnded,
		/// No commit has been submitted
		NoCommit,
		/// Could not verify signature of a commit
		SignatureInvalid,
		/// The voter is in the middle of vote
		InMotion,
	}

	//we use unbounded storage because we size of council can vary
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	/// Collection of all proposals hashes
	#[pallet::storage]
	pub type Proposals<T: Config> =
		StorageValue<_, BoundedVec<T::Hash, T::MaxProposals>, ValueQuery>;
	/// The actual data of proposal
	#[pallet::storage]
	pub type ProposalData<T: Config> =
		StorageMap<_, Identity, T::Hash, Proposal<T::AccountId, T::BlockNumber, BalanceOf<T>>>;
	/// The list of council member with their voting tokens
	#[pallet::storage]
	pub type Members<T: Config> =
		CountedStorageMap<_, Identity, T::AccountId, VoterBalance<BalanceOf<T>>, ValueQuery>;
	/// Vote commits submitted by voters
	#[pallet::storage]
	pub type Commits<T: Config> =
		StorageDoubleMap<_, Identity, T::AccountId, Identity, T::Hash, Commit<T::Signature>>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig;

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			// Create pot account
			let account_id = <Pallet<T>>::account_id();
			let min = T::Currency::minimum_balance();
			if T::Currency::free_balance(&account_id) < min {
				let _ = T::Currency::make_free_balance_be(&account_id, min);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Join committee and deposit money to have skin in a game
		#[pallet::weight(10_000_000)]
		pub fn join_committee(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer is a member already | tested
			ensure!(!Self::is_member(&signer), Error::<T>::AlreadyMember);

			//check if signer has identity | tested
			ensure!(T::IdentityProvider::check_existence(&signer), Error::<T>::NoIdentity);

			//check if the account has enough money to deposit
			ensure!(
				T::Currency::can_reserve(&signer, T::BasicDeposit::get()),
				Error::<T>::NotEnoughFunds
			);

			T::Currency::reserve(&signer, T::BasicDeposit::get())?;

			//deposit 100 voting tokens to the voter
			Self::deposit_votes(&signer, T::MaxVotingTokens::get());

			//reserve the fixed amount specified in the config
			Self::set_reserved_balance(&signer, T::BasicDeposit::get());

			Self::deposit_event(Event::<T>::Joined(signer));

			Ok(())
		}

		/// leave committee and cash out
		#[pallet::weight(10_000_000)]
		pub fn leave_committee(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer has identity | tested
			ensure!(T::IdentityProvider::check_existence(&signer), Error::<T>::NoIdentity);

			// ensure that user is not in the middle of voting process
			let active_votes = <Commits<T>>::iter_prefix_values(signer.clone()).count();
			ensure!(active_votes == 0, Error::<T>::InMotion);

			// find the exact amount of reserved funds that need to be returned to the free balance
			let reserved_balance = <Members<T>>::get(signer.clone()).reserved_balance;
			let balance = T::Currency::unreserve(&signer, reserved_balance);
			//remove entries
			<Members<T>>::remove(signer.clone());

			Self::deposit_event(Event::<T>::Left { account: signer, cashout: balance });

			Ok(())
		}

		/// Creates the proposal with given text and duration in blocks
		#[pallet::weight(10_000_000)]
		pub fn create_proposal(
			origin: OriginFor<T>,
			proposal_text: Box<Data>,
			duration: T::BlockNumber,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			if duration < T::MinLength::get() {
				ensure!(false, Error::<T>::WrongProposalLength);
			}

			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			// ensure that we don't have too many proposal
			let length_res = <Proposals<T>>::decode_len();
			if let Some(length) = length_res {
				if length == T::MaxProposals::get() as usize {
					ensure!(false, Error::<T>::TooManyProposals);
				}
			}

			// ensure that proposal exists
			let proposal_hash = T::Hashing::hash_of(&proposal_text);
			let (exist, _) = Self::proposal_exist(&proposal_hash);
			ensure!(!exist, Error::<T>::DuplicateProposal);

			// try to append, if error happens, this is probably we have too many proposals
			ensure!(
				<Proposals<T>>::try_append(proposal_hash).is_ok(),
				Error::<T>::TooManyProposals
			);

			// calculate the end block of proposal
			let end = duration + frame_system::Pallet::<T>::block_number();

			// construct the proposal object
			let proposal = Proposal {
				title: *proposal_text,
				proposer: signer.clone(),
				ayes: 0,
				nays: 0,
				poll_end: end,
				reveal_end: None,
				votes: Vec::new(),
				revealed: Vec::new(),
				payout: BalanceOf::<T>::default(),
				closed: false
			};

			<ProposalData<T>>::insert(proposal_hash, proposal);
			Self::deposit_event(Event::<T>::Proposed { account: signer, proposal_hash });

			Ok(())
		}

		/// Closes the vote and starts revealing phase
		#[pallet::weight(10_000_000)]
		pub fn close_vote(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			//ensure that proposal data exists
			let proposal_data = <ProposalData<T>>::get(&proposal);
			ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

			//if we are here, then we know that data exists and we can unwrap it
			let mut proposal_data = proposal_data.unwrap();

			// if reveal end is set, then we know that voting phase ended
			ensure!(proposal_data.reveal_end.is_none(), Error::<T>::VoteAlreadyEnded);

			//make sure that we don't close voting phase too early
			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(proposal_data.poll_end <= current_block, Error::<T>::TooEarly);

			// set the end of reveal phase
			let current_block = frame_system::Pallet::<T>::block_number();
			proposal_data.reveal_end = Some(current_block + T::RevealLength::get());

			<ProposalData<T>>::insert(proposal, proposal_data);

			Self::deposit_event(Event::<T>::ClosedCommit(proposal));
			Ok(())
		}

		/// Closes the reveal and announces the results
		#[pallet::weight(10_000_000)]
		pub fn close_reveal(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			//ensure that proposal data exists
			let proposal_data = <ProposalData<T>>::get(&proposal);
			ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

			//if we are here, then we know that data exists and we can unwrap it
			let mut proposal_data = proposal_data.unwrap();

			//if reveal phase end is not set, that means that we did not start it
			ensure!(proposal_data.reveal_end.is_some(), Error::<T>::RevealNotStarted);
			//if reveal phase end is not set, that means that we did not start it
			ensure!(!proposal_data.closed, Error::<T>::RevealEnded);

			let reveal_end = proposal_data.reveal_end.unwrap();
			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(reveal_end <= current_block, Error::<T>::TooEarly);

			//refund voting tokens to voters
			for (_, (account, votes, _)) in proposal_data.votes.iter().enumerate() {
				let amount = u8::pow(*votes, 2);
				Self::deposit_votes(account, amount);
			}

			//deduce winning side, slash and reward voters
			let result = proposal_data.ayes.cmp(&proposal_data.nays);
			let pot_address = Self::account_id();
			let amount: BalanceOf<T>;
			match result {
				Ordering::Greater => {
					let losers: Vec<T::AccountId> = proposal_data
						.votes
						.iter()
						.filter(|entry| entry.2 == Vote::No)
						.map(|entry| entry.0.clone())
						.collect();
					amount = Self::slash_voting_side(losers, &pot_address)?;
					let winners: Vec<T::AccountId> = proposal_data
						.votes
						.iter()
						.filter(|entry| entry.2 == Vote::Yes)
						.map(|entry| entry.0.clone())
						.collect();
					Self::reward_voting_side(winners, &pot_address, amount)?;
					Self::deposit_event(Event::<T>::Approved(proposal));
				},
				Ordering::Less => {
					let losers: Vec<T::AccountId> = proposal_data
						.votes
						.iter()
						.filter(|entry| entry.2 == Vote::Yes)
						.map(|entry| entry.0.clone())
						.collect();
					amount = Self::slash_voting_side(losers, &pot_address)?;
					let winners: Vec<T::AccountId> = proposal_data
						.votes
						.iter()
						.filter(|entry| entry.2 == Vote::No)
						.map(|entry| entry.0.clone())
						.collect();
					Self::reward_voting_side(winners, &pot_address, amount)?;
					Self::deposit_event(Event::<T>::Disapproved(proposal));
				},
				Ordering::Equal => {
					let losers: Vec<T::AccountId> =
						proposal_data.votes.iter().map(|entry| entry.0.clone()).collect();
					amount = Self::slash_voting_side(losers, &pot_address)?;
					Self::reward_voting_side(
						vec![proposal_data.clone().proposer],
						&pot_address,
						amount,
					)?;
					Self::deposit_event(Event::<T>::Tie(proposal));
				},
			}

			//set the amount that was slashed and paid
			proposal_data.payout = amount;
			//close proposal
			proposal_data.closed = true;
			<ProposalData<T>>::insert(&proposal, proposal_data.clone());

			Self::deposit_event(Event::<T>::ClosedReveal {
				proposal_hash: proposal,
				yes: proposal_data.ayes,
				no: proposal_data.nays,
				revealed: proposal_data.revealed.len() as u32,
				payout: proposal_data.payout,
			});
			Ok(())
		}

		/// Reveal your vote.
		/// Can be done anytime before reveal vote timeout but is not incentivised
		#[pallet::weight(10_000_000)]
		pub fn reveal_vote(origin: OriginFor<T>, proposal: T::Hash, vote: Vote) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			//verify the signature exists
			let commit = <Commits<T>>::take(&signer, &proposal);
			ensure!(commit.is_some(), Error::<T>::NoCommit);
			let commit = commit.unwrap();

			let proposal_data = <ProposalData<T>>::get(&proposal);
			ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);
			let mut proposal_data = proposal_data.unwrap();

			let reveal_exist = proposal_data.reveal_end;

			if let Some(reveal_end) = reveal_exist {
				let current_block = frame_system::Pallet::<T>::block_number();

				// if voter decides to reveal votes after the end, he will just be slashed
				// the voter is incentivised to perform this action in order to refund voting tokens
				// or to cash out
				if current_block > reveal_end {
					let pot_address = Self::account_id();
					let _ = Self::slash_voting_side(vec![signer.clone()], &pot_address)?;
					let amount = u8::pow(commit.number, 2);
					Self::deposit_votes(&signer, amount);

					//probably need to refund, but let it be additional punishment
					return Ok(());
				}
			}

			//get the data that supposed to be signed
			let data = (vote.clone(), commit.salt).encode();
			//and check signature validity
			let valid_sign = commit.signature.verify(data.as_slice(), &signer);
			ensure!(valid_sign, Error::<T>::SignatureInvalid);

			let voted = Self::already_voted(&signer, &proposal_data);
			ensure!(!voted, Error::<T>::DuplicateVote);

			match vote {
				Vote::Yes => proposal_data.ayes += commit.number as u32,
				Vote::No => proposal_data.nays += commit.number as u32,
			}

			//push the vote counters
			proposal_data.votes.push((signer.clone(), commit.number, vote));
			//update the list of voters that revealed their choices
			proposal_data.revealed.push(signer.clone());

			<ProposalData<T>>::insert(proposal, proposal_data);

			Self::deposit_event(Event::<T>::Voted { account: signer, proposal_hash: proposal });

			Ok(())
		}

		/// Secretly submit the vote with the salt
		#[pallet::weight(10_000_000)]
		pub fn commit_vote(
			origin: OriginFor<T>,
			proposal: T::Hash,
			data: T::Signature,
			number: VoteToken,
			salt: u32,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			if number == 0 {
				ensure!(false, Error::<T>::InvalidArgument);
			}

			//make sure that vote has not been committed before
			let committed = Self::already_committed_and_exist(&signer, &proposal);
			ensure!(!committed, Error::<T>::DuplicateVote);

			//ensure that proposal data exists
			let proposal_data = <ProposalData<T>>::get(&proposal);
			ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);
			let proposal_data = proposal_data.unwrap();

			//ensure that we don't commit to finished proposal
			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(current_block < proposal_data.poll_end, Error::<T>::VoteEnded);

			//subtract voting tokens based on quadratic scale
			//i.e. tokens=vote^2
			//make sure that voter has enough voting tokens
			let enough_tokens = Self::decrease_votes(&signer, number.pow(2));
			ensure!(enough_tokens, Error::<T>::NotEnoughVotingTokens);

			//create commit instance
			let commit = Commit { signature: data, salt, number };
			<Commits<T>>::insert(signer.clone(), proposal, commit);

			Self::deposit_event(Event::<T>::Committed { account: signer, proposal_hash: proposal });

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn is_member(who: &T::AccountId) -> bool {
		<Members<T>>::contains_key(who)
	}

	pub fn proposal_exist(proposal: &T::Hash) -> (bool, BoundedVec<T::Hash, T::MaxProposals>) {
		let proposals = <Proposals<T>>::get();
		(proposals.contains(proposal), proposals)
	}

	pub fn already_voted(
		who: &T::AccountId,
		proposal: &types::Proposal<T::AccountId, T::BlockNumber, BalanceOf<T>>,
	) -> bool {
		proposal.revealed.contains(who)
	}

	pub fn already_committed_and_exist(who: &T::AccountId, proposal_hash: &T::Hash) -> bool {
		<Commits<T>>::get(who, proposal_hash).is_some()
	}

	/// Deposit voting tokens to the account and make sure it does not exceed the limit
	pub fn deposit_votes(who: &T::AccountId, tokens: u8) {
		<Members<T>>::mutate(who, |balance| {
			balance.voting_tokens += tokens;
			if balance.voting_tokens > T::MaxVotingTokens::get() {
				balance.voting_tokens = T::MaxVotingTokens::get();
			}
		});
	}

	/// Update the internal record of funds reserved under the account
	pub fn set_reserved_balance(who: &T::AccountId, funds: BalanceOf<T>) {
		<Members<T>>::mutate(who, |balance| {
			balance.reserved_balance = funds;
		});
	}

	/// tries to decrease the voting tokens of a specific account by specified amount.
	/// Returns false if account does not have enough voting tokens
	pub fn decrease_votes(who: &T::AccountId, amount: u8) -> bool {
		<Members<T>>::try_mutate(who, |balance| {
			if balance.voting_tokens < amount {
				return Err(());
			}
			balance.voting_tokens = balance.voting_tokens.saturating_sub(amount);
			Ok(())
		})
		.is_ok()
	}

	/// Slashes the losing side, puts money in a pot and returns the total amount slashed
	pub fn slash_voting_side(
		voters: Vec<T::AccountId>,
		pot: &T::AccountId,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut payout: BalanceOf<T> = BalanceOf::<T>::default();
		for voter in voters {
			let denominator: BalanceOf<T> = 10u8.into();
			let slash = T::Currency::reserved_balance(&voter)
				.checked_div(&denominator.clone())
				.get_or_insert(BalanceOf::<T>::default())
				.to_owned();
			let lost = T::Currency::repatriate_reserved(
				&voter,
				pot,
				slash,
				frame_support::traits::BalanceStatus::Reserved,
			)?;
			//calculate how much funds have actually been slashed
			let slashed = slash.saturating_sub(lost);
			<Members<T>>::mutate(&voter, |balance| {
				balance.reserved_balance = balance.reserved_balance.saturating_sub(slashed);
			});
			// even though we may not necessary
			payout = payout.saturating_add(slashed);
		}
		Ok(payout)
	}

	/// Rewards evenly every member from the pot with the provided sum
	pub fn reward_voting_side(
		voters: Vec<T::AccountId>,
		pot: &T::AccountId,
		total: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let len = voters.len() as u32;
		let share = total / len.into();
		for voter in voters {
			let lost = T::Currency::repatriate_reserved(
				pot,
				&voter,
				share,
				frame_support::traits::BalanceStatus::Reserved,
			)?;
			let actual_share = share.saturating_sub(lost);
			//increase the reserved funds under the account
			<Members<T>>::mutate(&voter, |balance| {
				balance.reserved_balance = balance.reserved_balance.saturating_add(actual_share);
			});
		}
		Ok(())
	}

	/// get voting pot address to deposit slashed tokens to and take rewards from
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}
}
