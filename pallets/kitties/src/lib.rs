#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	pub use crate::migrations::current_version::*;

	use crate::migrations::upgrade_storage;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Randomness},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::AccountIdConversion;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBred { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred{ who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
		KittyOnSale {who: T::AccountId, kitty_id: KittyId},
		KittyBought {who: T::AccountId, kitty_id: KittyId},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		NoOwner,
		AlreadyOnSale,
		AlreadyOwned,
		NotOnSale,
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			upgrade_storage::<T>()
		}
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		// pub fn create(origin: OriginFor<T>) -> DispatchResult {
		pub fn create(origin: OriginFor<T>, name: KittyName) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_kitty_id()?;
			// let kitty = Kitty(Self::random_value(&who));
			let dna = Self::random_value(&who);
			let kitty = Kitty{ dna, name};

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?;
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyId, kitty_id_2: KittyId, name: KittyName) -> DispatchResult{
		// pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyId, kitty_id_2: KittyId) -> DispatchResult{
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			ensure!(Kitties::<T>::contains_key(kitty_id_1),Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2),Error::<T>::InvalidKittyId);

			// let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			// let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let kitty_id = Self::get_next_kitty_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// let selector = Self::random_value(&who);
			
			// let mut data = [0u8; 16];
			let dna = [0u8; 16];

			// for i in 0..kitty_1.0.len(){
			// 	// 0 choose kitty2, and 1 choose kitty1
			// 	data[i] = (kitty_1.0[i] &selector[i]) | (kitty_2.0[i] & !selector[i]);
			// }

			// let kitty = Kitty(data);
			let kitty = Kitty{ dna, name};

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?;
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1,kitty_id_2));
			NextKittyId::<T>::set(kitty_id + 1);

			Self::deposit_event(Event::KittyBred { who, kitty_id, kitty});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn transfer(origin: OriginFor<T>, recipient: T::AccountId, kitty_id: KittyId) -> DispatchResult { 
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id),Error::<T>::InvalidKittyId);

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::NotOwner)?;
			ensure!(owner == who, Error::<T>::NotOwner);


			KittyOwner::<T>::insert(kitty_id, &recipient);
			Self::deposit_event(Event::KittyTransferred { who, recipient, kitty_id});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn sale(
			origin: OriginFor<T>,
			kitty_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(Self::kitty_on_sale(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			<KittyOnSale<T>>::insert(kitty_id, ());
			Self::deposit_event(Event::KittyOnSale { who, kitty_id});

			Ok(())
		}


		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn buy(
			origin: OriginFor<T>,
			kitty_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			let owner = Self::kitty_owner(kitty_id).ok_or::<DispatchError>(Error::<T>::NoOwner.into())?;
			ensure!(owner != who, Error::<T>::AlreadyOwned);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale);

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?;
			// T::Currency::unreserve(&owner, price)?;
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;
			
			<KittyOwner<T>>::insert(kitty_id, &who);
			<KittyOnSale<T>>::remove(kitty_id);

			Self::deposit_event(Event::KittyBought { who, kitty_id});

			Ok(())
		}

	}	

	impl<T: Config> Pallet<T> {
		fn get_next_kitty_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8;16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
