use {
    crate::accounts_db::AccountsMap,
    parking_lot::RwLock,
    solana_account::{Account, AccountSharedData, ReadableAccount},
    solana_address::Address,
    solana_hash::Hash,
    solana_sysvar::Sysvar,
    solana_sysvar_id::SysvarId,
    std::sync::Arc,
    wincode::DeserializeOwned,
};

/// Blockhash shared between the SVM and its readers; Clone snapshots rather than aliases
pub(crate) struct SharedHash(Arc<RwLock<Hash>>);

impl SharedHash {
    pub(crate) fn new(hash: Hash) -> Self {
        SharedHash(Arc::new(RwLock::new(hash)))
    }

    pub(crate) fn get(&self) -> Hash {
        *self.0.read()
    }

    pub(crate) fn set(&self, hash: Hash) {
        *self.0.write() = hash;
    }

    pub(crate) fn share(&self) -> Arc<RwLock<Hash>> {
        Arc::clone(&self.0)
    }
}

impl Clone for SharedHash {
    fn clone(&self) -> Self {
        SharedHash::new(self.get())
    }
}

/// Thread-safe read handle sharing the SVM's account map and latest blockhash
#[derive(Clone)]
pub struct LiteSvmReader {
    accounts: Arc<RwLock<AccountsMap>>,
    latest_blockhash: Arc<RwLock<Hash>>,
}

impl LiteSvmReader {
    pub(crate) fn new(
        accounts: Arc<RwLock<AccountsMap>>,
        latest_blockhash: Arc<RwLock<Hash>>,
    ) -> Self {
        Self {
            accounts,
            latest_blockhash,
        }
    }

    pub fn get_account(&self, address: &Address) -> Option<Account> {
        self.get_account_shared(address).map(Into::into)
    }

    pub fn get_account_shared(&self, address: &Address) -> Option<AccountSharedData> {
        self.accounts.read().get(address).cloned()
    }

    pub fn get_balance(&self, address: &Address) -> Option<u64> {
        self.accounts.read().get(address).map(|acc| acc.lamports())
    }

    pub fn latest_blockhash(&self) -> Hash {
        *self.latest_blockhash.read()
    }

    pub fn get_sysvar<T>(&self) -> T
    where
        T: Sysvar + SysvarId + DeserializeOwned<Dst = T>,
    {
        T::deserialize_from(self.get_account_shared(&T::id()).unwrap().data()).unwrap()
    }

    /// Runs f over every account under one read guard
    pub fn scan_accounts<R>(
        &self,
        f: impl FnOnce(&mut dyn Iterator<Item = (&Address, &AccountSharedData)>) -> R,
    ) -> R {
        let map = self.accounts.read();
        let mut iter = map.iter();
        f(&mut iter)
    }

    /// Returns all accounts owned by the given program, with their addresses
    pub fn get_program_accounts(&self, program_id: &Address) -> Vec<(Address, Account)> {
        self.scan_accounts(|iter| {
            iter.filter(|(_, account)| account.owner() == program_id)
                .map(|(address, account)| (*address, account.clone().into()))
                .collect()
        })
    }
}
