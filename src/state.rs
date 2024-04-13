use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ProgramState {
    pub is_initialized: bool,
    pub merkle_tree: Pubkey,
    pub collection_mint: Pubkey,
    pub collection_metadata: Pubkey,
    pub collection_master_edition: Pubkey,
    pub mcdegens_treasury: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ClockInData {
    pub root: Pubkey,
    pub data_hash: Pubkey,
    pub creator_hash: Pubkey,
    pub nonce: u64,
}
