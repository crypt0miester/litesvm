use {
    litesvm::{LiteSVM, LiteSvmReader},
    solana_address::Address,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::sync::atomic::{AtomicBool, Ordering},
};

const TRANSFERS: u64 = 500;
// Rent-exempt for a zero-data account, so the first transfer clears the rent check
const LAMPORTS_PER_TRANSFER: u64 = 1_000_000;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn readers_run_beside_the_writer() {
    assert_send_sync::<LiteSvmReader>();

    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    let payer_pk = payer.pubkey();
    let recipient = Address::new_unique();
    svm.airdrop(&payer_pk, 10_000_000_000).unwrap();

    let reader = svm.reader();
    let blockhash = svm.latest_blockhash();
    assert_eq!(reader.latest_blockhash(), blockhash);

    // Readers exit on this flag, set even if the writer panics
    let done = AtomicBool::new(false);
    struct Release<'a>(&'a AtomicBool);
    impl Drop for Release<'_> {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    std::thread::scope(|s| {
        let _release = Release(&done);
        for _ in 0..3 {
            let reader = reader.clone();
            let done = &done;
            s.spawn(move || {
                let mut last_seen = 0u64;
                while !done.load(Ordering::Acquire) {
                    // Balances only move forward, and a debit lands with its credit
                    let seen = reader.get_balance(&recipient).unwrap_or(0);
                    assert!(seen >= last_seen, "recipient balance went backwards");
                    assert_eq!(seen % LAMPORTS_PER_TRANSFER, 0);
                    last_seen = seen;
                    reader.scan_accounts(|iter| {
                        let mut found = 0;
                        for (address, _) in iter {
                            if address == &payer_pk || address == &recipient {
                                found += 1;
                            }
                        }
                        assert!(found >= 1);
                    });
                }
                last_seen
            });
        }

        for _ in 0..TRANSFERS {
            // A fresh blockhash per transfer keeps every signature unique
            let blockhash = svm.latest_blockhash();
            let msg = Message::new_with_blockhash(
                &[solana_system_interface::instruction::transfer(
                    &payer_pk,
                    &recipient,
                    LAMPORTS_PER_TRANSFER,
                )],
                Some(&payer_pk),
                &blockhash,
            );
            let tx = Transaction::new(&[&payer], msg, blockhash);
            svm.send_transaction(tx).unwrap();
            svm.expire_blockhash();
        }
    });

    assert_eq!(
        reader.get_balance(&recipient).unwrap(),
        TRANSFERS * LAMPORTS_PER_TRANSFER
    );
    assert_eq!(
        svm.get_balance(&recipient).unwrap(),
        TRANSFERS * LAMPORTS_PER_TRANSFER
    );
}
