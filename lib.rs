#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod polka_token {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[allow(clippy::cast_possible_truncation)]  // Silence the false positive on Custom(String)
    pub enum PSP22Error {
        InsufficientBalance,
        InsufficientAllowance,
        Custom(String),
    }

    #[ink(storage)]
    pub struct PspCoin {
        total_supply: u128,
        balances: Mapping<AccountId, u128>,
        allowances: Mapping<(AccountId, AccountId), u128>,
        name: String,
        symbol: String,
        decimals: u8,
    }

    impl Default for PspCoin {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PspCoin {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                total_supply: 0,
                balances: Mapping::default(),
                allowances: Mapping::default(),
                name: String::from("PolkaToken"),
                symbol: String::from("PLKT"),
                decimals: 18,
            }
        }

        #[ink(constructor)]
        pub fn new_with_supply(total_supply: u128, name: String, symbol: String) -> Self {
            let caller = Self::env().caller();
            let mut balances = Mapping::default();
            balances.insert(caller, &total_supply);

            Self {
                total_supply,
                balances,
                allowances: Mapping::default(),
                name,
                symbol,
                decimals: 18,
            }
        }

        // PSP22 Core Methods
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u128 {
            self.balances.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: u128, _data: Vec<u8>) -> Result<(), PSP22Error> {
            let from = self.env().caller();
            self._transfer(from, to, value)
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            
            if caller != from {
                let allowance = self.allowances.get((from, caller)).unwrap_or(0);
                if allowance < value {
                    return Err(PSP22Error::InsufficientAllowance);
                }
                let new_allowance = allowance.checked_sub(value).ok_or(PSP22Error::Custom(String::from("arithmetic underflow")))?;
                self.allowances.insert((from, caller), &new_allowance);
            }

            self._transfer(from, to, value)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            if owner == spender {
                return Ok(());
            }
            self.allowances.insert((owner, spender), &value);
            Ok(())
        }

        #[ink(message)]
        pub fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            if owner == spender || delta_value == 0 {
                return Ok(());
            }
            let current_allowance = self.allowances.get((owner, spender)).unwrap_or(0);
            let new_allowance = current_allowance.checked_add(delta_value).ok_or(PSP22Error::Custom(String::from("arithmetic overflow")))?;
            self.allowances.insert((owner, spender), &new_allowance);
            Ok(())
        }

        #[ink(message)]
        pub fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            if owner == spender || delta_value == 0 {
                return Ok(());
            }
            let current_allowance = self.allowances.get((owner, spender)).unwrap_or(0);
            if current_allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }
            let new_allowance = current_allowance.checked_sub(delta_value).ok_or(PSP22Error::Custom(String::from("arithmetic underflow")))?;
            self.allowances.insert((owner, spender), &new_allowance);
            Ok(())
        }

        // PSP22 Metadata Methods
        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            Some(self.name.clone())
        }

        #[ink(message)]
        pub fn symbol(&self) -> Option<String> {
            Some(self.symbol.clone())
        }

        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.decimals
        }

        // Mintable
        #[ink(message)]
        pub fn mint(&mut self, value: u128) -> Result<(), PSP22Error> {
            if value == 0 {
                return Ok(());
            }
            
            let caller = self.env().caller();
            let new_supply = self.total_supply.checked_add(value)
                .ok_or(PSP22Error::Custom(String::from("max supply exceeded")))?;
            
            self.total_supply = new_supply;
            let balance = self.balances.get(caller).unwrap_or(0);
            let new_balance = balance.checked_add(value).ok_or(PSP22Error::Custom(String::from("arithmetic overflow")))?;
            self.balances.insert(caller, &new_balance);
            
            Ok(())
        }

        // Burnable
        #[ink(message)]
        pub fn burn(&mut self, value: u128) -> Result<(), PSP22Error> {
            if value == 0 {
                return Ok(());
            }

            let caller = self.env().caller();
            let balance = self.balances.get(caller).unwrap_or(0);
            
            if balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            let new_balance = balance.checked_sub(value).ok_or(PSP22Error::Custom(String::from("arithmetic underflow")))?;
            self.balances.insert(caller, &new_balance);
            self.total_supply = self.total_supply.checked_sub(value).ok_or(PSP22Error::Custom(String::from("arithmetic underflow")))?;
            
            Ok(())
        }

        // Internal helper
        fn _transfer(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
        ) -> Result<(), PSP22Error> {
            if from == to || value == 0 {
                return Ok(());
            }

            let from_balance = self.balances.get(from).unwrap_or(0);
            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            let new_from_balance = from_balance.checked_sub(value).ok_or(PSP22Error::Custom(String::from("arithmetic underflow")))?;
            self.balances.insert(from, &new_from_balance);
            let to_balance = self.balances.get(to).unwrap_or(0);
            let new_to_balance = to_balance.checked_add(value).ok_or(PSP22Error::Custom(String::from("arithmetic overflow")))?;
            self.balances.insert(to, &new_to_balance);

            Ok(())
        }
    }
}