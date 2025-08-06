use crate::domain::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct GovernanceProposal {
    pub id: u64,
    pub proposal_type: ProposalType,
    pub model_id: ModelId,
    pub proposer: String,
    pub created_at: u64,
    pub voting_deadline: u64,
    pub description: String,
    pub votes: HashMap<String, Vote>,
    pub status: ProposalStatus,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ProposalType {
    ActivateModel,
    DeprecateModel,
    GrantBadge(BadgeType),
    RevokeBadge(BadgeType),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ProposalStatus {
    Open,
    Passed,
    Rejected,
    Executed,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct GovernanceConfig {
    pub voting_period_ns: u64,
    pub quorum_threshold: u32,      // Percentage (0-100)
    pub approval_threshold: u32,    // Percentage (0-100)
    pub authorized_voters: Vec<String>,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            voting_period_ns: 7 * 24 * 60 * 60 * 1_000_000_000, // 7 days in nanoseconds
            quorum_threshold: 33, // 33% quorum
            approval_threshold: 66, // 66% approval
            authorized_voters: Vec::new(),
        }
    }
}

pub struct GovernanceEngine {
    proposals: HashMap<u64, GovernanceProposal>,
    next_proposal_id: u64,
    config: GovernanceConfig,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            next_proposal_id: 1,
            config: GovernanceConfig::default(),
        }
    }

    pub fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        model_id: ModelId,
        proposer: String,
        description: String,
        current_time: u64,
    ) -> Result<u64, String> {
        if !self.config.authorized_voters.contains(&proposer) {
            return Err("Proposer not authorized".to_string());
        }

        let proposal = GovernanceProposal {
            id: self.next_proposal_id,
            proposal_type,
            model_id,
            proposer,
            created_at: current_time,
            voting_deadline: current_time + self.config.voting_period_ns,
            description,
            votes: HashMap::new(),
            status: ProposalStatus::Open,
        };

        let proposal_id = self.next_proposal_id;
        self.proposals.insert(proposal_id, proposal);
        self.next_proposal_id += 1;

        Ok(proposal_id)
    }

    pub fn cast_vote(
        &mut self,
        proposal_id: u64,
        voter: String,
        vote: Vote,
        current_time: u64,
    ) -> Result<(), String> {
        if !self.config.authorized_voters.contains(&voter) {
            return Err("Voter not authorized".to_string());
        }

        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if current_time > proposal.voting_deadline {
            return Err("Voting period has ended".to_string());
        }

        if !matches!(proposal.status, ProposalStatus::Open) {
            return Err("Proposal is not open for voting".to_string());
        }

        proposal.votes.insert(voter, vote);
        Ok(())
    }

    pub fn tally_votes(&mut self, proposal_id: u64, current_time: u64) -> Result<ProposalStatus, String> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if current_time <= proposal.voting_deadline {
            return Err("Voting period not yet ended".to_string());
        }

        let total_voters = self.config.authorized_voters.len() as u32;
        let total_votes = proposal.votes.len() as u32;
        let yes_votes = proposal.votes.values().filter(|v| matches!(v, Vote::Yes)).count() as u32;

        // Check quorum
        let quorum_met = (total_votes * 100) >= (total_voters * self.config.quorum_threshold);
        
        if !quorum_met {
            proposal.status = ProposalStatus::Rejected;
            return Ok(ProposalStatus::Rejected);
        }

        // Check approval threshold
        let approval_met = (yes_votes * 100) >= (total_votes * self.config.approval_threshold);
        
        if approval_met {
            proposal.status = ProposalStatus::Passed;
            Ok(ProposalStatus::Passed)
        } else {
            proposal.status = ProposalStatus::Rejected;
            Ok(ProposalStatus::Rejected)
        }
    }

    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<(), String> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if !matches!(proposal.status, ProposalStatus::Passed) {
            return Err("Proposal must be in Passed state to execute".to_string());
        }

        proposal.status = ProposalStatus::Executed;
        Ok(())
    }

    pub fn get_proposal(&self, proposal_id: u64) -> Option<&GovernanceProposal> {
        self.proposals.get(&proposal_id)
    }

    pub fn list_proposals(&self) -> Vec<&GovernanceProposal> {
        self.proposals.values().collect()
    }

    pub fn add_authorized_voter(&mut self, voter: String) {
        if !self.config.authorized_voters.contains(&voter) {
            self.config.authorized_voters.push(voter);
        }
    }
}