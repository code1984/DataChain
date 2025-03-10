use anchor_lang::prelude::*;

#[error_code]
pub enum DataChainError {
    #[msg("Operation not permitted")]
    Unauthorized,
    
    #[msg("Invalid parameters provided")]
    InvalidParameters,
    
    #[msg("Insufficient token balance")]
    InsufficientBalance,
    
    #[msg("Token transfer failed")]
    TransferFailed,
    
    #[msg("Token mint failed")]
    MintFailed,
    
    #[msg("Dataset not found")]
    DatasetNotFound,
    
    #[msg("Dataset already registered")]
    DatasetAlreadyRegistered,
    
    #[msg("Invalid dataset price")]
    InvalidDatasetPrice,
    
    #[msg("Access already purchased")]
    AccessAlreadyPurchased,
    
    #[msg("Access not purchased")]
    AccessNotPurchased,
    
    #[msg("Invalid rating value (must be 1-5)")]
    InvalidRating,
    
    #[msg("Already rated")]
    AlreadyRated,
    
    #[msg("AI model not found")]
    ModelNotFound,
    
    #[msg("AI model already registered")]
    ModelAlreadyRegistered,
    
    #[msg("Invalid model price")]
    InvalidModelPrice,
    
    #[msg("Model usage failed")]
    ModelUsageFailed,
    
    #[msg("Proposal not found")]
    ProposalNotFound,
    
    #[msg("Proposal already exists")]
    ProposalAlreadyExists,
    
    #[msg("Invalid proposal type")]
    InvalidProposalType,
    
    #[msg("Proposal voting period ended")]
    ProposalVotingEnded,
    
    #[msg("Proposal execution failed")]
    ProposalExecutionFailed,
    
    #[msg("Already voted on proposal")]
    AlreadyVoted,
    
    #[msg("Insufficient voting power")]
    InsufficientVotingPower,
    
    #[msg("Proposal quorum not reached")]
    QuorumNotReached,
    
    #[msg("Proposal rejected")]
    ProposalRejected,
    
    #[msg("Invalid token metadata")]
    InvalidTokenMetadata,
    
    #[msg("Token minting cooldown active")]
    MintingCooldown,
    
    #[msg("Token maximum supply reached")]
    MaxSupplyReached,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    
    #[msg("Invalid authority")]
    InvalidAuthority,
    
    #[msg("System error")]
    SystemError,
} 