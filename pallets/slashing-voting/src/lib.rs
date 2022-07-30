#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	//use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`

	#[pallet::config] // <-- Step 2. code block will replace this.
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event] // <-- Step 3. code block will replace this.
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error] // <-- Step 4. code block will replace this.
	pub enum Error<T> {
		UndefinedBehaviour,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//#[pallet::storage] // <-- Step 5. code block will replace this.

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call] // <-- Step 6. code block will replace this.
	impl<T: Config> Pallet<T> {}
}
