use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum McPayError {
    #[error("CERROR: Invalid Instruction")]
    InvalidInstruction,
    #[error("CERROR: Invalid Instruction Data")]
    InvalidInstructionData,
    #[error("CERROR: Invalid Program State PDA")]
    InvalidProgramStatePDA,
    #[error("CERROR: Invalid Wallet State PDA")]
    InvalidWalletStatePDA,
    #[error("CERROR: Invalid Receipt State PDA")]
    InvalidReceiptStatePDA,
    #[error("CERROR: Program State Not Initialized")]
    ProgramStateNotInitialized,
    #[error("CERROR: Invalid Mint Authority")]
    InvalidMintAuthority,
    #[error("CERROR: Invalid Merkle Tree")]
    InvalidMerkleTree,
    #[error("CERROR: Invalid Tree Authority")]
    InvalidTreeAuthority,
    #[error("CERROR: Invalid SPL Token Program ID")]
    InvalidSPLTokenID,
    #[error("CERROR: Invalid Treasury")]
    InvalidTreasury,
    #[error("CERROR: Invalid Associated Token Account")]
    InvalidATA,
    #[error("CERROR: Invalid MPL Token Metadata Program ID")]
    InvalidMPLTokenMetadataProgramID,
    #[error("CERROR: Invalid Metadata Account")]
    InvalidMetadataAccount,
    #[error("CERROR: Invalid Bubblegum Program ID")]
    InvalidBubblegumProgramID,
    #[error("CERROR: Invalid Bubblegum Authority")]
    InvalidBubblegumAuthority,
    #[error("CERROR: Invalid SPL Account Compression Program ID")]
    InvalidSPLAccountCompressionProgramID,
    #[error("CERROR: Invalid SPL Noop Program ID")]
    InvalidSPLNoopProgramID,
    #[error("CERROR: Invalid System Program ID")]
    InvalidSystemProgramID,
    #[error("CERROR: Invalid Sysvar Rent Program ID")]
    InvalidSysvarRentProgramID,
    #[error("CERROR: Invalid SPL Associated Token Program ID")]
    InvalidSPLAssociatedTokenProgramID,
    #[error("CERROR: Invalid Master Edition Account")]
    InvalidMasterEditionAccount,
    #[error("CERROR: Invalid Collection Mint")]
    InvalidCollectionMint,
    #[error("CERROR: Invalid Collection Metadata Account")]
    InvalidCollectionMetadataAccount,
    #[error("CERROR: Invalid Collection Master Edition Account")]
    InvalidCollectionMasterEditionAccount,
    #[error("CERROR: Sold Out")]
    SoldOut,
    #[error("CERROR: Whitelist Only")]
    WhitelistOnly,
    #[error("CERROR: Wallet Blacklisted")]
    WalletBlacklisted,
    #[error("CERROR: Max Per Wallet Reached")]
    MaxPerWalletReached,
    #[error("CERROR: Too Soon")]
    TooSoon,
    #[error("CERROR: Duplicate Receipt")]
    DuplicateReceipt,
    #[error("CERROR: Wallet Already on Whitelist")]
    AlreadyOnWhitelist,
    #[error("CERROR: Invalid Fee Amount")]
    InvalidFeeAmount,
    #[error("CERROR: No Updates Indicated")]
    NoUpdatesIndicated,
    #[error("CERROR: Wallet State Not Initialized")]
    WalletStateNotInitialized,
    #[error("CERROR: Wallet State Already Initialized")]
    WalletStateAlreadyInitialized,
    #[error("CERROR: AmountOverflow")]
    AmountOverflow,
}

impl From<McPayError> for ProgramError {
    fn from(e: McPayError) -> Self {
        ProgramError::Custom(e as u32)
    }
}