use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ProgramState {
    pub is_initialized: bool,
    pub is_enabled: u8,
    pub merkle_tree: Pubkey,
    pub level_one_rate: u64,
    pub level_seven_rate: u64,
    pub level_thirty_rate: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct AssetState {
    pub is_initialized: bool,
    pub clock_in_utime: i64,
    pub clock_in_wallet: Pubkey,
    pub clock_out_utime: i64,
    pub pickle_due: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ClockInData {
    pub root: Pubkey,
    pub data_hash: Pubkey,
    pub creator_hash: Pubkey,
    pub nonce: u64,
    pub proof_length: u8,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ClockOutData {
    pub root: Pubkey,
    pub data_hash: Pubkey,
    pub creator_hash: Pubkey,
    pub nonce: u64,
    pub proof_length: u8,
}
