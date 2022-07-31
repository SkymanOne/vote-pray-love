#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::BoundedVec;
pub use pallet::*;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::{Commit, Data, Proposal, Vote};
	use frame_support::dispatch::{DispatchError, DispatchResult};
	use frame_support::ensure;
	use frame_support::pallet_prelude::CountedStorageMap;
	use frame_support::pallet_prelude::StorageDoubleMap;
	use frame_support::pallet_prelude::StorageMap;
	use frame_support::sp_runtime::traits::Hash;
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_support::{
		pallet_prelude::{OptionQuery, ValueQuery, *},
		Blake2_128Concat, Identity,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_std::boxed::Box;
	use sp_std::vec::Vec;
	use sp_runtime::traits::{IdentifyAccount, Member, Verify};

	pub type MemberCount = u32;
	pub type ProposalIndex = u32;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// type ProposalOf<T> = Box<
	// 	Proposal<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber>,
	// >;

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

		/// Maximum number of proposals allowed to be active in parallel.
		type MaxProposals: Get<ProposalIndex>;

		type Public: IdentifyAccount<AccountId = Self::AccountId>;
		type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Joined(T::AccountId),
		Left {
			account: T::AccountId,
			cashout: BalanceOf<T>,
		},
		/// A motion (given hash) has been proposed (by given account)
		Proposed {
			account: T::AccountId,
			proposal_hash: T::Hash,
		},
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Voted {
			account: T::AccountId,
			proposal_hash: T::Hash,
		},
		/// A motion (given hash) has been committed on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Committed {
			account: T::AccountId,
			proposal_hash: T::Hash,
		},
		/// A motion was approved by the required threshold.
		Approved {
			proposal_hash: T::Hash,
		},
		/// A motion was not approved by the required threshold.
		Disapproved {
			proposal_hash: T::Hash,
		},
		/// A motion was executed; result will be `Ok` if it returned without error.
		Executed {
			proposal_hash: T::Hash,
			result: DispatchResult,
		},
		/// A single member did some action; result will be `Ok` if it returned without error.
		MemberExecuted {
			proposal_hash: T::Hash,
			result: DispatchResult,
		},
		/// A proposal was closed because its threshold was reached or after its duration was up.
		Closed {
			proposal_hash: T::Hash,
			yes: MemberCount,
			no: MemberCount,
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
		/// Mismatched index
		WrongIndex,
		/// Duplicate vote ignored
		DuplicateVote,
		/// Members are already initialized!
		AlreadyInitialized,
		/// The close call was made too early, before the end of the voting.
		TooEarly,
		/// There can only be a maximum of `MaxProposals` active proposals.
		TooManyProposals,
		/// The given length bound for the proposal was too low.
		WrongProposalLength,
		/// Not enough funds to join the voting council
		NotEnoughFunds,
		/// Proposal Ended
		ProposalEnded,
		/// No commit has been submitted
		NoCommit,
		/// Could not verify signature of a commit
		SignatureInvalid,
	}

	//we use unbounded storage because we size of council can vary
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Members<T: Config> =
		CountedStorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type Proposals<T: Config> =
		StorageValue<_, BoundedVec<T::Hash, T::MaxProposals>, ValueQuery>;

	#[pallet::storage]
	pub type ProposalData<T: Config> =
		StorageMap<_, Identity, T::Hash, Proposal<T::AccountId, T::BlockNumber>>;

	#[pallet::storage]
	pub type Commits<T: Config> =
		StorageDoubleMap<_, Identity, T::Hash, Identity, T::AccountId, Commit<T::Signature>>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Join committee and deposit money to have skin in a game
		#[pallet::weight(1_000)]
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

			<Members<T>>::insert(&signer, T::BasicDeposit::get());

			Self::deposit_event(Event::<T>::Joined(signer));

			Ok(())
		}

		/// Creates the proposal with given text and duration in blocks
		#[pallet::weight(1_000)]
		pub fn create_proposal(
			origin: OriginFor<T>,
			proposal_text: Box<Data>,
			duration: T::BlockNumber,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			let length_res = <Proposals<T>>::decode_len();
			if let Some(length) = length_res {
				if length == T::MaxProposals::get() as usize {
					ensure!(false, Error::<T>::TooManyProposals);
				}
			}

			let proposal_hash = T::Hashing::hash_of(&proposal_text);
			let (exist, _) = Self::proposal_exist(&proposal_hash);
			ensure!(!exist, Error::<T>::DuplicateProposal);

			ensure!(
				<Proposals<T>>::try_append(proposal_hash).is_ok(),
				Error::<T>::WrongProposalLength
			);

			let end = duration + frame_system::Pallet::<T>::block_number();

			let proposal = Proposal {
				title: *proposal_text,
				proposer: signer.clone(),
				ayes: Vec::new(),
				nays: Vec::new(),
				end,
			};

			<ProposalData<T>>::insert(proposal_hash, proposal);

			Self::deposit_event(Event::<T>::Proposed { account: signer, proposal_hash });

			Ok(())
		}

		//TODO: reveal vote
		#[pallet::weight(1_000)]
		pub fn reveal_vote(origin: OriginFor<T>, proposal: T::Hash, vote: Vote) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			let result = Self::already_voted_and_exist(&signer, &proposal);
			ensure!(result.is_some(), Error::<T>::ProposalMissing);

			let voted = result.unwrap();
			ensure!(!voted, Error::<T>::DuplicateVote);

			//verify the signature
			let commit = <Commits<T>>::get(&proposal, &signer);
			ensure!(commit.is_some(), Error::<T>::NoCommit);
			let commit = commit.unwrap();

			let data = (vote.clone(), commit.salt).encode();

			let valid_sign = commit.signature.verify(data.as_slice(), &signer);
			ensure!(valid_sign, Error::<T>::SignatureInvalid);

			let proposal_data = <ProposalData<T>>::get(&proposal);
			ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

			let mut proposal_data = proposal_data.unwrap();
			match vote {
				Vote::Yes => proposal_data.ayes.push(signer.clone()),
				Vote::No => proposal_data.nays.push(signer.clone()),
			}

			<ProposalData<T>>::insert(proposal, proposal_data);

			Self::deposit_event(Event::<T>::Voted { account: signer, proposal_hash: proposal });

			Ok(())
		}

		/// Secretly submit the vote with the salt
		#[pallet::weight(1_000)]
		pub fn commit_vote(
			origin: OriginFor<T>,
			proposal: T::Hash,
			data: T::Signature,
			salt: u32,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			//check if signer is a member already | tested
			ensure!(Self::is_member(&signer), Error::<T>::NotMember);

			let committed = Self::already_committed_and_exist(&signer, &proposal);
			ensure!(!committed, Error::<T>::DuplicateVote);

			let proposal_data = <ProposalData<T>>::get(&proposal);
			ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

			let proposal_data = proposal_data.unwrap();

			let current_block = frame_system::Pallet::<T>::block_number();

			ensure!(current_block < proposal_data.end, Error::<T>::ProposalEnded);

			let commit = Commit { signature: data, salt };

			<Commits<T>>::insert(proposal, signer.clone(), commit);

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

	pub fn already_voted_and_exist(who: &T::AccountId, proposal_hash: &T::Hash) -> Option<bool> {
		let result = <ProposalData<T>>::get(proposal_hash);
		if let Some(proposal) = result {
			Some(proposal.ayes.contains(who) || proposal.nays.contains(who))
		} else {
			None
		}
	}

	pub fn already_committed_and_exist(
		who: &T::AccountId,
		proposal_hash: &T::Hash,
	) -> bool {
		<Commits<T>>::get(proposal_hash, who).is_some()
	}
}
