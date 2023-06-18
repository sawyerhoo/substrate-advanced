#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use offchain::*;
mod offchain;

#[frame_support::pallet]
mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::{
		offchain::{
			AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
			SigningTypes,
		},
		pallet_prelude::*,
	};

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct PayloadStruct<P> {
		public: P,
		count: u64,
	}

	impl<T: SigningTypes> SignedPayload<T> for PayloadStruct<T::Public> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AppCrypto: AppCrypto<Self::Public, Self::Signature>;
	}


	#[pallet::event]
	// #[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}


	#[pallet::pallet]
	pub struct Pallet<T>(_);


	


	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_block_number: T::BlockNumber) {
			//通过Http接口获取外部价格
			let price = crate::offchain::get_count().unwrap_or(0);
			let value:u64 = price.into();

			let signer = Signer::<T, T::AppCrypto>::any_account();

			if let Some((_, res)) = signer.send_unsigned_transaction(
				// this line is to prepare and return payload
				|acct| PayloadStruct { count: value, public: acct.public.clone() },
				|payload, signature| Call::unsigned_extrinsic_with_signed_payload { payload, signature },
			) {
				match res {
					Ok(()) => {log::info!("### OCW ==> unsigned tx with signed payload successfully sent.");}
					Err(()) => {log::error!("### OCW ==> sending unsigned tx with signed payload failed.");}
				};
			} else {
				// The case of `None`: no account is available for sending
				log::error!("### OCW ==> No local account available");
			}


		}
	}


	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("my-pallet")
					.priority(100)
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::unsigned_extrinsic_with_signed_payload { ref payload, ref signature } => {
					if !SignedPayload::<T>::verify::<T::AppCrypto>(payload, signature.clone()) {
						return InvalidTransaction::BadProof.into()
					}
					valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn offchain_index_set(origin: OriginFor<T>, number: u64) -> DispatchResult {
			let _signer = ensure_signed(origin)?;

			crate::offchain::offchain_index_set(
				frame_system::Pallet::<T>::block_number(),
				number,
			);

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn unsigned_extrinsic_with_signed_payload(
			origin: OriginFor<T>,
			payload: PayloadStruct<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			log::info!(
				"[ {:?} ] in call unsigned_extrinsic_with_signed_payload: {:?}",
				frame_system::Pallet::<T>::block_number(),
				payload.count,
			);

			Ok(())
		}
	}
}