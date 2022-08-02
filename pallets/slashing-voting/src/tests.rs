use crate::mock::Identity;
use crate::types::*;
use crate::{mock::*, Error};
use frame_support::pallet_prelude::*;
use frame_support::{assert_noop, assert_ok};
use pallet_identity::IdentityInfo;

#[test]
fn not_join_without_identity() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		assert_noop!(QuadraticVoting::join_committee(origin), Error::<Test>::NoIdentity);
	});
}

#[test]
fn disallow_action_for_non_members() {
	new_test_ext().execute_with(|| {
		let bob_origin = Origin::signed(get_bob());
		let _ = Identity::set_identity(bob_origin.clone(), Box::new(data()));

		let result = QuadraticVoting::create_proposal(
			bob_origin,
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);
		assert_noop!(result, Error::<Test>::NotMember);
	});
}

#[test]
fn join_with_identity() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let result = Identity::set_identity(origin.clone(), Box::new(data()));
		assert!(result.is_ok());
		assert_ok!(QuadraticVoting::join_committee(origin));
	});
}

#[test]
fn create_proposal_success() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		assert_ok!(QuadraticVoting::join_committee(origin.clone()));

		let result = QuadraticVoting::create_proposal(
			origin,
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);
		assert_ok!(result);
	});
}

#[test]
fn no_proposal_duplicates() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		assert_ok!(QuadraticVoting::join_committee(origin.clone()));

		let result = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);
		assert_ok!(result);
		let result = QuadraticVoting::create_proposal(
			origin,
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);
		assert_noop!(result, Error::<Test>::DuplicateProposal);
	});
}

fn data() -> IdentityInfo<MaxAdditionalFields> {
	IdentityInfo {
		display: pallet_identity::Data::Raw(b"ten".to_vec().try_into().unwrap()),
		additional: BoundedVec::default(),
		legal: Default::default(),
		web: Default::default(),
		riot: Default::default(),
		twitter: Default::default(),
		email: Default::default(),
		pgp_fingerprint: Default::default(),
		image: Default::default(),
	}
}
