use std::{collections::{HashMap, HashSet}, task::Waker};
use seahash::hash;

pub type Receipt = u32;

#[derive(Hash, Debug, PartialEq, Eq)]
pub enum KeyTypes {
    List(u64),
} 

impl KeyTypes {
    pub fn list(key: &[u8]) -> KeyTypes {
        KeyTypes::List(hash(key))
    }
}

#[derive(Default, Debug)]
pub struct RecieptMap {
    counter: Receipt,
    wakers: HashMap<Receipt, Waker>,
    timed_out: HashSet<Receipt>,
    keys: HashMap<KeyTypes, Vec<Receipt>>,
}

impl RecieptMap {
    pub fn get_receipt(&mut self) -> Receipt {
        self.counter += 1;
        self.counter
    }

    pub fn insert(&mut self, receipt: Receipt, item: Waker, key: KeyTypes) {
        self.wakers.insert(receipt, item);
        self.keys.entry(key).or_default().push(receipt);
    }

    pub fn receipt_timed_out(&self, receipt: Receipt) -> bool {
        self.timed_out.contains(&receipt)
    }
}