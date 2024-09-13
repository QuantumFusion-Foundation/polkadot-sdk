// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{borrow::Borrow, path::{self, PathBuf}};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kvdb_nomtdb::NomtDB;
use kvdb_rocksdb::{Database, DatabaseConfig};
use kvdb::{DBOp, DBTransaction, KeyValueDB};
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use sc_client_api::{Backend as _, BlockImportOperation, NewBlockState, StateBackend};
use sc_client_db::{Backend, BlocksPruning, DatabaseSettings, DatabaseSource, PruningMode};
use sp_core::H256;
use sp_runtime::{
	testing::{Block as RawBlock, ExtrinsicWrapper, Header},
	StateVersion, Storage,
};
use tempfile::TempDir;

pub(crate) type Block = RawBlock<ExtrinsicWrapper<u64>>;

fn create_backend_nomt(temp_dir: &TempDir) -> NomtDB {
	// let path = temp_dir.path().to_owned();

	// let trie_cache_maximum_size = match config {
	// 	BenchmarkConfig::NoCache => None,
	// 	BenchmarkConfig::TrieNodeCache => Some(2 * 1024 * 1024 * 1024),
	// };

	// let settings = DatabaseSettings {
	// 	trie_cache_maximum_size,
	// 	state_pruning: Some(PruningMode::ArchiveAll),
	// 	source: DatabaseSource::NomtDb { path },
	// 	blocks_pruning: BlocksPruning::KeepAll,
	// };

	// Backend::new(settings, 100).expect("Creates backend")
    NomtDB::default()
}

fn create_backend_rocks() -> Database {
    let path = PathBuf::from("/Users/max/workspace/Rust/Profiterole/polkadot-sdk/kvdb_rocks_db_data");
	let opts = DatabaseConfig::with_columns(5);
    let db = Database::open(&opts, path).expect("Creates RocksDB");
    db
}

/// Generate the storage that will be used for the benchmark
///
/// Returns the `Vec<key>` and the `Vec<(key, value)>`
fn generate_storage() -> (Vec<Vec<u8>>, Vec<(Vec<u8>, Vec<u8>)>) {
	let mut rng = StdRng::seed_from_u64(353893213);

	let mut storage = Vec::new();
	let mut keys = Vec::new();

	for _ in 0..1_000 {
		let key_len: usize = rng.gen_range(32..128);
		let key = (&mut rng)
			.sample_iter(Uniform::new_inclusive(0, 255))
			.take(key_len)
			.collect::<Vec<u8>>();

		let value_len: usize = rng.gen_range(20..60);
		let value = (&mut rng)
			.sample_iter(Uniform::new_inclusive(0, 255))
			.take(value_len)
			.collect::<Vec<u8>>();

		keys.push(key.clone());
		storage.push((key, value));
	}

	(keys, storage)
}

fn read_write_benchmarks(c: &mut Criterion) {
	sp_tracing::try_init_simple();

	let (keys, storage) = generate_storage();
	let path = TempDir::new().expect("Creates temporary directory");

	let mut group = c.benchmark_group("Writing generated storage");
	group.sample_size(20);

	let mut bench_multiple_values_write = |desc, multiplier| {
		let db = create_backend_nomt(&path);
        let rdb = create_backend_rocks();

		group.bench_function("Writes to nomt", |b| {
			b.iter_batched(
				|| storage.clone(),
				|data| {
                    let mut trans = db.transaction();
					for (k, v) in data.iter() {
                        trans.put(0, k, v)
					}
                    db.write(trans).expect("Doesn't fail");
				},
				BatchSize::SmallInput,
			)
		});

		group.bench_function("Writes to rocks", |b| {
			b.iter_batched(
				|| storage.clone(),
				|data| {
                    let mut trans = rdb.transaction();
					for (k, v) in data.iter() {
                        trans.put(0, k, v)
					}
                    rdb.write(trans).expect("Doesn't fail");
				},
				BatchSize::SmallInput,
			)
		});
	};

	bench_multiple_values_write(
		"write some data",
		1,
	);

	// let mut group = c.benchmark_group("Reading storage");
	// group.sample_size(20);

	let mut bench_multiple_values_read = |desc, multiplier| {
		let db = create_backend_nomt(&path);
        let rdb = create_backend_rocks();

		group.bench_function("Reads from Nomt", |b| {
			b.iter_batched(
				|| storage.clone(),
				|data| {
                    // let mut trans = db.transaction();
					for (k, v) in data.iter() {
                        let val = db.get(0, k).expect("Doesn't fail");
                        let v = v.clone();
                        assert_eq!(val.unwrap(), v);
					}
				},
				BatchSize::SmallInput,
			)
		});

		group.bench_function("Reads from Rocks", |b| {
			b.iter_batched(
				|| storage.clone(),
				|data| {
                    // let mut trans = db.transaction();
					for (k, v) in data.iter() {
                        let val = rdb.get(0, k).expect("Doesn't fail");
                        let v = v.clone();
                        assert_eq!(val.unwrap(), v);
					}
				},
				BatchSize::SmallInput,
			)
		});
	};

	bench_multiple_values_read(
		"read some data",
		1,
	);
	// bench_multiple_values(BenchmarkConfig::NoCache, "no cache and reading each key once", 1);

	// bench_multiple_values(
	// 	BenchmarkConfig::TrieNodeCache,
	// 	"with trie node cache and reading 4 times each key in a row",
	// 	4,
	// );
	// bench_multiple_values(
	// 	BenchmarkConfig::NoCache,
	// 	"no cache and reading 4 times each key in a row",
	// 	4,
	// );

	group.finish();

	// let mut group = c.benchmark_group("Reading a single value");

	// let mut bench_single_value = |config, desc, multiplier| {
	// 	let backend = create_backend(config, &path);

	// 	group.bench_function(desc, |b| {
	// 		b.iter_batched(
	// 			|| backend.state_at(block_hash).expect("Creates state"),
	// 			|state| {
	// 				for key in keys.iter().take(1).cycle().take(multiplier) {
	// 					let _ = state.storage(&key).expect("Doesn't fail").unwrap();
	// 				}
	// 			},
	// 			BatchSize::SmallInput,
	// 		)
	// 	});
	// };

	// bench_single_value(
	// 	BenchmarkConfig::TrieNodeCache,
	// 	"with trie node cache and reading the key once",
	// 	1,
	// );
	// bench_single_value(BenchmarkConfig::NoCache, "no cache and reading the key once", 1);

	// bench_single_value(
	// 	BenchmarkConfig::TrieNodeCache,
	// 	"with trie node cache and reading 4 times each key in a row",
	// 	4,
	// );
	// bench_single_value(
	// 	BenchmarkConfig::NoCache,
	// 	"no cache and reading 4 times each key in a row",
	// 	4,
	// );

	// group.finish();

	// let mut group = c.benchmark_group("Hashing a value");

	// let mut bench_single_value = |config, desc, multiplier| {
	// 	let backend = create_backend(config, &path);

	// 	group.bench_function(desc, |b| {
	// 		b.iter_batched(
	// 			|| backend.state_at(block_hash).expect("Creates state"),
	// 			|state| {
	// 				for key in keys.iter().take(1).cycle().take(multiplier) {
	// 					let _ = state.storage_hash(&key).expect("Doesn't fail").unwrap();
	// 				}
	// 			},
	// 			BatchSize::SmallInput,
	// 		)
	// 	});
	// };

	// bench_single_value(
	// 	BenchmarkConfig::TrieNodeCache,
	// 	"with trie node cache and hashing the key once",
	// 	1,
	// );
	// bench_single_value(BenchmarkConfig::NoCache, "no cache and hashing the key once", 1);

	// bench_single_value(
	// 	BenchmarkConfig::TrieNodeCache,
	// 	"with trie node cache and hashing 4 times each key in a row",
	// 	4,
	// );
	// bench_single_value(
	// 	BenchmarkConfig::NoCache,
	// 	"no cache and hashing 4 times each key in a row",
	// 	4,
	// );

	// group.finish();

	// let mut group = c.benchmark_group("Hashing `:code`");

	// let mut bench_single_value = |config, desc| {
	// 	let backend = create_backend(config, &path);

	// 	group.bench_function(desc, |b| {
	// 		b.iter_batched(
	// 			|| backend.state_at(block_hash).expect("Creates state"),
	// 			|state| {
	// 				let _ = state
	// 					.storage_hash(sp_core::storage::well_known_keys::CODE)
	// 					.expect("Doesn't fail")
	// 					.unwrap();
	// 			},
	// 			BatchSize::SmallInput,
	// 		)
	// 	});
	// };

	// bench_single_value(BenchmarkConfig::TrieNodeCache, "with trie node cache");
	// bench_single_value(BenchmarkConfig::NoCache, "no cache");

	// group.finish();
}

criterion_group!(benches, read_write_benchmarks);
criterion_main!(benches);
