use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TemplateModule::add_liquidity(RuntimeOrigin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		// assert_eq!(TemplateModule::something(), Some(42));
// 	});
// }

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			TemplateModule::cause_error(RuntimeOrigin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }


// use crate::{mock::*, Error};
// use frame_support::{assert_noop, assert_ok};

// #[test]
// fn it_works_for_create_claim() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         // Dispatch a signed extrinsic.
//         assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
//         // Read pallet storage and assert an expected result.
//         assert!(PoeModule::proofs(claim.clone()).is_some());
//     });
// }

// #[test]
// fn it_fails_for_create_claim_with_claim_length_error() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3];
//         // Dispatch a signed extrinsic.
//         assert_noop!(PoeModule::create_claim(Origin::signed(1), claim), Error::<Test>::ClaimLengthError);
//     });
// }

// #[test]
// fn it_fails_for_create_claim_with_claim_alreay_exist() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         // Dispatch a signed extrinsic.
//         assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
//         assert_noop!(PoeModule::create_claim(Origin::signed(1), claim), Error::<Test>::ClaimAlreadyExist);
//     });
// }

// #[test]
// fn it_fails_for_revoke_claim_with_claim_not_exist() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         assert_noop!(PoeModule::revoke_claim(Origin::signed(1), claim), Error::<Test>::ClaimNotExist);
//     });
// }

// #[test]
// fn it_fails_for_revoke_claim_with_not_owner() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         // Dispatch a signed extrinsic.
//         assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
//         assert_noop!(PoeModule::revoke_claim(Origin::signed(2), claim), Error::<Test>::OnlyOwnerCanRevoke);
//     });
// }

// #[test]
// fn it_works_for_revoke_claim() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         // Dispatch a signed extrinsic.
//         assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
//         assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim));
//     });
// }

// #[test]
// fn it_fails_for_transfer_claim_with_transfer_to_self() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         assert_noop!(PoeModule::transfer_claim(Origin::signed(1), 1, claim), Error::<Test>::NotAbleToTransferToSelf);
//     });
// }

// #[test]
// fn it_fails_for_transfer_claim_with_claim_not_exist() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         assert_noop!(PoeModule::transfer_claim(Origin::signed(1), 2, claim), Error::<Test>::ClaimNotExist);
//     });
// }

// #[test]
// fn it_fails_for_transfer_claim_with_not_owner() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         // Dispatch a signed extrinsic.
//         assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
//         assert_noop!(PoeModule::transfer_claim(Origin::signed(2), 1, claim), Error::<Test>::OnlyOwnerCanTransfer);
//     });
// }

// #[test]
// fn it_works_for_transfer_claim() {
//     new_test_ext().execute_with(|| {
//         let claim = vec![1, 2, 3, 4, 5, 6, 7, 8];
//         // Dispatch a signed extrinsic.
//         assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
//         assert_ok!(PoeModule::transfer_claim(Origin::signed(1), 2, claim));
//     });
// }