use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ProgramState {  // 1 + 1 + 1 + 32 + 8 + 8 + 8 + 32 + 32 + 1 + 32 + 32 + 32
    pub is_initialized: bool,
    pub clock_in_is_enabled: u8,
    pub clock_out_is_enabled: u8,
    pub merkle_tree: Pubkey,
    pub level_one_rate: u64,
    pub level_seven_rate: u64,
    pub level_thirty_rate: u64,
    pub pickle_mint: Pubkey,
    pub mcpay_vault_pda: Pubkey,
    pub mcpay_vault_bump: u8,
    pub mcpay_vault_pickle_ata: Pubkey,
    pub mcdegens_treasury: Pubkey,
    pub mcdegens_pickle_ata: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct AssetState {  // 1 + 32 + 8 + 8 + 1 + 8
    pub is_initialized: bool,
    pub clock_in_wallet: Pubkey,
    pub clock_in_utime: i64,
    pub clock_out_utime: i64,
    pub level: u8,
    pub chips_due: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ClockInData {
    pub root: Pubkey,
    pub data_hash: Pubkey,
    pub creator_hash: Pubkey,
    pub nonce: u64,
    pub proof_length: u8,
    pub level: u8,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ClockOutData {
    pub root: Pubkey,
    pub data_hash: Pubkey,
    pub creator_hash: Pubkey,
    pub nonce: u64,
    pub proof_length: u8,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UpdateStateData {
    pub new_clock_in_is_enabled: u8,
    pub new_clock_out_is_enabled: u8,
    pub new_merkle_tree: Pubkey,
    pub new_level_one_rate: u64,
    pub new_level_seven_rate: u64,
    pub new_level_thirty_rate: u64,
    pub new_pickle_mint: Pubkey,
    pub new_mcdegens_treasury: Pubkey,
    pub new_mcdegens_pickle_ata: Pubkey,
}
