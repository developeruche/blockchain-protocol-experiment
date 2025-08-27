//! Primitives module for pod-core replica.

use crate::{
    primitives::{
        errors::PodError,
        pod::{PodTransaction, PodVote},
    },
    utils::sign_tx,
};
use alloy::{
    primitives::B256,
    signers::{Signature, k256::ecdsa::SigningKey, local::LocalSigner},
};
use chrono::Utc;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Replica {
    pub id: u64,
    pub signer: LocalSigner<SigningKey>,
    pub connected_clients: Vec<String>, // This should be a vec of client ip address... maybe
    pub next_sn: u64,
    pub replica_log: Vec<PodVote>,
    pub tx_entry: HashSet<B256>,
}

impl Replica {
    pub fn new(id: u64, signer: LocalSigner<SigningKey>) -> Self {
        Self {
            id,
            signer,
            connected_clients: Vec::new(),
            next_sn: 0,
            replica_log: Vec::new(),
            tx_entry: HashSet::new(),
        }
    }

    pub fn sign_tx(&self, tx: &PodTransaction) -> Result<Signature, PodError> {
        sign_tx(tx, &self.signer)
    }

    pub fn on_connect(&mut self, client_ip: String) -> Vec<PodVote> {
        self.connected_clients.push(client_ip);
        self.replica_log.clone()
    }

    pub fn on_write(&mut self, vote: &PodTransaction) -> Result<Option<PodVote>, PodError> {
        if self.tx_entry.contains(&vote.hash()) {
            return Ok(None);
        }
        self.do_vote(vote)
    }

    pub fn do_vote(&mut self, tx: &PodTransaction) -> Result<Option<PodVote>, PodError> {
        if self.tx_entry.contains(&tx.hash()) {
            return Ok(None);
        }
        let ts = self.get_current_round();
        let sn = self.next_sn;

        let signature = self.sign_tx(tx)?;
        let vote = PodVote {
            transaction: tx.clone(),
            ts,
            sn,
            replica_id: self.id,
            signature,
        };

        self.replica_log.push(vote.clone());

        Ok(Some(vote))
    }

    pub fn on_end_of_round(&mut self) -> Result<Option<PodVote>, PodError> {
        let tx = PodTransaction::heartbeat_tx();
        self.do_vote(&tx)
    }

    pub fn get_current_round(&self) -> u64 {
        Utc::now().timestamp() as u64
    }
}
