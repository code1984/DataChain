use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Transfer};
use solana_program::{
    program::invoke_signed,
    sysvar::clock::Clock,
};
use crate::errors::DataChainError;

// Token configuration
pub const TOKEN_DECIMALS: u8 = 9;
pub const TOTAL_SUPPLY: u64 = 100_000_000_000_000_000; // 100 million tokens with 9 decimals
pub const MINT_COOLDOWN: i64 = 3600; // Minting cooldown period in seconds (1 hour)
pub const AUTHORITY_TRANSFER_EXPIRY: i64 = 86400; // Authority transfer validity period in seconds (24 hours)

// Token data structure
#[account]
pub struct TokenInfo {
    // Token authority (e.g., mint authority)
    pub authority: Pubkey,
    
    // Token mint address
    pub mint: Pubkey,
    
    // Token metadata
    pub name: String,
    pub symbol: String,
    pub uri: String,
    
    // Token supply info
    pub current_supply: u64,
    pub max_supply: u64,
    
    // Last mint timestamp for cooldown
    pub last_mint_timestamp: i64,
    
    // Authority transfer info
    pub pending_authority: Option<Pubkey>,
    pub authority_transfer_expiry: i64,
    
    // Token parameters
    pub decimals: u8,
    
    // Token version for future upgrades
    pub version: u8,
}

// Token authority transfer request
#[account]
pub struct AuthorityTransferRequest {
    // Current authority
    pub current_authority: Pubkey,
    
    // New authority to transfer to
    pub new_authority: Pubkey,
    
    // Expiration timestamp
    pub expiry_timestamp: i64,
}

// Context for authority transfer request
#[derive(Accounts)]
pub struct RequestAuthorityTransfer<'info> {
    #[account(mut)]
    pub current_authority: Signer<'info>,
    
    pub new_authority: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = token_info.authority == current_authority.key() @ DataChainError::Unauthorized
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

// Context for authority transfer acceptance
#[derive(Accounts)]
pub struct AcceptAuthorityTransfer<'info> {
    #[account(mut)]
    pub new_authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = token_info.pending_authority == Some(new_authority.key()) @ DataChainError::Unauthorized,
        constraint = token_info.authority_transfer_expiry > clock.unix_timestamp @ DataChainError::InvalidTimestamp
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    pub clock: Sysvar<'info, Clock>,
}

// Implementation of token functions
pub fn initialize_token(
    ctx: Context<InitializeToken>,
    name: String,
    symbol: String,
    uri: String,
    decimals: u8,
) -> Result<()> {
    let token_info = &mut ctx.accounts.token_info;
    let authority = &ctx.accounts.authority;
    let mint = &ctx.accounts.mint;
    let clock = &ctx.accounts.clock;
    
    // Initialize token info
    token_info.authority = authority.key();
    token_info.mint = mint.key();
    token_info.name = name;
    token_info.symbol = symbol;
    token_info.uri = uri;
    token_info.current_supply = 0;
    token_info.max_supply = TOTAL_SUPPLY;
    token_info.last_mint_timestamp = clock.unix_timestamp;
    token_info.pending_authority = None;
    token_info.authority_transfer_expiry = 0;
    token_info.decimals = decimals;
    token_info.version = 1;
    
    msg!("DATA token initialized successfully");
    
    Ok(())
}

pub fn mint_tokens(
    ctx: Context<MintTokens>,
    amount: u64,
) -> Result<()> {
    let token_info = &mut ctx.accounts.token_info;
    let mint = &ctx.accounts.mint;
    let authority = &ctx.accounts.authority;
    let recipient = &ctx.accounts.recipient;
    let token_program = &ctx.accounts.token_program;
    let clock = &ctx.accounts.clock;
    
    // Check if minting cooldown period has passed
    if clock.unix_timestamp - token_info.last_mint_timestamp < MINT_COOLDOWN {
        return err!(DataChainError::MintingCooldown);
    }
    
    // Check if max supply would be exceeded
    if token_info.current_supply.checked_add(amount).ok_or(DataChainError::InvalidParameters)? > token_info.max_supply {
        return err!(DataChainError::MaxSupplyReached);
    }
    
    // Create mint instruction
    let cpi_accounts = MintTo {
        mint: mint.to_account_info(),
        to: recipient.to_account_info(),
        authority: authority.to_account_info(),
    };
    
    // Execute mint instruction
    token::mint_to(
        CpiContext::new(
            token_program.to_account_info(),
            cpi_accounts,
        ),
        amount,
    )?;
    
    // Update token info
    token_info.current_supply = token_info.current_supply.checked_add(amount).ok_or(DataChainError::InvalidParameters)?;
    token_info.last_mint_timestamp = clock.unix_timestamp;
    
    msg!("Minted {} DATA tokens to {}", amount, recipient.key());
    
    Ok(())
}

pub fn transfer_tokens(
    ctx: Context<TransferTokens>,
    amount: u64,
    memo: Option<String>,
) -> Result<()> {
    let sender_token = &ctx.accounts.sender_token;
    let recipient_token = &ctx.accounts.recipient_token;
    let token_program = &ctx.accounts.token_program;
    
    // Create transfer instruction
    let cpi_accounts = Transfer {
        from: sender_token.to_account_info(),
        to: recipient_token.to_account_info(),
        authority: ctx.accounts.sender.to_account_info(),
    };
    
    // Execute transfer instruction
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            cpi_accounts,
        ),
        amount,
    )?;
    
    // Log memo if provided
    if let Some(memo_text) = memo {
        msg!("Memo: {}", memo_text);
    }
    
    msg!("Transferred {} DATA tokens from {} to {}", 
         amount, 
         sender_token.owner, 
         recipient_token.owner);
    
    Ok(())
}

pub fn request_authority_transfer(
    ctx: Context<RequestAuthorityTransfer>,
) -> Result<()> {
    let token_info = &mut ctx.accounts.token_info;
    let clock = &ctx.accounts.clock;
    
    // Set pending authority and expiry
    token_info.pending_authority = Some(ctx.accounts.new_authority.key());
    token_info.authority_transfer_expiry = clock.unix_timestamp + AUTHORITY_TRANSFER_EXPIRY;
    
    msg!("Authority transfer requested to {}", ctx.accounts.new_authority.key());
    
    Ok(())
}

pub fn accept_authority_transfer(
    ctx: Context<AcceptAuthorityTransfer>,
) -> Result<()> {
    let token_info = &mut ctx.accounts.token_info;
    
    // Update authority
    token_info.authority = ctx.accounts.new_authority.key();
    token_info.pending_authority = None;
    token_info.authority_transfer_expiry = 0;
    
    msg!("Authority transfer accepted by {}", ctx.accounts.new_authority.key());
    
    Ok(())
}

// Account validation structs with their implementation would be added
#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = mint.mint_authority.unwrap() == authority.key() @ DataChainError::Unauthorized
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = recipient.mint == mint.key() @ DataChainError::InvalidParameters
    )]
    pub recipient: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = token_info.mint == mint.key() @ DataChainError::InvalidParameters,
        constraint = token_info.authority == authority.key() @ DataChainError::Unauthorized
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct InitializeToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<TokenInfo>(),
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = authority.key(),
    )]
    pub mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
} 