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
    #[error("CERROR: Program State Not Initialized")]
    ProgramStateNotInitialized,
    #[error("CERROR: Invalid Merkle Tree")]
    InvalidMerkleTree,
    #[error("CERROR: Invalid Asset State PDA")]
    InvalidAssetStatePDA,
    #[error("CERROR: Asset Already Clocked In")]
    AlreadyClockedIn,
    #[error("CERROR: Asset Not Clocked In")]
    NotClockedIn,
    #[error("CERROR: Invalid Level")]
    InvalidLevel,
    #[error("CERROR: Clock In Disabled")]
    ClockInDisabled,
    #[error("CERROR: Clock Out Disabled")]
    ClockOutDisabled,
    #[error("CERROR: Invalid SPL Token Program ID")]
    InvalidSPLTokenProgramID,
    #[error("CERROR: Invalid McPay Vault PDA")]
    InvalidMcPayVaultPDA,
    #[error("CERROR: Invalid McPay Vault Pickle ATA")]
    InvalidMcPayVaultPickleATA,
    #[error("CERROR: Invalid Associated Token Account")]
    InvalidATA,
    #[error("CERROR: Invalid SPL Account Compression Program ID")]
    InvalidSPLAccountCompressionProgramID,
    #[error("CERROR: Invalid System Program ID")]    
    InvalidSystemProgramID,
    #[error("CERROR: Invalid Clock In Wallet")]    
    InvalidClockInWallet,
    #[error("CERROR: Invalid McDegens Pickle ATA")]    
    InvalidMcDegensPickleATA,
    #[error("CERROR: Insufficient Funds in Pickle Vault")]
    InsufficientVaultPickle,
    #[error("CERROR: Insufficient SOL in McPay Vault")]
    InsufficientVaultSOL,
    #[error("CERROR: Invalid McDegens Treasury")]
    InvalidMcDegensTreasury,
    #[error("CERROR: Too Soon")]
    TooSoon,
    #[error("CERROR: No Updates Indicated")]
    NoUpdatesIndicated,
    #[error("CERROR: AmountOverflow")]
    AmountOverflow,
}

impl From<McPayError> for ProgramError {
    fn from(e: McPayError) -> Self {
        ProgramError::Custom(e as u32)
    }
}