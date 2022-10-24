use std::{mem};
use borsh::{BorshDeserialize, BorshSerialize};
use pchain_types;
use pchain_sdk::{contract,init, action, view, contract_field, contract_methods};
use pchain_sdk::collections::{FastMap};

/// Note that this implementation is not at all gas-optimal, and is instead written for
/// terseness and clarity.
/// 
/// Known issue 1: applying #[contract(meta)] on impl EnglishCastles causes compiler to complain.
#[contract]
pub struct PRFC1Implementor {
    token: Token,
    balancemap: FastMap<pchain_types::PublicAddress, u64>,
    allowancesmap: FastMap<pchain_types::PublicAddress, FastMap<pchain_types::PublicAddress, u64>>
}

#[contract_methods(meta)]
impl PRFC1Implementor {

    #[init]
    fn init(){
        let init_owner = pchain_sdk::transaction::from_address();
        let mut balancemap = FastMap::new();
        balancemap.insert(&init_owner, 10000000000000000);
        PRFC1Implementor {
            token: Token{
                name: "Moon Rock".to_string(),
                symbol: "MOON".to_string(),
                decimals: 8 as u8,
                total_supply: 10000000000000000 as u64
            },
            balancemap: balancemap,
            allowancesmap: FastMap::new()
        }.set()
    }

    #[view]
    fn token(&self) -> Token {
        let token = self.token.clone();
        token
    }

    #[view]
    fn allowance(&self, owner_address: pchain_types::PublicAddress, spender_address: pchain_types::PublicAddress) -> u64 {
        
        let owner_allowances = match self.allowancesmap.get(&owner_address){
            Some(
                owner_allowances
            ) => owner_allowances,
            None => return 0 as u64
        };
        let spender_allowance = match owner_allowances.get(&spender_address){
            Some(
                spender_allowance
            ) => spender_allowance,
            None => return 0 as u64
        };
        spender_allowance
    }

    #[view]
    fn balance_of(&self, address: pchain_types::PublicAddress) -> u64 {
        let balance_of_address = match self.balancemap.get(&address){
            Some(
                balance
            ) => balance,
            None => return 0 as u64
        };
        balance_of_address
    }


    #[action]
    fn transfer(&mut self, to_address: pchain_types::PublicAddress, amount: u64) {
        let from_address = pchain_sdk::transaction::from_address();
        // assert_eq!(txn.from_address, self.get_owner(token_id));
        
        let to_account_balance =  self.balance_of(to_address);
        let from_account_balance = self.balance_of(from_address);

        if from_account_balance >= amount {
            let new_from_account_balance = from_account_balance - amount;
            let new_to_account_balance = to_account_balance + amount;
            self.balancemap.insert(&from_address, new_from_account_balance);
            self.balancemap.insert(&to_address, new_to_account_balance);
        }

        let event = TransferEvent {
            owner_address: from_address,
            recipient_address: to_address,
            amount,
        };
        
        pchain_sdk::emit_event(&event.topic(), &event.into_value());
    }

    #[action]
    fn transfer_from(& mut self, from_address: pchain_types::PublicAddress, to_address: pchain_types::PublicAddress, amount: u64) {
        let txn_from_address = pchain_sdk::transaction::from_address();
        // assert_eq!(txn.from_address, self.get_spender(token_id).unwrap());
        // assert_eq!(from_address, self.get_owner(token_id));
        let from_account_balance =  self.balance_of(from_address);
        let to_account_balance =  self.balance_of(to_address);
        let spender_allowance = self.allowance(from_address, txn_from_address);

        if spender_allowance >= amount {
            let new_from_account_balance = from_account_balance - amount;
            let new_spender_allowance = spender_allowance - amount;
            let new_to_account_balance = to_account_balance + amount;
            match self.allowancesmap.get_mut(&from_address) {
                Some(inner_map) => {
                    inner_map.insert(&from_address, new_spender_allowance);
                    self.balancemap.insert(&to_address, new_to_account_balance);
                    self.balancemap.insert(&from_address, new_from_account_balance);
                    let event = TransferEvent {
                        owner_address: from_address,
                        recipient_address: to_address,
                        amount
                    };
                    pchain_sdk::emit_event(&event.topic(), &event.into_value());
                },
                None => {
                    let event = TransferEvent {
                        owner_address: from_address,
                        recipient_address: to_address,
                        amount:0
                    };
                    pchain_sdk::emit_event(&event.topic(), &event.into_value());
                },
            }
         
        }

        
    }

    #[action]
    fn set_allowance(& mut self, spender_address: pchain_types::PublicAddress, amount: u64) {
        let txn_from_address = pchain_sdk::transaction::from_address();
        match self.balancemap.get(&txn_from_address){
            Some(balance) => {
                if balance >= amount {
                    match self.allowancesmap.get_mut(&txn_from_address) {
                        Some(inner_map) => {
                            inner_map.insert(&txn_from_address, amount);
                            let event = SetAllowanceEvent {
                                spender_address,
                                owner_address:txn_from_address,
                                amount
                            };
                            pchain_sdk::emit_event(&event.topic(), &event.into_value());
                        },
                        None => {
                            let event = SetAllowanceEvent {
                                spender_address,
                                owner_address:txn_from_address,
                                amount:0
                            };
                            pchain_sdk::emit_event(&event.topic(), &event.into_value());
                        },
                    }
                }
            },
            None => {
                let event = SetAllowanceEvent {
                    spender_address,
                    owner_address:txn_from_address,
                    amount:0
                };
                pchain_sdk::emit_event(&event.topic(), &event.into_value());
            },
        }    
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, PartialOrd, Clone)]
#[contract_field]
struct Token {
    pub name: String,
    pub symbol: String,
    decimals: u8,
    total_supply: u64
}

struct TransferEvent {
    owner_address: pchain_types::PublicAddress,
    recipient_address: pchain_types::PublicAddress,
    amount: u64,
}

struct SetAllowanceEvent {
    owner_address: pchain_types::PublicAddress,
    spender_address: pchain_types::PublicAddress,
    amount: u64,
}

impl TransferEvent {
    fn topic(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>() 
            + mem::size_of::<pchain_types::PublicAddress>() + 1);
        res.extend((0 as u8).to_le_bytes());
        res.extend(self.owner_address);
        res.extend(self.recipient_address);
        res
    }

    fn into_value(self) -> Vec<u8> {
        let mut res = Vec::with_capacity(
            mem::size_of::<u64>());
        res.extend(self.amount.to_le_bytes());
        res
    }
}

impl SetAllowanceEvent {
    fn topic(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>() 
            + mem::size_of::<pchain_types::PublicAddress>() + 1);
        res.extend((1 as u8).to_le_bytes());
        res.extend(self.owner_address);
        res.extend(self.spender_address);
        res
    }

    fn into_value(self) -> Vec<u8> {
        let mut res = Vec::with_capacity(
            mem::size_of::<u64>() 
        );
        res.extend(self.amount.to_le_bytes());
        res
    }
}
