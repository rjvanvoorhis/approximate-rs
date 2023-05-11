use std::{
    collections::HashSet,
    fmt::Debug,
    time::{Duration, Instant},
};

use rand::{prelude::Distribution, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

const NUCLEOTIDES: &[u8] = b"ACGT";

#[derive(Debug)]
pub struct SplitIndicies {
    pub positives: Vec<u32>,
    pub negatives: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SplitKeys {
    pub positives: Vec<String>,
    pub negatives: Vec<String>,
}

pub struct Kmers {
    pub k: u8,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExperimentResults {
    pub positive_keys: u32,
    pub negative_keys: u32,
    pub serialized_size: usize,
    pub false_positive_count: u32,
    pub false_negative_count: u32,
    pub negatives_query_duration: Duration,
    pub positives_query_duration: Duration,
}

impl Distribution<String> for Kmers {
    /// https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
    ///
    /// ```
    /// use approximate_rs::utils::{Kmers};
    /// use rand::distributions::Distribution;
    /// use rand::Rng;
    ///
    /// let kmer = Kmers {k: 10};
    /// let mut rng = rand::thread_rng();
    /// let sample = kmer.sample(&mut rng);
    /// println!("sample = {sample}");
    /// assert!(sample.len() == 10);
    /// ```
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> String {
        (0..self.k)
            .map(|_| {
                let idx = rng.gen_range(0..NUCLEOTIDES.len());
                NUCLEOTIDES[idx] as char
            })
            .collect()
    }
}

pub trait MembershipSupport<T> {
    fn contains(&self, item: &T) -> bool;
}

pub trait KnowsSize {
    fn size_in_bytes(&self) -> usize;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct QueryResults {
    pub false_positive_count: u32,
    pub total_duration: Duration,
    pub queries: u32,
    pub serialized_bytes: u32,
}

impl SplitKeys {
    pub fn new<R: rand::Rng + ?Sized>(
        rng: &mut R,
        positive_size: usize,
        total_size: usize,
        kmer_size: u8,
    ) -> Self {
        let mut positives: HashSet<String> = HashSet::with_capacity(positive_size);
        let mut negatives: HashSet<String> = HashSet::with_capacity(total_size - positive_size);
        let distribution = Kmers { k: kmer_size };
        for x in distribution.sample_iter(rng) {
            if positives.len() < positive_size {
                positives.insert(x);
            } else {
                negatives.insert(x);
                if negatives.len() + positive_size >= total_size {
                    break;
                }
            }
        }
        SplitKeys {
            positives: positives.into_iter().collect(),
            negatives: negatives.into_iter().collect(),
        }
    }
}

impl SplitIndicies {
    pub fn new(rng: &mut StdRng, positive_size: u32, total_size: u32) -> Self {
        let sample: Vec<u32> = (0..=total_size).collect();
        let mut mask = vec![false; total_size as usize];
        sample
            .choose_multiple(rng, positive_size as usize)
            .into_iter()
            .for_each(|&x| {
                mask[x as usize] = true;
            });
        let mut positives = Vec::with_capacity(positive_size as usize);
        // let mut positives: Vec<u32> = Vec::with_capacity(positive_size as usize);
        let mut negatives: Vec<u32> = Vec::with_capacity((total_size - positive_size) as usize);
        mask.into_iter()
            .enumerate()
            .for_each(|(value, is_positive)| {
                if is_positive {
                    positives.push(value as u32);
                } else {
                    negatives.push(value as u32)
                }
            });
        Self {
            positives,
            negatives,
        }
    }
}

pub fn run_experiment<M>(keys: &SplitKeys, amq: &M) -> ExperimentResults
where
    M: MembershipSupport<String> + KnowsSize,
{
    let mut results = ExperimentResults {
        positive_keys: keys.positives.len() as u32,
        negative_keys: keys.negatives.len() as u32,
        serialized_size: amq.size_in_bytes(),
        ..Default::default()
    };
    keys.positives.iter().for_each(|item| {
        let now = Instant::now();
        let is_present = amq.contains(item);
        if !is_present {
            results.false_negative_count += 1;
        };
        results.positives_query_duration += now.elapsed();
    });

    keys.negatives.iter().for_each(|item| {
        let now = Instant::now();
        let is_present = amq.contains(item);
        if is_present {
            results.false_positive_count += 1;
        };
        results.negatives_query_duration += now.elapsed();
    });

    results
}
