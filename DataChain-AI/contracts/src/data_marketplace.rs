use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solana_program::{
    program::invoke_signed,
    sysvar::clock::Clock,
};
use crate::errors::DataChainError;

// Constants for the data marketplace
pub const PLATFORM_FEE_PERCENTAGE: u8 = 2; // 2% platform fee
pub const RATING_MIN: u8 = 1;
pub const RATING_MAX: u8 = 5;
pub const MARKETPLACE_VERSION: u8 = 1;
pub const MAX_DATASET_NAME_LENGTH: usize = 50;
pub const MAX_DATASET_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_DATA_TYPE_LENGTH: usize = 50;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_REVIEW_LENGTH: usize = 500;

// Dataset structure
#[account]
pub struct Dataset {
    // Dataset owner
    pub owner: Pubkey,
    
    // Dataset metadata
    pub name: String,
    pub description: String,
    pub data_type: String,
    
    // Dataset access info
    pub price: u64,
    pub uri: String,
    pub preview_uri: String,
    
    // Dataset statistics
    pub purchases: u64,
    pub rating_sum: u64,
    pub rating_count: u64,
    
    // Creation timestamp
    pub created_at: i64,
    pub updated_at: i64,
    
    // Version for future upgrades
    pub version: u8,
}

// Purchase record structure
#[account]
pub struct PurchaseRecord {
    // The buyer
    pub buyer: Pubkey,
    
    // The dataset purchased
    pub dataset: Pubkey,
    
    // Timestamp of purchase
    pub purchased_at: i64,
    
    // Has the buyer rated this dataset?
    pub has_rated: bool,
    
    // Transaction details
    pub price_paid: u64,
}

// Rating structure
#[account]
pub struct DatasetRating {
    // The rater
    pub rater: Pubkey,
    
    // The dataset rated
    pub dataset: Pubkey,
    
    // Rating details
    pub rating: u8,
    pub review: Option<String>,
    
    // Timestamp of rating
    pub rated_at: i64,
}

// Marketplace configuration
#[account]
pub struct MarketplaceConfig {
    // Authority that can update marketplace parameters
    pub authority: Pubkey,
    
    // Fee configuration
    pub platform_fee_percentage: u8,
    pub platform_fee_recipient: Pubkey,
    
    // Statistics
    pub total_datasets: u64,
    pub total_purchases: u64,
    pub total_volume: u64,
    
    // Version for future upgrades
    pub version: u8,
}

// Context for registering a dataset
#[derive(Accounts)]
pub struct RegisterDataset<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<Dataset>() + MAX_DATASET_NAME_LENGTH + MAX_DATASET_DESCRIPTION_LENGTH + MAX_DATA_TYPE_LENGTH + (2 * MAX_URI_LENGTH),
    )]
    pub dataset: Account<'info, Dataset>,
    
    #[account(mut)]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Context for purchasing a dataset
#[derive(Accounts)]
pub struct PurchaseDataset<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        constraint = dataset.owner != buyer.key() @ DataChainError::Unauthorized
    )]
    pub dataset: Account<'info, Dataset>,
    
    #[account(mut)]
    pub dataset_owner: AccountInfo<'info>,
    
    #[account(
        init,
        payer = buyer,
        space = 8 + std::mem::size_of::<PurchaseRecord>(),
    )]
    pub purchase_record: Account<'info, PurchaseRecord>,
    
    #[account(
        mut,
        constraint = buyer_token.owner == buyer.key() @ DataChainError::Unauthorized
    )]
    pub buyer_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = owner_token.owner == dataset_owner.key() @ DataChainError::Unauthorized
    )]
    pub owner_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = platform_fee_token.owner == marketplace_config.platform_fee_recipient @ DataChainError::Unauthorized
    )]
    pub platform_fee_token: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Context for rating a dataset
#[derive(Accounts)]
pub struct RateDataset<'info> {
    #[account(mut)]
    pub rater: Signer<'info>,
    
    #[account(mut)]
    pub dataset: Account<'info, Dataset>,
    
    #[account(
        mut,
        constraint = purchase_record.buyer == rater.key() @ DataChainError::Unauthorized,
        constraint = purchase_record.dataset == dataset.key() @ DataChainError::AccessNotPurchased,
        constraint = !purchase_record.has_rated @ DataChainError::AlreadyRated
    )]
    pub purchase_record: Account<'info, PurchaseRecord>,
    
    #[account(
        init,
        payer = rater,
        space = 8 + std::mem::size_of::<DatasetRating>() + MAX_REVIEW_LENGTH,
    )]
    pub rating: Account<'info, DatasetRating>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// Implementation of marketplace functions
pub fn register_dataset(
    ctx: Context<RegisterDataset>,
    name: String,
    description: String,
    data_type: String,
    price: u64,
    uri: String,
    preview_uri: String,
) -> Result<()> {
    // Validate input parameters
    if name.len() > MAX_DATASET_NAME_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if description.len() > MAX_DATASET_DESCRIPTION_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if data_type.len() > MAX_DATA_TYPE_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if uri.len() > MAX_URI_LENGTH || preview_uri.len() > MAX_URI_LENGTH {
        return err!(DataChainError::InvalidParameters);
    }
    
    if price == 0 {
        return err!(DataChainError::InvalidDatasetPrice);
    }
    
    let dataset = &mut ctx.accounts.dataset;
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    let clock = &ctx.accounts.clock;
    
    // Initialize dataset
    dataset.owner = ctx.accounts.owner.key();
    dataset.name = name;
    dataset.description = description;
    dataset.data_type = data_type;
    dataset.price = price;
    dataset.uri = uri;
    dataset.preview_uri = preview_uri;
    dataset.purchases = 0;
    dataset.rating_sum = 0;
    dataset.rating_count = 0;
    dataset.created_at = clock.unix_timestamp;
    dataset.updated_at = clock.unix_timestamp;
    dataset.version = MARKETPLACE_VERSION;
    
    // Update marketplace stats
    marketplace_config.total_datasets = marketplace_config.total_datasets.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("Dataset registered: {}", dataset.name);
    
    Ok(())
}

pub fn purchase_dataset(
    ctx: Context<PurchaseDataset>,
    dataset_id: Pubkey,
) -> Result<()> {
    let dataset = &mut ctx.accounts.dataset;
    let buyer = &ctx.accounts.buyer;
    let purchase_record = &mut ctx.accounts.purchase_record;
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    let clock = &ctx.accounts.clock;
    
    // Check if the dataset exists and matches provided ID
    if dataset.key() != dataset_id {
        return err!(DataChainError::DatasetNotFound);
    }
    
    // Calculate fees
    let purchase_amount = dataset.price;
    let platform_fee = (purchase_amount as u128)
        .checked_mul(marketplace_config.platform_fee_percentage as u128)
        .ok_or(DataChainError::InvalidParameters)?
        .checked_div(100)
        .ok_or(DataChainError::InvalidParameters)? as u64;
    
    let seller_amount = purchase_amount.checked_sub(platform_fee).ok_or(DataChainError::InvalidParameters)?;
    
    // Transfer platform fee
    {
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_token.to_account_info(),
            to: ctx.accounts.platform_fee_token.to_account_info(),
            authority: buyer.to_account_info(),
        };
        
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
            ),
            platform_fee,
        )?;
    }
    
    // Transfer payment to dataset owner
    {
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_token.to_account_info(),
            to: ctx.accounts.owner_token.to_account_info(),
            authority: buyer.to_account_info(),
        };
        
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
            ),
            seller_amount,
        )?;
    }
    
    // Create purchase record
    purchase_record.buyer = buyer.key();
    purchase_record.dataset = dataset.key();
    purchase_record.purchased_at = clock.unix_timestamp;
    purchase_record.has_rated = false;
    purchase_record.price_paid = purchase_amount;
    
    // Update dataset stats
    dataset.purchases = dataset.purchases.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    
    // Update marketplace stats
    marketplace_config.total_purchases = marketplace_config.total_purchases.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    marketplace_config.total_volume = marketplace_config.total_volume.checked_add(purchase_amount).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("Dataset purchased: {} by {}", dataset.name, buyer.key());
    
    Ok(())
}

pub fn rate_dataset(
    ctx: Context<RateDataset>,
    dataset_id: Pubkey,
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
    
    let dataset = &mut ctx.accounts.dataset;
    let purchase_record = &mut ctx.accounts.purchase_record;
    let rating = &mut ctx.accounts.rating;
    let clock = &ctx.accounts.clock;
    
    // Check if the dataset matches provided ID
    if dataset.key() != dataset_id {
        return err!(DataChainError::DatasetNotFound);
    }
    
    // Create rating record
    rating.rater = ctx.accounts.rater.key();
    rating.dataset = dataset.key();
    rating.rating = rating_value;
    rating.review = review;
    rating.rated_at = clock.unix_timestamp;
    
    // Update purchase record
    purchase_record.has_rated = true;
    
    // Update dataset stats
    dataset.rating_sum = dataset.rating_sum.checked_add(rating_value as u64).ok_or(DataChainError::InvalidParameters)?;
    dataset.rating_count = dataset.rating_count.checked_add(1).ok_or(DataChainError::InvalidParameters)?;
    
    msg!("Dataset rated: {} with rating {}", dataset.name, rating_value);
    
    Ok(())
}

// Initialize marketplace configuration
pub fn initialize_marketplace(
    ctx: Context<InitializeMarketplace>,
    platform_fee_percentage: u8,
) -> Result<()> {
    if platform_fee_percentage > 100 {
        return err!(DataChainError::InvalidParameters);
    }
    
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    
    marketplace_config.authority = ctx.accounts.authority.key();
    marketplace_config.platform_fee_percentage = platform_fee_percentage;
    marketplace_config.platform_fee_recipient = ctx.accounts.platform_fee_recipient.key();
    marketplace_config.total_datasets = 0;
    marketplace_config.total_purchases = 0;
    marketplace_config.total_volume = 0;
    marketplace_config.version = MARKETPLACE_VERSION;
    
    msg!("Marketplace initialized with {}% fee", platform_fee_percentage);
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub platform_fee_recipient: AccountInfo<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<MarketplaceConfig>(),
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
} 