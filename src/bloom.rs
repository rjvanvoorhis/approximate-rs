use probabilistic_collections::bloom::BloomFilter;
use std::hash::Hash;

use crate::utils::{KnowsSize, MembershipSupport};

#[derive(Debug)]
pub struct BloomFilterWrapper<T> {
    pub bloom_filter: BloomFilter<T>,
}

impl<T> BloomFilterWrapper<T>
where
    T: Hash,
{
/// Creates  a bloom filter with the specified false positive rate
///
/// ```rust
/// # use approximate_rs::{bloom::*, utils::*};
/// let keys: Vec<u32> = (1..1000).into_iter().collect();
/// let positive_keys: Vec<u32> = (1..1000).into_iter().collect();
/// let negative_keys: Vec<u32> = (1000..10_000).into_iter().collect();
/// let fpp = 0.01;
/// let tol = 0.001;
/// let bloom_filter: BloomFilterWrapper<u32> = BloomFilterWrapper::new(&positive_keys, 0.001);
/// let true_positives = positive_keys.iter().filter_map(|key| {match bloom_filter.contains(key) {true => Some(key), false => None}}).count();
/// let false_positives = negative_keys.iter().filter_map(|key| {match bloom_filter.contains(key) {true => Some(key), false => None}}).count();
/// let expected_fasle_positives = ((fpp + tol) * negative_keys.len() as f64) as usize;
/// assert!(true_positives == positive_keys.len());
/// assert!(false_positives < expected_fasle_positives);
/// ```
    pub fn new(keys: &Vec<T>, false_positive_rate: f64) -> Self {

        let mut bloom_filter: BloomFilter<T> = BloomFilter::new(keys.len(), false_positive_rate);
        keys.iter().for_each(|key| bloom_filter.insert(key));
        Self { bloom_filter }
    }
}

impl<T> MembershipSupport<T> for BloomFilterWrapper<T>
where
    T: Hash,
{
    fn contains(&self, item: &T) -> bool {
        self.bloom_filter.contains(item)
    }
}

impl<T> KnowsSize for BloomFilterWrapper<T>
where
    T: Hash + serde::ser::Serialize,
{
    fn size_in_bytes(&self) -> usize {
        bincode::serialized_size(&self.bloom_filter).unwrap() as usize
    }
}
