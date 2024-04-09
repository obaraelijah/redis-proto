use crate::types::{Score, Key};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::cmp::Ordering;

#[derive(PartialEq, Eq)]
pub struct SortedSetMember {
    pub score: Score,
    pub member: String,
}

impl SortedSetMember {
    fn new(key: &[u8], score: Score) -> Self {
        SortedSetMember {
            score,
            member: String::from_utf8_lossy(key).to_string(),
        }
    }
}

impl PartialOrd<SortedSetMember> for SortedSetMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unimplemented!()
    }
}

pub struct SortedSet {
    members_hash: HashMap<Key, Score>,
    scores: BTreeSet<SortedSetMember>,
}

