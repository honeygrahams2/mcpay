use borsh::{
    BorshDeserialize, 
    BorshSerialize,
};
use mpl_bubblegum::{
    types::LeafSchema, 
    utils::get_asset_id,
};
use solana_program::{
    account_info::{
        AccountInfo,
        next_account_info, 
    },
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke_signed, 
    rent::Rent, 
    system_instruction, 
    sysvar::Sysvar, 
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use crate::{
    error::McPayError, 
    instruction::McPayInstruction, 
    state::ClockInData,
};

pub fn assert_true(cond: bool, err: ProgramError, msg: &str) -> ProgramResult {
    if !cond {
        msg!(msg);
        Err(err)
    } else {
        Ok(())
    }
}

pub struct Processor {}
impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instr: McPayInstruction = McPayInstruction::unpack(instruction_data)?;
        match instr {
            McPayInstruction::ClockIn {
                clock_in_data,                
            } => {
                msg!("Clock In");
                Self::process_clocking_in(
                    program_id,
                    accounts,
                    clock_in_data,
                )
            },
        }?;

        Ok(())
    }

    fn process_clocking_in(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        clock_in_data: ClockInData,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let signer = next_account_info(accounts_iter)?; // 0
        // let program_state_pda = next_account_info(accounts_iter)?; // 1
        let leaf_delegate = next_account_info(accounts_iter)?; // 2
        let merkle_tree = next_account_info(accounts_iter)?; // 3
        let mpl_bubblegum_program_id = next_account_info(accounts_iter)?; // 4

        if !signer.is_signer {
            msg!("CERROR: Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let asset_id = get_asset_id(merkle_tree.key, clock_in_data.nonce);

        let leaf = mpl_bubblegum::types::LeafSchema::V1 { 
            id: asset_id, 
            owner: *signer.key, 
            delegate: *leaf_delegate.key, 
            nonce: clock_in_data.nonce as u64, 
            data_hash: clock_in_data.data_hash.to_bytes(), 
            creator_hash: clock_in_data.creator_hash.to_bytes(),
        };

        let verify_cpi = mpl_bubblegum::instructions::VerifyLeafCpi::new(
            mpl_bubblegum_program_id, 
            mpl_bubblegum::instructions::VerifyLeafCpiAccounts {
                merkle_tree,
            }, 
            mpl_bubblegum::instructions::VerifyLeafInstructionArgs {
                index: clock_in_data.nonce.try_into().unwrap(),
                leaf: leaf.hash(),
                root: clock_in_data.root.to_bytes(),
            }
        );
        verify_cpi.invoke()?;

        msg!("Success!");

        Ok(())
    }
}