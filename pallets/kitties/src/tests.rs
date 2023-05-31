use crate::{Error, mock::*, Event};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_create(){
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;

        assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
        let kitty = KittiesModule::kitties(kitty_id);
        // Asserts that a `KittyCreated` event has been emitted
        System::assert_has_event(Event::KittyCreated {
            who: account_id,
            kitty_id,
            kitty: kitty.unwrap(),
        }.into());
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
        assert_eq!(KittiesModule::kitties(kitty_id).is_some(),true);
        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(account_id));
        assert_eq!(KittiesModule::kitty_parents(kitty_id), None);

        crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
        assert_noop!(
            KittiesModule::create(RuntimeOrigin::signed(account_id)),
            Error::<Test>::InvalidKittyId
        );
    });
}

#[test]
fn it_works_for_breed() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;

        assert_noop!(
            KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
            Error::<Test>::SameKittyId
        );

        assert_noop!(
            KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1),
            Error::<Test>::InvalidKittyId
        );

        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

        assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);
        assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1));
        // Asserts that a `KittyBred` event has been emitted
        System::assert_has_event(Event::KittyBred {
            who: account_id,
            kitty_id: kitty_id + 2,
            kitty: KittiesModule::kitties(kitty_id + 2).unwrap(),
        }.into());

        let breed_kitty_id = 2;
        assert_eq!(KittiesModule::next_kitty_id(),breed_kitty_id + 1);
        assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true);
        assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));
        assert_eq!(KittiesModule::kitty_parents(breed_kitty_id), Some((kitty_id, kitty_id + 1)));
    });
}

#[test]
fn it_works_for_transfer() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let recipient = 2;

        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

        assert_noop!(
            KittiesModule::transfer(RuntimeOrigin::signed(recipient), account_id, kitty_id ),
            Error::<Test>::NotOwner
        );
        
        assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id));
        // Asserts that a `KittyTransferred` event has been emitted
        System::assert_has_event(Event::KittyTransferred {
            who: account_id,
            recipient,
            kitty_id,
        }.into());

        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(recipient));

        assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(recipient), account_id, kitty_id));

        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
    });
}