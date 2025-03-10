use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use solana_program::{
    program::invoke_signed,
    sysvar::clock::Clock,
};
use crate::errors::DataChainError;

// Constants for governance system
pub const GOVERNANCE_VERSION: u8 = 1;
pub const VOTING_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days in seconds
pub const EXECUTION_DELAY: i64 = 2 * 24 * 60 * 60; // 2 days in seconds after voting ends
pub const QUORUM_PERCENTAGE: u8 = 10; // 10% of total supply must vote for proposal to be valid
pub const APPROVAL_THRESHOLD_PERCENTAGE: u8 = 60; // 60% of votes must be YES for approval
pub const MAX_PROPOSAL_TITLE_LENGTH: usize = 100;
pub const MAX_PROPOSAL_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_PROPOSAL_LINK_LENGTH: usize = 200;
pub const MAX_EXECUTION_PARAMS_SIZE: usize = 1000;

// Proposal types
pub enum ProposalType {
    UpdateFees = 0,
    UpgradeProgram = 1,
    AddFeature = 2,
    RemoveFeature = 3,
    FundProject = 4,
    Other = 5,
}

// Proposal status
pub enum ProposalStatus {
    Active = 0,
    Approved = 1,
    Rejected = 2,
    Executed = 3,
    Expired = 4,
}

// Governance proposal structure
#[account]
pub struct Proposal {
    // Proposal creator
    pub creator: Pubkey,
    
    // Proposal metadata
    pub title: String,
    pub description: String,
    pub proposal_type: u8,
    pub link: Option<String>,
    
    // Voting information
    pub yes_votes: u64,
    pub no_votes: u64,
    pub total_eligible_votes: u64,
    
    // Execution parameters (serialized)
    pub execution_params: Option<Vec<u8>>,
    
    // Time information
    pub created_at: i64,
    pub voting_ends_at: i64,
    pub executed_at: Option<i64>,
    
    // Status of the proposal
    pub status: u8,
    
    // Version for future upgrades
    pub version: u8,
}

// Vote record structure
#[account]
pub struct Vote {
    // The voter
    pub voter: Pubkey,
    
    // The proposal voted on
    pub proposal: Pubkey,
    
    // Vote value (true = yes, false = no)
    pub vote: bool,
    
    // Vote weight (depends on token balance)
    pub weight: u64,
    
    // Timestamp of vote
    pub voted_at: i64,
}

// Governance configuration
#[account]
pub struct GovernanceConfig {
    // Authority that can update governance parameters
    pub authority: Pubkey,
    
    // Governance token
    pub governance_token: Pubkey,
    
    // Voting parameters
    pub voting_period: i64,
    pub execution_delay: i64,
    pub quorum_percentage: u8,
    pub approval_threshold_percentage: u8,
    
    // Statistics
    pub total_proposals: u64,
    pub executed_proposals: u64,
    
    // Version for future upgrades
    pub version: u8,
}

// Context for creating a proposal
#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        space = 8 + std::mem::size_of::<Proposal>() + MAX_PROPOSAL_TITLE_LENGTH + MAX_PROPOSAL_DESCRIPTION_LENGTH + MAX_PROPOSAL_LINK_LENGTH + MAX_EXECUTION_PARAMS_SIZE,
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        constraint = creator_token.mint == governance_config.governance_token @ DataChainError::Unauthorized,
        constraint = creator_token.owner == creator.key() @ DataChainError::Unauthorized,
        constraint = creator_token.amount > 0 @ DataChainError::InsufficientVotingPower
    )]
    pub creator_token: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub governance_config: Account<'info, GovernanceConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Context for voting on a proposal
#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    
    #[account(
        mut,
        constraint = proposal.status == ProposalStatus::Active as u8 @ DataChainError::ProposalVotingEnded,
        constraint = proposal.voting_ends_at > clock.unix_timestamp @ DataChainError::ProposalVotingEnded
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        init,
        payer = voter,
        space = 8 + std::mem::size_of::<Vote>(),
    )]
    pub vote_record: Account<'info, Vote>,
    
    #[account(
        constraint = voter_token.mint == governance_config.governance_token @ DataChainError::Unauthorized,
        constraint = voter_token.owner == voter.key() @ DataChainError::Unauthorized,
        constraint = voter_token.amount > 0 @ DataChainError::InsufficientVotingPower
    )]
    pub voter_token: Account<'info, TokenAccount>,
    
    pub governance_config: Account<'info, GovernanceConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Context for executing a proposal
#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        constraint = proposal.status == ProposalStatus::Approved as u8 @ DataChainError::ProposalRejected,
        constraint = proposal.voting_ends_at + governance_config.execution_delay < clock.unix_timestamp @ DataChainError::InvalidTimestamp
    )]
    pub proposal: Account<'info, Proposal>,
    
    pub governance_config: Account<'info, GovernanceConfig>,
    
    // Note: Additional accounts required for execution would be passed based on proposal type
    
    pub clock: Sysvar<'info, Clock>,
}

// Implementation of governance functions
pub fn create_proposal(
    ctx: Context<CreateProposal>,
    title: String,
    description: String,
    proposal_type: u8,
    link: Option<String>,
    execution_params: Option<Vec<u8>>,
) -> Result<()> {
    // Validate input parameters
    if title.len() > MAX_PROPOSAL_TITLE_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if description.len() > MAX_PROPOSAL_DESCRIPTION_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if let Some(link_text) = &link {
        if link_text.len() > MAX_PROPOSAL_LINK_LENGTH {
            return err!(DataChainError::InvalidParameters);
        }
    }
    
    if let Some(params) = &execution_params {
        if params.len() > MAX_EXECUTION_PARAMS_SIZE {
            return err!(DataChainError::InvalidParameters);
        }
    }
    
    if proposal_type > ProposalType::Other as u8 {
        return err!(DataChainError::InvalidProposalType);
    }
    
    let proposal = &mut ctx.accounts.proposal;
    let governance_config = &mut ctx.accounts.governance_config;
    let clock = &ctx.accounts.clock;
    
    // Initialize proposal
    proposal.creator = ctx.accounts.creator.key();
    proposal.title = title;
    proposal.description = description;
    proposal.proposal_type = proposal_type;
    proposal.link = link;
    proposal.yes_votes = 0;
    proposal.no_votes = 0;
    proposal.total_eligible_votes = 10_000_000_000; // This should be fetched from token supply
    proposal.execution_params = execution_params;
    proposal.created_at = clock.unix_timestamp;
    proposal.voting_ends_at = clock.unix_timestamp + governance_config.voting_period;
    proposal.executed_at = None;
    proposal.status = ProposalStatus::Active as u8;
    proposal.version = GOVERNANCE_VERSION;
    
    // Update governance stats
    governance_config.total_proposals = governance_config.total_proposals.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("Proposal created: {}", proposal.title);
    
    Ok(())
}

pub fn vote(
    ctx: Context<Vote>,
    proposal_id: Pubkey,
    vote_value: bool,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let vote_record = &mut ctx.accounts.vote_record;
    let voter_token = &ctx.accounts.voter_token;
    let clock = &ctx.accounts.clock;
    
    // Check if the proposal matches provided ID
    if proposal.key() != proposal_id {
        return err!(DataChainError::ProposalNotFound);
    }
    
    // Get voter's token balance (voting weight)
    let vote_weight = voter_token.amount;
    
    // Record the vote
    vote_record.voter = ctx.accounts.voter.key();
    vote_record.proposal = proposal.key();
    vote_record.vote = vote_value;
    vote_record.weight = vote_weight;
    vote_record.voted_at = clock.unix_timestamp;
    
    // Update proposal vote counts
    if vote_value {
        proposal.yes_votes = proposal.yes_votes.checked_add(vote_weight).ok_or(DataChainError::InvalidParameters)?;
    } else {
        proposal.no_votes = proposal.no_votes.checked_add(vote_weight).ok_or(DataChainError::InvalidParameters)?;
    }
    
    msg!("Vote cast on proposal: {}", proposal.title);
    
    Ok(())
}

pub fn execute_proposal(
    ctx: Context<ExecuteProposal>,
    proposal_id: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let governance_config = &ctx.accounts.governance_config;
    let clock = &ctx.accounts.clock;
    
    // Check if the proposal matches provided ID
    if proposal.key() != proposal_id {
        return err!(DataChainError::ProposalNotFound);
    }
    
    // Check if quorum was reached
    let total_votes = proposal.yes_votes.checked_add(proposal.no_votes).ok_or(DataChainError::InvalidParameters)?;
    let quorum_threshold = (proposal.total_eligible_votes as u128)
        .checked_mul(governance_config.quorum_percentage as u128)
        .ok_or(DataChainError::InvalidParameters)?
        .checked_div(100)
        .ok_or(DataChainError::InvalidParameters)? as u64;
    
    if total_votes < quorum_threshold {
        return err!(DataChainError::QuorumNotReached);
    }
    
    // Check if proposal was approved
    let approval_threshold = (total_votes as u128)
        .checked_mul(governance_config.approval_threshold_percentage as u128)
        .ok_or(DataChainError::InvalidParameters)?
        .checked_div(100)
        .ok_or(DataChainError::InvalidParameters)? as u64;
    
    if proposal.yes_votes < approval_threshold {
        proposal.status = ProposalStatus::Rejected as u8;
        return err!(DataChainError::ProposalRejected);
    }
    
    // Execute the proposal logic based on proposal type
    // This would typically involve calling other functions or programs
    // based on the execution_params
    
    // For simplicity, we just mark it as executed
    proposal.status = ProposalStatus::Executed as u8;
    proposal.executed_at = Some(clock.unix_timestamp);
    
    // Update governance stats
    let governance_config = &mut ctx.accounts.governance_config;
    governance_config.executed_proposals = governance_config.executed_proposals.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("Proposal executed: {}", proposal.title);
    
    Ok(())
}

// Initialize governance configuration
pub fn initialize_governance(
    ctx: Context<InitializeGovernance>,
    voting_period: i64,
    execution_delay: i64,
    quorum_percentage: u8,
    approval_threshold_percentage: u8,
) -> Result<()> {
    // Validate parameters
    if quorum_percentage > 100 || approval_threshold_percentage > 100 {
        return err!(DataChainError::InvalidParameters);
    }
    
    if voting_period <= 0 || execution_delay <= 0 {
        return err!(DataChainError::InvalidParameters);
    }
    
    let governance_config = &mut ctx.accounts.governance_config;
    
    governance_config.authority = ctx.accounts.authority.key();
    governance_config.governance_token = ctx.accounts.governance_token.key();
    governance_config.voting_period = voting_period;
    governance_config.execution_delay = execution_delay;
    governance_config.quorum_percentage = quorum_percentage;
    governance_config.approval_threshold_percentage = approval_threshold_percentage;
    governance_config.total_proposals = 0;
    governance_config.executed_proposals = 0;
    governance_config.version = GOVERNANCE_VERSION;
    
    msg!("Governance system initialized");
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeGovernance<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub governance_token: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<GovernanceConfig>(),
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
} 