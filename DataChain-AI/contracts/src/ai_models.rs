use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solana_program::{
    program::invoke_signed,
    sysvar::clock::Clock,
};
use crate::errors::DataChainError;

// Constants for AI model marketplace
pub const MODEL_VERSION: u8 = 1;
pub const PLATFORM_FEE_PERCENTAGE: u8 = 15; // 15% platform fee for AI models
pub const RATING_MIN: u8 = 1;
pub const RATING_MAX: u8 = 5;
pub const MAX_MODEL_NAME_LENGTH: usize = 50;
pub const MAX_MODEL_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_MODEL_TYPE_LENGTH: usize = 50;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_QUERY_PARAMS_LENGTH: usize = 1000;
pub const MAX_REVIEW_LENGTH: usize = 500;

// AI Model structure
#[account]
pub struct AiModel {
    // Model owner
    pub owner: Pubkey,
    
    // Model metadata
    pub name: String,
    pub description: String,
    pub model_type: String,
    
    // Model access info
    pub price_per_query: u64,
    pub uri: String,
    
    // Model statistics
    pub usages: u64,
    pub rating_sum: u64,
    pub rating_count: u64,
    
    // Revenue info
    pub total_revenue: u64,
    
    // Creation timestamp
    pub created_at: i64,
    pub updated_at: i64,
    
    // Version for future upgrades
    pub version: u8,
}

// Usage record structure
#[account]
pub struct ModelUsage {
    // The user
    pub user: Pubkey,
    
    // The model used
    pub model: Pubkey,
    
    // Timestamp of usage
    pub used_at: i64,
    
    // Has the user rated this model?
    pub has_rated: bool,
    
    // Query details
    pub query_params: String,
    
    // Transaction details
    pub price_paid: u64,
}

// Rating structure
#[account]
pub struct ModelRating {
    // The rater
    pub rater: Pubkey,
    
    // The model rated
    pub model: Pubkey,
    
    // Rating details
    pub rating: u8,
    pub review: Option<String>,
    
    // Timestamp of rating
    pub rated_at: i64,
}

// Model marketplace configuration
#[account]
pub struct ModelMarketplaceConfig {
    // Authority that can update marketplace parameters
    pub authority: Pubkey,
    
    // Fee configuration
    pub platform_fee_percentage: u8,
    pub platform_fee_recipient: Pubkey,
    
    // Statistics
    pub total_models: u64,
    pub total_usages: u64,
    pub total_volume: u64,
    
    // Version for future upgrades
    pub version: u8,
}

// Context for registering an AI model
#[derive(Accounts)]
pub struct RegisterModel<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<AiModel>() + MAX_MODEL_NAME_LENGTH + MAX_MODEL_DESCRIPTION_LENGTH + MAX_MODEL_TYPE_LENGTH + MAX_URI_LENGTH,
    )]
    pub model: Account<'info, AiModel>,
    
    #[account(mut)]
    pub marketplace_config: Account<'info, ModelMarketplaceConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Context for using an AI model
#[derive(Accounts)]
pub struct UseModel<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub model: Account<'info, AiModel>,
    
    #[account(mut)]
    pub model_owner: AccountInfo<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<ModelUsage>() + MAX_QUERY_PARAMS_LENGTH,
    )]
    pub usage_record: Account<'info, ModelUsage>,
    
    #[account(
        mut,
        constraint = user_token.owner == user.key() @ DataChainError::Unauthorized
    )]
    pub user_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = owner_token.owner == model_owner.key() @ DataChainError::Unauthorized
    )]
    pub owner_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = platform_fee_token.owner == marketplace_config.platform_fee_recipient @ DataChainError::Unauthorized
    )]
    pub platform_fee_token: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub marketplace_config: Account<'info, ModelMarketplaceConfig>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Context for rating an AI model
#[derive(Accounts)]
pub struct RateModel<'info> {
    #[account(mut)]
    pub rater: Signer<'info>,
    
    #[account(mut)]
    pub model: Account<'info, AiModel>,
    
    #[account(
        mut,
        constraint = usage_record.user == rater.key() @ DataChainError::Unauthorized,
        constraint = usage_record.model == model.key() @ DataChainError::ModelNotFound,
        constraint = !usage_record.has_rated @ DataChainError::AlreadyRated
    )]
    pub usage_record: Account<'info, ModelUsage>,
    
    #[account(
        init,
        payer = rater,
        space = 8 + std::mem::size_of::<ModelRating>() + MAX_REVIEW_LENGTH,
    )]
    pub rating: Account<'info, ModelRating>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Implementation of AI model marketplace functions
pub fn register_model(
    ctx: Context<RegisterModel>,
    name: String,
    description: String,
    model_type: String,
    price_per_query: u64,
    uri: String,
) -> Result<()> {
    // Validate input parameters
    if name.len() > MAX_MODEL_NAME_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if description.len() > MAX_MODEL_DESCRIPTION_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if model_type.len() > MAX_MODEL_TYPE_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if uri.len() > MAX_URI_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if price_per_query == 0 {
        return err!(DataChainError::InvalidModelPrice);
    }
    
    let model = &mut ctx.accounts.model;
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    let clock = &ctx.accounts.clock;
    
    // Initialize model
    model.owner = ctx.accounts.owner.key();
    model.name = name;
    model.description = description;
    model.model_type = model_type;
    model.price_per_query = price_per_query;
    model.uri = uri;
    model.usages = 0;
    model.rating_sum = 0;
    model.rating_count = 0;
    model.total_revenue = 0;
    model.created_at = clock.unix_timestamp;
    model.updated_at = clock.unix_timestamp;
    model.version = MODEL_VERSION;
    
    // Update marketplace stats
    marketplace_config.total_models = marketplace_config.total_models.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("AI Model registered: {}", model.name);
    
    Ok(())
}

pub fn use_model(
    ctx: Context<UseModel>,
    model_id: Pubkey,
    query_params: String,
) -> Result<()> {
    // Validate query params
    if query_params.len() > MAX_QUERY_PARAMS_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    let model = &mut ctx.accounts.model;
    let user = &ctx.accounts.user;
    let usage_record = &mut ctx.accounts.usage_record;
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    let clock = &ctx.accounts.clock;
    
    // Check if the model exists and matches provided ID
    if model.key() != model_id {
        return err!(DataChainError::ModelNotFound);
    }
    
    // Calculate fees
    let usage_amount = model.price_per_query;
    let platform_fee = (usage_amount as u128)
        .checked_mul(marketplace_config.platform_fee_percentage as u128)
        .ok_or(DataChainError::InvalidParameters)?
        .checked_div(100)
        .ok_or(DataChainError::InvalidParameters)? as u64;
    
    let owner_amount = usage_amount.checked_sub(platform_fee).ok_or(DataChainError::InvalidParameters)?;
    
    // Transfer platform fee
    {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.platform_fee_token.to_account_info(),
            authority: user.to_account_info(),
        };
        
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
            ),
            platform_fee,
        )?;
    }
    
    // Transfer payment to model owner
    {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.owner_token.to_account_info(),
            authority: user.to_account_info(),
        };
        
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
            ),
            owner_amount,
        )?;
    }
    
    // Create usage record
    usage_record.user = user.key();
    usage_record.model = model.key();
    usage_record.used_at = clock.unix_timestamp;
    usage_record.has_rated = false;
    usage_record.query_params = query_params;
    usage_record.price_paid = usage_amount;
    
    // Update model stats
    model.usages = model.usages.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    model.total_revenue = model.total_revenue.checked_add(usage_amount).ok_or(DataChainError::InvalidParameters)?;
    
    // Update marketplace stats
    marketplace_config.total_usages = marketplace_config.total_usages.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    marketplace_config.total_volume = marketplace_config.total_volume.checked_add(usage_amount).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("AI Model used: {} by {}", model.name, user.key());
    
    Ok(())
}

pub fn rate_model(
    ctx: Context<RateModel>,
    model_id: Pubkey,
    rating_value: u8,
    review: Option<String>,
) -> Result<()> {
    if rating_value < RATING_MIN || rating_value > RATING_MAX {
        return err!(DataChainError::InvalidRating);
    }
    
    if let Some(review_text) = &review {
        if review_text.len() > MAX_REVIEW_LENGTH {
            return err!(DataChainError::InvalidParameters);
        }
    }
    
    let model = &mut ctx.accounts.model;
    let usage_record = &mut ctx.accounts.usage_record;
    let rating = &mut ctx.accounts.rating;
    let clock = &ctx.accounts.clock;
    
    // Check if the model matches provided ID
    if model.key() != model_id {
        return err!(DataChainError::ModelNotFound);
    }
    
    // Create rating record
    rating.rater = ctx.accounts.rater.key();
    rating.model = model.key();
    rating.rating = rating_value;
    rating.review = review;
    rating.rated_at = clock.unix_timestamp;
    
    // Update usage record
    usage_record.has_rated = true;
    
    // Update model stats
    model.rating_sum = model.rating_sum.checked_add(rating_value as u64).ok_or(DataChainError::InvalidParameters)?;
    model.rating_count = model.rating_count.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("AI Model rated: {} with rating {}", model.name, rating_value);
    
    Ok(())
}

// Initialize model marketplace configuration
pub fn initialize_model_marketplace(
    ctx: Context<InitializeModelMarketplace>,
    platform_fee_percentage: u8,
) -> Result<()> {
    if platform_fee_percentage > 100 {
        return err!(DataChainError::InvalidParameters);
    }
    
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    
    marketplace_config.authority = ctx.accounts.authority.key();
    marketplace_config.platform_fee_percentage = platform_fee_percentage;
    marketplace_config.platform_fee_recipient = ctx.accounts.platform_fee_recipient.key();
    marketplace_config.total_models = 0;
    marketplace_config.total_usages = 0;
    marketplace_config.total_volume = 0;
    marketplace_config.version = MODEL_VERSION;
    
    msg!("AI Model marketplace initialized with {}% fee", platform_fee_percentage);
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeModelMarketplace<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub platform_fee_recipient: AccountInfo<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<ModelMarketplaceConfig>(),
    )]
    pub marketplace_config: Account<'info, ModelMarketplaceConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
} 