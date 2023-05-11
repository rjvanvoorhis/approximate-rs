# Assignment 3: Comparing Different AMQs

This project provides three main structs

## BloomFilterWrapper

This is a thin wrapper around the `probabilistic-collection` crate's "BloomFilter" datastructure. A bloom filter may be initialized with a given target false positive rate. The wrapper can then be used to execute membership queries on the keys making up the construction set

```rust
# use approximate_rs::{bloom::*, utils::*};
let keys: Vec<u32> = (1..1000).into_iter().collect();
let positive_keys: Vec<u32> = (1..1000).into_iter().collect();
let negative_keys: Vec<u32> = (1000..10_000).into_iter().collect();
let fpp = 0.01;
let tol = 0.001;
let bloom_filter: BloomFilterWrapper<u32> = BloomFilterWrapper::new(&positive_keys, 0.001);
let true_positives = positive_keys.iter().filter_map(|key| {match bloom_filter.contains(key) {true => Some(key), false => None}}).count();
let false_positives = negative_keys.iter().filter_map(|key| {match bloom_filter.contains(key) {true => Some(key), false => None}}).count();
let expected_fpp = ((fpp + tol) * negative_keys.len() as f64) as usize;
assert!(true_positives == positive_keys.len());
assert!(false_positives < expected_fpp);
```

## MphfWrapper

Like the BloomFilterWrapper this struct is mostly just a wrapper around a 3rd party library. In this case it wraps the Mphf struct from the boomphf crate. Like the BloomFilterWrapper it is initialized with a set of positive keys and then supports membership queries on this set with a "contains" method.

```rust
# use approximate_rs::{mphf::*, utils::*};
let keys: Vec<u32> = (1..1000).into_iter().collect();
let positive_keys: Vec<u32> = (1..1000).into_iter().collect();
let mphf: MphfWrapper<u32> = MphfWrapper::new(&positive_keys);
let true_positives = positive_keys.iter().filter_map(|key| {match mphf.contains(key) {true => Some(key), false => None}}).count();
assert!(true_positives == positive_keys.len());
```

## Fingerprint Array

The FingerprintArray accepts a set of keys and a desired width, w, is the number of bits to store as the fingerprint for each key. The width determines the fasle positive rate of the datastructure and approaches $FPP \approx 1 / 2^w$.

```rust
use approximate_rs::{utils::*, fingerprint::*};
let positive_keys: Vec<u32> = (1..=1000_u32).into_iter().collect();
let negative_keys: Vec<u32> = (1001..=10_000_u32).into_iter().collect();
let width = 7_u32;
let base = 2_u32;
let tol = 0.0005_f64;
let expected_fpp = 1_f64 / (base.pow(width) as f64) + 0.0005_f64;
let amq = FingerprintArray::<u32>::new(&positive_keys, width as usize);
let true_positives = positive_keys
  .iter()
  .filter_map(|key| {if amq.contains(key) {Some(key)} else {None}})
  .count();
let false_positives = negative_keys
  .iter()
  .filter_map(|key| {if amq.contains(key) {Some(key)} else {None}})
  .count();
println!("Got {false_positives} false positives");
let observed_fpp = (false_positives as f64) / (negative_keys.len() as f64);
assert!(observed_fpp < expected_fpp);
assert_eq!(true_positives, positive_keys.len());
```

# Experiment

This project includes a test suite that can be executed by doing the following

```bash
cargo build --release
./target/release/experiment --help
```

This will output the help text for the experiment cli:

```
Execute membership queries with different datastructures

Usage: experiment [OPTIONS] <COMMAND>

Commands:
  bloom-filter  Execute membership queries using a bloom filter with a configurable false positive rate
  mphf          Execute membership queries using a minimal perfect hash function
  fingerprint   Execute queries using a fingerprint array with a configurable fingerprint size to tune the false positive rate
  help          Print this message or the help of the given subcommand(s)

Options:
  -t, --total-keys <TOTAL_KEYS>        The number of keys to generate for the test [default: 100000]
  -p, --positive-keys <POSITIVE_KEYS>  The number of true positives [default: 1000]
  -k, --kmer-size <KMER_SIZE>          The number of characters in each key [default: 30]
  -h, --help                           Print help
```

The experiment can be executed and will produce results in the following format:

```bash
./target/release/experiment --positive-keys 5000 --total-keys 50000 fingerprint --width 3 | jq -r
```

and will produce output in the following format

```
{
  "positive_keys": 5000,
  "negative_keys": 45000,
  "serialized_size": 4232,
  "false_positive_count": 5520,
  "false_negative_count": 0,
  "negatives_query_duration": {
    "secs": 0,
    "nanos": 4066600
  },
  "positives_query_duration": {
    "secs": 0,
    "nanos": 374881
  }
}
```

# References

2015\. Clap. https://github.com/clap-rs
Shunsuke Kanda. 2018. sucds. https://github.com/kampersanda/sucds 

Patrick Marks. 2021. boomphf. https://github.com/10XGenomics/rust-boomphf

Tolnay David and Tryzelaar Erick. 2017. Serde. https://github.com/serde-rs/serde

Jeffery Xiao. 2018. probabilistic-collections. https://gitlab.com/jeffrey-xiao/probabilistic-collections-rs
