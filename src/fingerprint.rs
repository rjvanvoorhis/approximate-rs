use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use boomphf::Mphf;
use sucds::{int_vectors::CompactVector, Serializable};

use crate::utils::{KnowsSize, MembershipSupport};
const GAMMA: f64 = 1.7;
pub struct FingerprintArray<T> {
    mphf: Mphf<T>,
    int_vector: CompactVector,
    mask: u64,
}

impl<T> FingerprintArray<T>
where
    T: Hash + Debug,
{
    pub fn fingerprint(&self, item: &T) -> u64 {
        let mut hasher = DefaultHasher::default();
        item.hash(&mut hasher);
        hasher.finish() & self.mask
    }

    /// Create a fingerprint array with the specified width containing the given keys
    /// 
    /// ```rust
    /// use approximate_rs::{utils::*, fingerprint::*};
    /// let positive_keys: Vec<u32> = (1..=1000_u32).into_iter().collect();
    /// let negative_keys: Vec<u32> = (1001..=10_000_u32).into_iter().collect();
    /// let width = 7_u32;
    /// let base = 2_u32;
    /// let tol = 0.0005_f64;
    /// let expected_fpp = 1_f64 / (base.pow(width) as f64) + 0.0005_f64;
    /// let amq = FingerprintArray::<u32>::new(&positive_keys, width as usize);
    /// let true_positives = positive_keys
    ///   .iter()
    ///   .filter_map(|key| {if amq.contains(key) {Some(key)} else {None}})
    ///   .count();
    /// let false_positives = negative_keys
    ///   .iter()
    ///   .filter_map(|key| {if amq.contains(key) {Some(key)} else {None}})
    ///   .count();
    /// println!("Got {false_positives} false positives");
    /// let observed_fpp = (false_positives as f64) / (negative_keys.len() as f64);
    /// assert!(observed_fpp < expected_fpp);
    /// assert_eq!(true_positives, positive_keys.len());
    /// ```
    pub fn new(objects: &[T], fingerprint_size: usize) -> Self {
        let mphf = Mphf::new(GAMMA, objects);
        let mask = (1_u64 << fingerprint_size) - 1;
        let mut int_vector = CompactVector::with_capacity(objects.len(), fingerprint_size)
            .expect("The compact vector should initialize");
        (0..objects.len())
            .for_each(|_| int_vector.push_int(0).expect("The insert should be valid"));
        objects.iter().for_each(|item| {
            let mut hasher = DefaultHasher::new();
            item.hash(&mut hasher);
            let fingerprint = hasher.finish() & mask;
            int_vector
                .set_int(mphf.hash(item) as usize, fingerprint as usize)
                .expect("The hash value should be in bounds");
        });
        Self {
            mphf,
            mask,
            int_vector,
        }
    }
}

impl<T> MembershipSupport<T> for FingerprintArray<T>
where
    T: Hash + Debug,
{
    fn contains(&self, item: &T) -> bool {
        match self.mphf.try_hash(item) {
            None => false,
            Some(pos) => {
                let fingerprint = self.fingerprint(item);
                self.int_vector
                    .get_int(pos as usize)
                    .expect("The hash value should be in bounds")
                    == (fingerprint as usize)
            }
        }
    }
}

impl<T> KnowsSize for FingerprintArray<T>
where
    T: Hash + Debug,
{
    fn size_in_bytes(&self) -> usize {
        bincode::serialized_size(&self.mphf).unwrap() as usize
            + self.int_vector.size_in_bytes()
            + u64::size_of().unwrap()
    }
}
