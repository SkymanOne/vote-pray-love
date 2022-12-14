use crate::mock::Identity;
use crate::types::*;
use crate::*;
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

		let _ = QuadraticVoting::join_committee(origin.clone());

		let result = QuadraticVoting::create_proposal(
			origin,
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);
		assert_ok!(result);

		let results = <Proposals<Test>>::get();
		assert!(results.len() == 1);
	});
}

#[test]
fn no_proposal_duplicates() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);
		let result = QuadraticVoting::create_proposal(
			origin,
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);
		assert_noop!(result, Error::<Test>::DuplicateProposal);
	});
}

#[test]
fn submit_commits() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let result = QuadraticVoting::commit_vote(origin, proposal_hash, sig, 8, salt);
		assert_ok!(result);
	});
}

#[test]
fn cannot_submit_votes_more_than_have() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let result = QuadraticVoting::commit_vote(origin, proposal_hash, sig, 11, salt);
		assert_noop!(result, Error::<Test>::NotEnoughVotingTokens);
	});
}

#[test]
fn cannot_commit_after_deadline() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		System::set_block_number(System::block_number().saturating_add(105));

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let result = QuadraticVoting::commit_vote(origin, proposal_hash, sig, 5, salt);
		assert_noop!(result, Error::<Test>::VoteEnded);
	});
}

#[test]
fn reveal_vote_success() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		System::set_block_number(System::block_number().saturating_add(20));

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin.clone(), proposal_hash, sig, 8, salt);

		let result = QuadraticVoting::reveal_vote(origin, proposal_hash, Vote::Yes);
		assert_ok!(result);
	});
}

#[test]
fn cannot_reveal_incorrect_vote() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		System::set_block_number(System::block_number().saturating_add(20));

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin.clone(), proposal_hash, sig, 8, salt);

		let result = QuadraticVoting::reveal_vote(origin, proposal_hash, Vote::No);
		assert_noop!(result, Error::<Test>::SignatureInvalid);
	});
}

#[test]
fn close_vote_success() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		System::set_block_number(System::block_number().saturating_add(120));
		let proposal_hash = <Proposals<Test>>::get()[0];
		let result = QuadraticVoting::close_vote(origin, proposal_hash);
		assert_ok!(result);
	});
}

#[test]
fn cannot_close_vote_before_deadline() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let proposal_hash = <Proposals<Test>>::get()[0];
		let result = QuadraticVoting::close_vote(origin, proposal_hash);
		assert_noop!(result, Error::<Test>::TooEarly);
	});
}

#[test]
fn close_reveal_success() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		System::set_block_number(110);

		let proposal_hash = <Proposals<Test>>::get()[0];
		let _ = QuadraticVoting::close_vote(origin.clone(), proposal_hash);

		System::set_block_number(160);

		let result = QuadraticVoting::close_reveal(origin, proposal_hash);
		assert_ok!(result);
	});
}

#[test]
fn cannot_close_reveal_early() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		System::set_block_number(110);

		let proposal_hash = <Proposals<Test>>::get()[0];
		let _ = QuadraticVoting::close_vote(origin.clone(), proposal_hash);

		System::set_block_number(140);

		let result = QuadraticVoting::close_reveal(origin, proposal_hash);
		assert_noop!(result, Error::<Test>::TooEarly);
	});
}

#[test]
fn cannot_close_reveal_before_vote_end() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin = Origin::signed(alice);
		let _ = Identity::set_identity(origin.clone(), Box::new(data()));

		let _ = QuadraticVoting::join_committee(origin.clone());

		let _ = QuadraticVoting::create_proposal(
			origin.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let proposal_hash = <Proposals<Test>>::get()[0];

		System::set_block_number(140);

		let result = QuadraticVoting::close_reveal(origin, proposal_hash);
		assert_noop!(result, Error::<Test>::RevealNotStarted);
	});
}

#[test]
fn slashed_correctly() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin_alice = Origin::signed(alice.clone());
		let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
		let _ = QuadraticVoting::join_committee(origin_alice.clone());

		let bob = get_bob();
		let origin_bob = Origin::signed(bob.clone());
		let _ = Identity::set_identity(origin_bob.clone(), Box::new(data()));
		let _ = QuadraticVoting::join_committee(origin_bob.clone());

		let _ = QuadraticVoting::create_proposal(
			origin_alice.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin_alice, proposal_hash, sig, 8, salt);

		let (sig, salt) = generate("//Bob", Vote::No);
		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin_bob.clone(), proposal_hash, sig, 2, salt);

		System::set_block_number(101);

		let proposal_hash = <Proposals<Test>>::get()[0];
		let _ = QuadraticVoting::close_vote(origin_bob.clone(), proposal_hash);

		System::set_block_number(160);

		let alice_original_balance = <Members<Test>>::get(alice.clone()).reserved_balance;
		let bob_original_balance = <Members<Test>>::get(bob.clone()).reserved_balance;

		let _ = QuadraticVoting::close_reveal(origin_bob, proposal_hash);

		let alice_current_balance = <Members<Test>>::get(alice).reserved_balance;
		let bob_current_balance = <Members<Test>>::get(bob).reserved_balance;

		let slash = bob_original_balance - bob_current_balance;
		assert!(alice_current_balance == alice_original_balance + slash);
	});
}

#[test]
fn votes_deducted_and_refunded() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin_alice = Origin::signed(alice.clone());
		let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
		let _ = QuadraticVoting::join_committee(origin_alice.clone());

		let bob = get_bob();
		let origin_bob = Origin::signed(bob.clone());
		let _ = Identity::set_identity(origin_bob.clone(), Box::new(data()));
		let _ = QuadraticVoting::join_committee(origin_bob.clone());

		let _ = QuadraticVoting::create_proposal(
			origin_alice.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin_alice.clone(), proposal_hash, sig, 8, salt);

		let alice_original_votes = <Members<Test>>::get(alice.clone()).voting_tokens;
		assert!(alice_original_votes == MaxTokens::get() - 8_u8.pow(2));

		let (sig, salt) = generate("//Bob", Vote::No);
		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin_bob.clone(), proposal_hash, sig, 2, salt);

		let bob_original_votes = <Members<Test>>::get(bob.clone()).voting_tokens;
		assert!(bob_original_votes == MaxTokens::get() - 2_u8.pow(2));

		System::set_block_number(101);

		let proposal_hash = <Proposals<Test>>::get()[0];
		let _ = QuadraticVoting::close_vote(origin_bob.clone(), proposal_hash);

		let _ = QuadraticVoting::reveal_vote(origin_alice, proposal_hash, Vote::Yes);
		let _ = QuadraticVoting::reveal_vote(origin_bob.clone(), proposal_hash, Vote::No);

		System::set_block_number(160);

		let _ = QuadraticVoting::close_reveal(origin_bob, proposal_hash);

		let alice_tokens = <Members<Test>>::get(alice).voting_tokens;
		let bob_tokens = <Members<Test>>::get(bob).voting_tokens;

		assert!(alice_tokens == MaxTokens::get());
		assert!(bob_tokens == MaxTokens::get());
	});
}

#[test]
fn cannot_leave_while_in_vote() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin_alice = Origin::signed(alice);
		let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
		let _ = QuadraticVoting::join_committee(origin_alice.clone());

		let _ = QuadraticVoting::create_proposal(
			origin_alice.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin_alice.clone(), proposal_hash, sig, 8, salt);

		let result = QuadraticVoting::leave_committee(origin_alice.clone());
		assert_noop!(result, Error::<Test>::InMotion);

		System::set_block_number(110);

		let _ = QuadraticVoting::close_vote(origin_alice.clone(), proposal_hash);

		let result = QuadraticVoting::leave_committee(origin_alice);
		assert_noop!(result, Error::<Test>::InMotion);
	});
}

#[test]
fn cashout() {
	new_test_ext().execute_with(|| {
		let alice = get_alice();
		let origin_alice = Origin::signed(alice.clone());
		let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
		let _ = QuadraticVoting::join_committee(origin_alice.clone());

		let bob = get_bob();
		let origin_bob = Origin::signed(bob);
		let _ = Identity::set_identity(origin_bob.clone(), Box::new(data()));
		let _ = QuadraticVoting::join_committee(origin_bob.clone());

		let _ = QuadraticVoting::create_proposal(
			origin_alice.clone(),
			Box::new(Data::Raw(BoundedVec::default())),
			100,
		);

		let results = <Proposals<Test>>::get();
		let proposal_hash = results[0];

		let (sig, salt) = generate("//Alice", Vote::Yes);
		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin_alice.clone(), proposal_hash, sig, 8, salt);


		let (sig, salt) = generate("//Bob", Vote::No);
		let sig = sp_runtime::MultiSignature::Sr25519(sig);
		let _ = QuadraticVoting::commit_vote(origin_bob.clone(), proposal_hash, sig, 2, salt);

		System::set_block_number(101);

		let proposal_hash = <Proposals<Test>>::get()[0];
		let _ = QuadraticVoting::close_vote(origin_bob.clone(), proposal_hash);

		let _ = QuadraticVoting::reveal_vote(origin_alice.clone(), proposal_hash, Vote::Yes);
		let _ = QuadraticVoting::reveal_vote(origin_bob.clone(), proposal_hash, Vote::No);

		System::set_block_number(160);

		let _ = QuadraticVoting::close_reveal(origin_bob, proposal_hash);

		let result = QuadraticVoting::leave_committee(origin_alice);
		assert_ok!(result);

		assert!(Balances::reserved_balance(alice) == 0);
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
