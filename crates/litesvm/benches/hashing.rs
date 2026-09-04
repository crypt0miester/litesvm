use {
    criterion::{criterion_group, criterion_main, Criterion},
    solana_address::Address,
    std::{collections::hash_map::RandomState, hash::BuildHasher, hint::black_box},
};

// RandomState is the std HashMap hasher, so this is what the accounts map pays without hashbrown
#[inline(never)]
fn std_default(address: &Address, hash_builder: &RandomState) -> u64 {
    hash_builder.hash_one(address)
}

#[cfg(feature = "hashbrown")]
#[inline(never)]
fn hashbrown(address: &Address, hash_builder: &hashbrown::DefaultHashBuilder) -> u64 {
    hash_builder.hash_one(address)
}

fn criterion_benchmark(c: &mut Criterion) {
    let address = Address::new_unique();

    let mut group = c.benchmark_group("hashers");

    group.bench_function("default", |b| {
        let hash_builder = RandomState::new();

        b.iter(|| {
            black_box(std_default(&address, &hash_builder));
        })
    });

    #[cfg(feature = "hashbrown")]
    group.bench_function("foldhash", |b| {
        let hash_builder = hashbrown::DefaultHashBuilder::default();

        b.iter(|| {
            black_box(hashbrown(&address, &hash_builder));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
