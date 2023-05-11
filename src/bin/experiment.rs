use approximate_rs::{
    bloom::BloomFilterWrapper,
    cli::{Cli, QueryCommands},
    fingerprint::FingerprintArray,
    mphf::MphfWrapper,
    utils::{run_experiment, SplitKeys},
};
use clap::Parser;
use eyre::{Context, Result};
use rand::{rngs::StdRng, SeedableRng};

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut rng = StdRng::seed_from_u64(42);
    let keys = SplitKeys::new(
        &mut rng,
        args.positive_keys as usize,
        args.total_keys as usize,
        args.kmer_size,
    );
    let results = match args.command {
        QueryCommands::Fingerprint(x) => {
            let fa = FingerprintArray::new(&keys.positives, x.width as usize);
            run_experiment(&keys, &fa)
            // assert_no_false_negatives(&keys.positives, &fa);
            // test_false_positive_rate(&keys.negatives, &fa)
        }
        QueryCommands::Mphf => {
            let mphf = MphfWrapper::new(&keys.positives);
            // assert_no_false_negatives(&keys.positives, &mphf);
            run_experiment(&keys, &mphf)
            // test_false_positive_rate(&keys.negatives, &mphf)
        }
        QueryCommands::BloomFilter(x) => {
            let bf = BloomFilterWrapper::new(&keys.positives, x.fpp);
            run_experiment(&keys, &bf)
            // assert_no_false_negatives(&keys.positives, &bf);
            // test_false_positive_rate(&keys.negatives, &bf)
        }
    };
    println!(
        "{}",
        serde_json::to_string(&results).wrap_err("Could not serialize results")?
    );
    Ok(())
}
