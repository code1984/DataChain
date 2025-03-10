use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Import project modules
pub mod errors;
pub mod data_token;
pub mod data_marketplace;
pub mod ai_models;
pub mod governance;

// Re-export key components
pub use errors::*;
pub use data_token::*;
pub use data_marketplace::*;
pub use ai_models::*;
pub use governance::*;

declare_id!("DATAnKVv5pzRz2DMyNwFiZCsZbM4QSFTUVeD5uBZ9Bs");

// Program entry point
#[program]
pub mod datachain_ai {
    use super::*;
    
    // DATA Token Management
    
    /// Initialize the DATA token
    pub fn initialize_token(
        ctx: Context<InitializeToken>,
        name: String,
        symbol: String,
        uri: String,
        decimals: u8,
    ) -> Result<()> {
        data_token::initialize_token(ctx, name, symbol, uri, decimals)
    }
    
    /// Mint DATA tokens to a specified account
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        data_token::mint_tokens(ctx, amount)
    }
    
    /// Transfer DATA tokens with memo
    pub fn transfer_tokens(
        ctx: Context<TransferTokens>,
        amount: u64,
        memo: Option<String>,
    ) -> Result<()> {
        data_token::transfer_tokens(ctx, amount, memo)
    }
    
    // Data Marketplace
    
    /// Register a new dataset on the marketplace
    pub fn register_dataset(
        ctx: Context<RegisterDataset>,
        name: String,
        description: String,
        data_type: String,
        price: u64,
        uri: String,
        preview_uri: String,
    ) -> Result<()> {
        data_marketplace::register_dataset(ctx, name, description, data_type, price, uri, preview_uri)
    }
    
    /// Purchase access to a dataset
    pub fn purchase_dataset(
        ctx: Context<PurchaseDataset>,
        dataset_id: Pubkey,
    ) -> Result<()> {
        data_marketplace::purchase_dataset(ctx, dataset_id)
    }
    
    /// Rate a dataset after purchase
    pub fn rate_dataset(
        ctx: Context<RateDataset>,
        dataset_id: Pubkey,
        rating: u8,
        review: Option<String>,
    ) -> Result<()> {
        data_marketplace::rate_dataset(ctx, dataset_id, rating, review)
    }
    
    // AI Model Marketplace
    
    /// Register a new AI model
    pub fn register_model(
        ctx: Context<RegisterModel>,
        name: String,
        description: String,
        model_type: String,
        price_per_query: u64,
        uri: String,
    ) -> Result<()> {
        ai_models::register_model(ctx, name, description, model_type, price_per_query, uri)
    }
    
    /// Use an AI model for analysis
    pub fn use_model(
        ctx: Context<UseModel>,
        model_id: Pubkey,
        query_params: String,
    ) -> Result<()> {
        ai_models::use_model(ctx, model_id, query_params)
    }
    
    /// Rate an AI model after usage
    pub fn rate_model(
        ctx: Context<RateModel>,
        model_id: Pubkey,
        rating: u8,
        review: Option<String>,
    ) -> Result<()> {
        ai_models::rate_model(ctx, model_id, rating, review)
    }
    
    // Governance
    
    /// Create a governance proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        proposal_type: u8,
        link: Option<String>,
        execution_params: Option<Vec<u8>>,
    ) -> Result<()> {
        governance::create_proposal(ctx, title, description, proposal_type, link, execution_params)
    }
    
    /// Vote on a governance proposal
    pub fn vote(
        ctx: Context<Vote>,
        proposal_id: Pubkey,
        vote: bool,
    ) -> Result<()> {
        governance::vote(ctx, proposal_id, vote)
    }
    
    /// Execute an approved proposal
    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
        proposal_id: Pubkey,
    ) -> Result<()> {
        governance::execute_proposal(ctx, proposal_id)
    }
}

// Context structs for instruction validation

#[derive(Accounts)]
pub struct InitializeToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = authority.key(),
    )]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint = mint.mint_authority.unwrap() == authority.key()
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = recipient.mint == mint.key()
    )]
    pub recipient: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    pub sender: Signer<'info>,
    #[account(
        mut,
        constraint = sender_token.owner == sender.key()
    )]
    pub sender_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub recipient_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// Other account validation structs would be defined here
// The implementation details will be in their respective module files 