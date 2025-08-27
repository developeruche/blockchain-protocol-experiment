//! Clients module for pod-core crate.

use crate::{
    primitives::pod::{PodDS, PodTransaction, PodTransactionTrace, PodVote},
    utils::median,
};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct PodClient {
    pub replica_pks: HashMap<u64, String>,
    pub n: usize,
    pub beta: usize,
    pub gamma: usize,
    pub alpha: usize,
    pub mrt: HashMap<u64, u64>,
    pub next_sn: HashMap<u64, u64>,
    pub tsps: BTreeMap<PodTransaction, HashMap<u64, u64>>,
    pub c_pp: HashMap<u64, PodVote>,
    pub c_tx: BTreeMap<PodTransaction, HashMap<u64, PodVote>>,
}

impl PodClient {
    pub fn new(
        replica_pks: HashMap<u64, String>,
        n: usize,
        beta: usize,
        gamma: usize,
        alpha: usize,
    ) -> Self {
        let mrt = HashMap::new();
        let next_sn = HashMap::new();
        let tsps = BTreeMap::new();
        let c_pp = HashMap::new();
        let c_tx = BTreeMap::new();

        PodClient {
            replica_pks,
            n,
            beta,
            gamma,
            alpha,
            mrt,
            next_sn,
            tsps,
            c_pp,
            c_tx,
        }
    }

    pub fn process_vote(&mut self, vote: PodVote) -> bool {
        let expected_sn = self.next_sn.entry(vote.replica_id.clone()).or_insert(0);

        if vote.sn != *expected_sn {
            return false; // TODO: Implement backlog handling
        }
        *expected_sn += 1;

        let current_mrt = self.mrt.entry(vote.replica_id.clone()).or_insert(0);
        if vote.ts < *current_mrt {
            return false;
        }
        *current_mrt = vote.ts;

        let tx_timestamps = self.tsps.entry(vote.transaction.clone()).or_default();
        if let Some(existing_ts) = tx_timestamps.get(&vote.replica_id) {
            if *existing_ts != vote.ts {
                return false; // Equivocation
            }
        }

        tx_timestamps.insert(vote.replica_id.clone(), vote.ts);

        self.c_pp.insert(vote.replica_id.clone(), vote.clone());
        self.c_tx
            .entry(vote.transaction.clone())
            .or_default()
            .insert(vote.replica_id, vote);

        true
    }

    pub fn read(&self) -> PodDS {
        let t = self.compute_tx_set();
        let r_perf = self.compute_past_perfect_round();
        PodDS {
            tx_trace: t,
            r_perf,
        }
    }

    fn compute_past_perfect_round(&self) -> u64 {
        let mut mrt_values: Vec<u64> = self
            .replica_pks
            .keys()
            .map(|id| *self.mrt.get(id).unwrap_or(&0))
            .collect();

        let mut padded_mrt = vec![0; self.beta];
        padded_mrt.append(&mut mrt_values);
        padded_mrt.sort();

        median(&mut padded_mrt[..self.alpha])
    }

    fn max_possible_ts(&self, timestamps: &HashMap<u64, u64>) -> u64 {
        let mut all_ts: Vec<u64> = self
            .replica_pks
            .keys()
            .map(|id| *timestamps.get(id).unwrap_or(&u64::MAX))
            .collect();
        all_ts.sort();

        let mut padded_ts = all_ts;
        padded_ts.extend(vec![u64::MAX; self.beta]);

        median(&mut padded_ts[self.n - self.alpha..])
    }

    fn min_possible_ts(&self, timestamps: &HashMap<u64, u64>) -> u64 {
        let mut all_ts: Vec<u64> = self
            .replica_pks
            .keys()
            .map(|id| {
                *timestamps
                    .get(id)
                    .unwrap_or_else(|| self.mrt.get(id).unwrap_or(&0))
            })
            .collect();

        let mut padded_ts = vec![0; self.beta];
        padded_ts.append(&mut all_ts);
        padded_ts.sort();

        median(&mut padded_ts[..self.alpha])
    }

    fn compute_tx_set(&self) -> Vec<PodTransactionTrace> {
        let mut traces = Vec::new();
        for (tx, timestamps_map) in &self.tsps {
            let r_min = self.min_possible_ts(timestamps_map);
            let r_max = self.max_possible_ts(timestamps_map);
            let r_conf = if timestamps_map.len() >= self.alpha {
                let mut timestamps: Vec<u64> = timestamps_map.values().cloned().collect();
                Some(median(&mut timestamps))
            } else {
                None
            };
            traces.push(PodTransactionTrace {
                transaction: tx.clone(),
                r_min,
                r_max,
                r_conf,
            });
        }
        traces
    }
}
