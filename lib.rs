#![cfg_attr(not(feature = "std"), no_std, no_main)]


mod test;
#[ink::contract]
mod erc20 {
    use ink::{
        U256,
        storage::Mapping,
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc20 {
        total_supply: U256,
        balances: Mapping<Address, U256>,
        allowances: Mapping<(Address, Address), U256>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<Address>,
        #[ink(topic)]
        to: Option<Address>,
        value: U256,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: Address,
        #[ink(topic)]
        spender: Address,
        value: U256,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(total_supply: U256) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });
            Self {
                total_supply,
                balances,
                allowances: Default::default(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> U256 {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: Address) -> U256 {
            self.balance_of_impl(&owner)
        }


        #[inline]
        fn balance_of_impl(&self, owner: &Address) -> U256 {
            self.balances.get(owner).unwrap_or_default()
        }


        #[ink(message)]
        pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
            self.allowance_impl(&owner, &spender)
        }


        #[inline]
        fn allowance_impl(&self, owner: &Address, spender: &Address) -> U256 {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }


        #[ink(message)]
        pub fn transfer(&mut self, to: Address, value: U256) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }


        #[ink(message)]
        pub fn approve(&mut self, spender: Address, value: U256) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

      
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            value: U256,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(&from, &to, value)?;
            #[allow(clippy::arithmetic_side_effects)]
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }


        fn transfer_from_to(
            &mut self,
            from: &Address,
            to: &Address,
            value: U256,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }
            #[allow(clippy::arithmetic_side_effects)]
            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_impl(to);
            self.balances
                .insert(to, &(to_balance.checked_add(value).unwrap()));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }
    }
}