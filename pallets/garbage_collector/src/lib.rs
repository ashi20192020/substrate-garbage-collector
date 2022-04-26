#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::{
	dispatch::Dispatchable,
	pallet_prelude::*,
	traits::{schedule::Named as ScheduleNamed, LockIdentifier},
};
use frame_system::pallet_prelude::*;
use sp_std::boxed::Box;
use sp_std::{vec, vec::Vec};

const GC_ID: LockIdentifier = *b"garbagec";

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod traits;

pub type CallOf<T> = <T as Config>::Call;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Sized {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Call: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;

		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		type Scheduler: ScheduleNamed<Self::BlockNumber, CallOf<Self>, Self::PalletsOrigin>;

		type GarbageCollectorOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn pallet_call)]
	pub type PalletCall<T> = StorageValue<_, (u8, u8), ValueQuery>;

	/// The next free index
	#[pallet::storage]
	#[pallet::getter(fn garbage_collector_count)]
	pub type GarbageCollectorCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CleanupCallScheduled { index: u64, priority: u64 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		SchedulerError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

		fn on_initialize(now: T::BlockNumber) -> Weight {
			Self::schedule_cleanup(now)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn submit_pallet(
			origin: OriginFor<T>,
			call: Box<CallOf<T>>,
		) -> DispatchResult {
			T::GarbageCollectorOrigin::ensure_origin(origin)?;
			let call = *call;

			let (pallet_idx, call_idx): (u8, u8) = call
				.using_encoded(|mut bytes| Decode::decode(&mut bytes))
				.expect(
					"decode input is output of Call encode; Call guaranteed to have two enums; qed",
				);

			<PalletCall<T>>::put((pallet_idx, call_idx));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn increase_index(
			origin: OriginFor<T>,
		) -> DispatchResult {
			T::GarbageCollectorOrigin::ensure_origin(origin)?;

			let index = Self::garbage_collector_count();
			GarbageCollectorCount::<T>::put(index + 1);

			Ok(())
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: sp_std::marker::PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			GarbageCollectorCount::<T>::put(0 as u32);
		}
	}
}

impl<T: Config> Pallet<T> {

	fn schedule_cleanup(
		when: T::BlockNumber,
	) -> Weight {
		let max_block_weight = T::BlockWeights::get().max_block;

		let bytes: Vec<u8> = Self::pallet_call().encode();

		if bytes != vec![0,0] {

			let call: <T as Config>::Call = Decode::decode(&mut &bytes[..]).unwrap();

			let index = Self::garbage_collector_count();
			T::Scheduler::schedule_named(
				(GC_ID, index).encode(),
				frame_support::traits::schedule::DispatchTime::At(when),
				None,
				63,
				frame_system::RawOrigin::Root.into(),
				call,
			)
				.map_err(|_| Error::<T>::SchedulerError).ok();

			Self::deposit_event(Event::<T>::CleanupCallScheduled {
				index: index.into(),
				priority: 63,
			});

		}

		max_block_weight
	}
}
