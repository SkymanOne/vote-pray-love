#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::{DispatchError, DispatchResult};
	use frame_support::traits::{BalanceStatus, Currency, OnUnbalanced, ReservableCurrency};
	use frame_support::{
		pallet_prelude::{CountedStorageMap, ValueQuery, *},
		traits::tokens::Balance,
		Blake2_128Concat, StorageMap,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	pub type MemberCount = u32;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub trait IdentityProvider<AccountId> {
		fn check_existence(account: &AccountId) -> bool;
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type IdentityProvider: IdentityProvider<Self::AccountId>;
		type Currency: ReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		type BasicDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Joined(T::AccountId),
		Left {
			account: T::AccountId,
			cashout: BalanceOf<T>,
		},
		/// A motion (given hash) has been proposed (by given account) with a threshold (given
		/// `MemberCount`).
		Proposed {
			account: T::AccountId,
			proposal_hash: T::Hash,
			threshold: MemberCount,
		},
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Voted {
			account: T::AccountId,
			proposal_hash: T::Hash,
			voted: bool,
			yes: MemberCount,
			no: MemberCount,
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
	}

	//we use unbounded storage because we size of council can vary
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Members<T: Config> =
		CountedStorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]

		pub fn join_committee(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(T::Currency::can_reserve(&signer, T::BasicDeposit::get()), Error::<T>::NotEnoughFunds);

			//TODO: check if signer has identity

			//TODO: check if signer is a member already

			//TODO: deposit

			<Members<T>>::insert(&signer, T::BasicDeposit::get());

			Ok(())
		}
	}
}
