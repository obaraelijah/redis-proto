use crate::types::{Score, Key, Count};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use crate::ops::RVec;
use std::collections::hash_map::Entry;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SortedSetMember {
    pub score: Score,
    pub member: String,
}

impl PartialOrd<SortedSetMember> for SortedSetMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let score_cmp = self.score.cmp(&other.score);
        if let Ordering::Equal = score_cmp {
            return Some(self.member.cmp(&other.member));
        }
        Some(score_cmp)
    }
}

impl Ord for SortedSetMember {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl SortedSetMember {
    fn new(key: &[u8], score: Score) -> Self {
        SortedSetMember {
            score,
            member: String::from_utf8_lossy(key).to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SortedSet {
    members_hash: HashMap<Key, Score>,
    scores: BTreeSet<SortedSetMember>,
}

impl SortedSet {
    /// Create a new SortedSet
    pub fn new() -> Self {
        SortedSet::default()
    }

    /// Add the following keys and scores to the sorted set
    pub fn add(&mut self, key_scores: RVec<(Score, Key)>) -> Count {
        key_scores
            .into_iter()
            .map(|(score, key)| match self.members_hash.entry(key) {
                Entry::Vacant(ent) => {
                    self.scores.insert(SortedSetMember::new(ent.key(), score));
                    ent.insert(score);
                    1
                }
                Entry::Occupied(_) => 0,
            })
            .sum()
    }

    /// Remove the following keys from the sorted set
    pub fn remove(&mut self, keys: &[Key]) -> Count {
        keys.iter()
            .map(|key| match self.members_hash.remove(key) {
                None => 0,
                Some(score) => {
                    let tmp = SortedSetMember::new(key, score);
                    self.scores.remove(&tmp);
                    1
                }
            })
            .sum()
    }
}