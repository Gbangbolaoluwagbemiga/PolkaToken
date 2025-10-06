#[cfg(test)]
mod tests {
    use ink::env::{test, DefaultEnvironment};
    use ink::primitives::Address;
    use ink::U256;
    use ink::scale::Decode;
    use crate::erc20::{Erc20, Error};

    fn setup() -> (Address, Address) {
        let alice: Address = Default::default();
        let bob: Address = [1u8; 20].into();
        (alice, bob)
    }

    // Helper function to decode Transfer event data
    fn decode_transfer_event(event_data: &[u8]) -> (Option<Address>, Option<Address>, U256) {
        let mut data = event_data;
        let from: Option<Address> = Decode::decode(&mut data).unwrap();
        let to: Option<Address> = Decode::decode(&mut data).unwrap();
        let value: U256 = Decode::decode(&mut data).unwrap();
        (from, to, value)
    }

    // Helper function to decode Approval event data
    fn decode_approval_event(event_data: &[u8]) -> (Address, Address, U256) {
        let mut data = event_data;
        let owner: Address = Decode::decode(&mut data).unwrap();
        let spender: Address = Decode::decode(&mut data).unwrap();
        let value: U256 = Decode::decode(&mut data).unwrap();
        (owner, spender, value)
    }

    #[ink::test]
    fn new_works() {
        let (alice, _) = setup();
        let initial_supply = U256::from(1000u32);
        let contract = Erc20::new(initial_supply);
        assert_eq!(contract.total_supply(), initial_supply);
        assert_eq!(contract.balance_of(alice), initial_supply);

        let events = test::recorded_events();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        let (from, to, value) = decode_transfer_event(&event.data);
        assert_eq!(from, None);
        assert_eq!(to, Some(alice));
        assert_eq!(value, initial_supply);
    }

    #[ink::test]
    fn transfer_works() {
        let (alice, bob) = setup();
        let initial_supply = U256::from(1000u32);
        let mut contract = Erc20::new(initial_supply);
        let transfer_amount = U256::from(100u32);
        assert_eq!(contract.balance_of(bob), U256::zero());

        let initial_events_len = test::recorded_events().len();
        contract.transfer(bob, transfer_amount).unwrap();

        let events = test::recorded_events();
        assert_eq!(events.len(), initial_events_len + 1);
        let event = &events[events.len() - 1];
        let (from, to, value) = decode_transfer_event(&event.data);
        assert_eq!(from, Some(alice));
        assert_eq!(to, Some(bob));
        assert_eq!(value, transfer_amount);

        assert_eq!(contract.balance_of(alice), initial_supply - transfer_amount);
        assert_eq!(contract.balance_of(bob), transfer_amount);
    }

    #[ink::test]
    fn transfer_fails_with_insufficient_balance() {
        let (alice, bob) = setup();
        let initial_supply = U256::from(1000u32);
        let mut contract = Erc20::new(initial_supply);
        let transfer_amount = U256::from(1001u32);

        let initial_events_len = test::recorded_events().len();
        let result = contract.transfer(bob, transfer_amount);
        assert_eq!(result, Err(Error::InsufficientBalance));

        let events = test::recorded_events();
        assert_eq!(events.len(), initial_events_len); // No event emitted on failure

        assert_eq!(contract.balance_of(alice), initial_supply);
        assert_eq!(contract.balance_of(bob), U256::zero());
    }

    #[ink::test]
    fn approve_works() {
        let (alice, bob) = setup();
        let initial_supply = U256::from(1000u32);
        let mut contract = Erc20::new(initial_supply);
        let approve_amount = U256::from(200u32);
        assert_eq!(contract.allowance(alice, bob), U256::zero());

        let initial_events_len = test::recorded_events().len();
        contract.approve(bob, approve_amount).unwrap();

        let events = test::recorded_events();
        assert_eq!(events.len(), initial_events_len + 1);
        let event = &events[events.len() - 1];
        let (owner, spender, value) = decode_approval_event(&event.data);
        assert_eq!(owner, alice);
        assert_eq!(spender, bob);
        assert_eq!(value, approve_amount);

        assert_eq!(contract.allowance(alice, bob), approve_amount);
    }

    #[ink::test]
    fn transfer_from_works() {
        let (alice, bob) = setup();
        let initial_supply = U256::from(1000u32);
        let mut contract = Erc20::new(initial_supply);
        let transfer_amount = U256::from(100u32);

        contract.approve(bob, U256::from(200u32)).unwrap();
        assert_eq!(contract.allowance(alice, bob), U256::from(200u32));

        test::set_caller(bob);
        let initial_events_len = test::recorded_events().len();
        let result = contract.transfer_from(alice, bob, transfer_amount);
        assert_eq!(result, Ok(()));

        let events = test::recorded_events();
        assert_eq!(events.len(), initial_events_len + 1);
        let event = &events[events.len() - 1];
        let (from, to, value) = decode_transfer_event(&event.data);
        assert_eq!(from, Some(alice));
        assert_eq!(to, Some(bob));
        assert_eq!(value, transfer_amount);

        assert_eq!(contract.balance_of(alice), initial_supply - transfer_amount);
        assert_eq!(contract.balance_of(bob), transfer_amount);
        assert_eq!(contract.allowance(alice, bob), U256::from(100u32));
    }

    #[ink::test]
    fn transfer_from_fails_with_insufficient_allowance() {
        let (alice, bob) = setup();
        let initial_supply = U256::from(1000u32);
        let mut contract = Erc20::new(initial_supply);
        let transfer_amount = U256::from(100u32);

        contract.approve(bob, U256::from(50u32)).unwrap();

        test::set_caller(bob);
        let initial_events_len = test::recorded_events().len();
        let result = contract.transfer_from(alice, bob, transfer_amount);
        assert_eq!(result, Err(Error::InsufficientAllowance));

        let events = test::recorded_events();
        assert_eq!(events.len(), initial_events_len); // No event emitted on failure

        assert_eq!(contract.balance_of(alice), initial_supply);
        assert_eq!(contract.balance_of(bob), U256::zero());
        assert_eq!(contract.allowance(alice, bob), U256::from(50u32));
    }

    #[ink::test]
    fn transfer_from_fails_with_insufficient_balance() {
        let (alice, bob) = setup();
        let initial_supply = U256::from(1000u32);
        let mut contract = Erc20::new(initial_supply);
        let transfer_amount = U256::from(1001u32);

        contract.approve(bob, U256::from(2000u32)).unwrap();

        test::set_caller(bob);
        let initial_events_len = test::recorded_events().len();
        let result = contract.transfer_from(alice, bob, transfer_amount);
        assert_eq!(result, Err(Error::InsufficientBalance));

        let events = test::recorded_events();
        assert_eq!(events.len(), initial_events_len); // No event emitted on failure

        assert_eq!(contract.balance_of(alice), initial_supply);
        assert_eq!(contract.balance_of(bob), U256::zero());
        assert_eq!(contract.allowance(alice, bob), U256::from(2000u32));
    }

    #[ink::test]
    fn allowance_returns_zero_by_default() {
        let (alice, bob) = setup();
        let contract = Erc20::new(U256::from(1000u32));
        assert_eq!(contract.allowance(alice, bob), U256::zero());
    }

    #[ink::test]
    fn balance_returns_zero_by_default() {
        let (_alice, bob) = setup();
        let contract = Erc20::new(U256::from(1000u32));
        assert_eq!(contract.balance_of(bob), U256::zero());
    }
}