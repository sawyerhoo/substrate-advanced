use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn create_claim_tests() {
	new_test_ext().execute_with(|| {
		// Uses a fresh storage environment for each test
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let sender = 1;

		assert_ok!(
			// Asserts that the function call returns Ok
			PoeModule::create_claim(RuntimeOrigin::signed(sender.clone()), claim.clone(),)
		);

		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((sender.clone(), frame_system::Pallet::<Test>::block_number())) // Asserts that the proof was added correctly to the storage
		);

		assert_noop!(
			PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	});
}

#[test]
fn revoke_claim_tests() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let owner = 1;
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(owner), claim.clone()); // Creates a claim before revoking it

		assert_noop!(
			//Asserts that the function call returns the expected error message
			PoeModule::revoke_claim(RuntimeOrigin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);

		assert_ok!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(owner), claim.clone(),) // Revokes the claim
		);

		assert_eq!(
			Proofs::<Test>::contains_key(&claim),
			false // Asserts that the claim is no longer in storage
		);

		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	});
}

#[test]
fn transfer_claim_tests() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let sender = 1;
		let receiver = 2;
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(sender.clone()), claim.clone()); // Creates a claim before transferring it

		assert_ok!(PoeModule::transfer_claim(
			RuntimeOrigin::signed(sender.clone()),
			claim.clone(),
			receiver.clone(),
		));

		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((receiver.clone(), frame_system::Pallet::<Test>::block_number())) // Asserts that the claim was transferred correctly
		);

		assert_noop!(
			// Asserts that the function call returns the expected error message
			PoeModule::transfer_claim(
				RuntimeOrigin::signed(sender.clone()),
				claim.clone(),
				receiver.clone(),
			),
			Error::<Test>::NotClaimOwner
		);

		let _ = PoeModule::revoke_claim(RuntimeOrigin::signed(receiver), claim.clone()); // revoking claim

		assert_noop!(
			// Asserts that the function call returns the expected error message
			PoeModule::transfer_claim(
				RuntimeOrigin::signed(receiver.clone()),
				claim.clone(),
				sender.clone(),
			),
			Error::<Test>::ClaimNotExist
		);
	});
}