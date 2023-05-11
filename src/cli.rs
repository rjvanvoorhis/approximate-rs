use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Debug, Clone)]
pub enum QueryMode {
    BloomFilter,
    Mphf,
    Fingerprint,
}

#[derive(Args, Debug, Clone)]
pub struct BloomFilterArgs {
    #[arg(long)]
    /// The desired false positive rate
    pub fpp: f64,
}

#[derive(Args, Debug, Clone)]
pub struct FingerprintArgs {
    #[arg(long)]
    /// The number of bits to store for each keys fingerprint
    pub width: u8,
}

#[derive(Subcommand, Debug, Clone)]
pub enum QueryCommands {
    /// Use a bloom filter with a configurable false positive rate
    BloomFilter(BloomFilterArgs),
    /// Use a minimal perfect hash function
    Mphf,
    /// Use a fingerprint array with a configurable fingerprint size to tune the false positive rate
    Fingerprint(FingerprintArgs),
}

#[derive(Parser, Debug)]
/// Execute membership queries with different datastructures
pub struct Cli {
    #[command(subcommand)]
    pub command: QueryCommands,

    /// The number of keys to generate for the test
    #[arg(short, long, default_value = "100000")]
    pub total_keys: u32,

    /// The number of true positives
    #[arg(short, long, default_value = "1000")]
    pub positive_keys: u32,

    /// The number of characters in each key
    #[arg(short, long, default_value = "30")]
    pub kmer_size: u8,
}
