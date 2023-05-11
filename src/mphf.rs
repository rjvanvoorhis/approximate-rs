use std::{fmt::Debug, hash::Hash};

use boomphf::Mphf;

use crate::utils::{KnowsSize, MembershipSupport};
const GAMMA: f64 = 1.7;
pub struct MphfWrapper<T> {
    mphf: Mphf<T>,
}

impl<T> MphfWrapper<T>
where
    T: Hash + Debug,
{
    pub fn new(objects: &[T]) -> Self {
        let mphf = Mphf::new(GAMMA, objects);
        Self { mphf }
    }
}

impl<T> MembershipSupport<T> for MphfWrapper<T>
where
    T: Hash + Debug,
{
    fn contains(&self, item: &T) -> bool {
        self.mphf.try_hash(item).is_some()
    }
}

impl<T> KnowsSize for MphfWrapper<T> {
    fn size_in_bytes(&self) -> usize {
        bincode::serialized_size(&self.mphf).expect("mphf should be serializable") as usize
    }
}
