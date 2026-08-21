#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;
use {crate::types::TransactionResult, solana_signature::Signature, std::collections::VecDeque};

#[derive(Clone)]
pub struct TransactionHistory {
    map: HashMap<Signature, TransactionResult>,
    order: VecDeque<Signature>,
    capacity: usize,
}

impl TransactionHistory {
    pub fn new() -> Self {
        TransactionHistory {
            map: HashMap::default(),
            order: VecDeque::new(),
            capacity: 32,
        }
    }

    pub fn set_capacity(&mut self, new_cap: usize) {
        self.capacity = new_cap;
        while self.order.len() > new_cap {
            if let Some(evicted) = self.order.pop_front() {
                self.map.remove(&evicted);
            }
        }
    }

    pub fn get_transaction(&self, signature: &Signature) -> Option<&TransactionResult> {
        self.map.get(signature)
    }

    pub fn is_enabled(&self) -> bool {
        self.capacity != 0
    }

    pub fn add_new_transaction(&mut self, signature: Signature, result: TransactionResult) {
        if self.capacity == 0 {
            return;
        }
        if self.order.len() == self.capacity && !self.map.contains_key(&signature) {
            if let Some(evicted) = self.order.pop_front() {
                self.map.remove(&evicted);
            }
        }
        if self.map.insert(signature, result).is_none() {
            self.order.push_back(signature);
        }
    }

    pub fn check_transaction(&self, signature: &Signature) -> bool {
        self.map.contains_key(signature)
    }

    #[cfg(feature = "persistence-internal")]
    pub fn entries(&self) -> impl Iterator<Item = (&Signature, &TransactionResult)> {
        self.order
            .iter()
            .filter_map(|sig| self.map.get(sig).map(|res| (sig, res)))
    }

    #[cfg(feature = "persistence-internal")]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[cfg(feature = "persistence-internal")]
    pub fn from_entries(entries: Vec<(Signature, TransactionResult)>, capacity: usize) -> Self {
        let mut history = TransactionHistory::new();
        history.capacity = entries.len().max(capacity);
        for (signature, result) in entries {
            history.add_new_transaction(signature, result);
        }
        history.set_capacity(capacity);
        history
    }
}
