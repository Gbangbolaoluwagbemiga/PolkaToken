#[cfg(test)]
mod tests {
    use super::*;
    use ink::env::{
        test::{default_accounts, set_caller, recorded_events, DefaultAccounts},
        DefaultEnvironment,
    };
    use ink::prelude::Address;
    use ink::U256;

    // Helper to set up the environment and create a new contract instance
    fn setup() -> (DefaultAccounts<DefaultEnvironment>, Erc20) {
        let accounts = default_accounts();
        set_caller::<DefaultEnvironment>(accounts.alice);
        let total_supply = U256::from(1000);
        let contract = Erc20::new(total_supply);
        (accounts, contract)
    }

    #[ink::test]
    fn new_works() {
        let (accounts, contract) = setup();
        assert_eq!(contract.total_supply(), U256::from(1000));
        assert_eq!(contract.balance_of(accounts.alice), U256::from(1000));
        assert_eq!(contract.balance_of(accounts.bob), U256::from(0));

        // Check Transfer event
        let events = recorded_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1, "Expected one Transfer event");
        let event = events[0].clone();
        assert_eq!(event.topics.len(), 3, "Expected three topics");
        let transfer_event = event.as_event::<Transfer>().unwrap().unwrap();
        assert_eq!(transfer_event.from, None);
        assert_eq!(transfer_event.to, Some(accounts.alice));
        assert_eq!(transfer_event.value, U256::from(1000));
    }

    #[ink::test]
    fn transfer_works() {
        let (accounts, mut contract) = setup();
        let result = contract.transfer(accounts.bob, U256::from(100));
        assert_eq!(result, Ok(()));
        assert_eq!(contract.balance_of(accounts.alice), U256::from(900));
        assert_eq!(contract.balance_of(accounts.bob), U256::from(100));

        // Check Transfer event
        let events = recorded_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1, "Expected one Transfer event");
        let event = events[0].clone();
        let transfer_event = event.as_event::<Transfer>().unwrap().unwrap();
        assert_eq!(transfer_event.from, Some(accounts.alice));
        assert_eq!(transfer_event.to, Some(accounts.bob));
        assert_eq!(transfer_event.value, U256::from(100));
    }

    #[ink::test]
    fn transfer_insufficient_balance_fails() {
        let (accounts, mut contract) = setup();
        let result = contract.transfer(accounts.bob, U256::from(1001));
        assert_eq!(result, Err(Error::InsufficientBalance));
        assert_eq!(contract.balance_of(accounts.alice), U256::from(1000));
        assert_eq!(contract.balance_of(accounts.bob), U256::from(0));
    }

    #[ink::test]
    fn approve_works() {
        let (accounts, mut contract) = setup();
        let result = contract.approve(accounts.bob, U256::from(200));
        assert_eq!(result, Ok(()));
        assert_eq!(contract.allowance(accounts.alice, accounts.bob), U256::from(200));

        // Check Approval event
        let events = recorded_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1, "Expected one Approval event");
        let event = events[0].clone();
        let approval_event = event.as_event::<Approval>().unwrap().unwrap();
        assert_eq!(approval_event.owner, accounts.alice);
        assert_eq!(approval_event.spender, accounts.bob);
        assert_eq!(approval_event.value, U256::from(200));
    }

    #[ink::test]
    fn transfer_from_works() {
        let (accounts, mut contract) = setup();
        // Alice approves Bob to spend 200 tokens
        contract.approve(accounts.bob, U256::from(200)).unwrap();
        // Switch to Bob as caller
        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = contract.transfer_from(accounts.alice, accounts.charlie, U256::from(150));
        assert_eq!(result, Ok(()));
        assert_eq!(contract.balance_of(accounts.alice), U256::from(850));
        assert_eq!(contract.balance_of(accounts.charlie), U256::from(150));
        assert_eq!(contract.allowance(accounts.alice, accounts.bob), U256::from(50));

        // Check Transfer event
        let events = recorded_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 2, "Expected Approval and Transfer events");
        let transfer_event = events[1].clone().as_event::<Transfer>().unwrap().unwrap();
        assert_eq!(transfer_event.from, Some(accounts.alice));
        assert_eq!(transfer_event.to, Some(accounts.charlie));
        assert_eq!(transfer_event.value, U256::from(150));
    }

    #[ink::test]
    fn transfer_from_insufficient_allowance_fails() {
        let (accounts, mut contract) = setup();
        // Alice approves Bob for 100 tokens
        contract.approve(accounts.bob, U256::from(100)).unwrap();
        // Switch to Bob
        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = contract.transfer_from(accounts.alice, accounts.charlie, U256::from(101));
        assert_eq!(result, Err(Error::InsufficientAllowance));
        assert_eq!(contract.balance_of(accounts.alice), U256::from(1000));
        assert_eq!(contract.balance_of(accounts.charlie), U256::from(0));
        assert_eq!(contract.allowance(accounts.alice, accounts.bob), U256::from(100));
    }

    #[ink::test]
    fn transfer_from_insufficient_balance_fails() {
        let (accounts, mut contract) = setup();
        // Alice approves Bob for 2000 tokens (more than her balance)
        contract.approve(accounts.bob, U256::from(2000)).unwrap();
        // Switch to Bob
        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = contract.transfer_from(accounts.alice, accounts.charlie, U256::from(1001));
        assert_eq!(result, Err(Error::InsufficientBalance));
        assert_eq!(contract.balance_of(accounts.alice), U256::from(1000));
        assert_eq!(contract.balance_of(accounts.charlie), U256::from(0));
        assert_eq!(contract.allowance(accounts.alice, accounts.bob), U256::from(2000));
    }
}