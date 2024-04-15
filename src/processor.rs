// use borsh::{
//     BorshDeserialize, 
//     BorshSerialize,
// };
use solana_program::{
    account_info::{
        AccountInfo,
        next_account_info,
    }, entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError, 
    pubkey::Pubkey 
};

use crate::{
    // error::McPayError, 
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
        _program_id: &Pubkey,
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
                    // program_id,
                    accounts,
                    clock_in_data,
                )
            },
        }?;

        Ok(())
    }

    fn process_clocking_in(
        // program_id: &Pubkey,
        accounts: &[AccountInfo],
        clock_in_data: ClockInData,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let signer = next_account_info(accounts_iter)?; // 0
        let program_state_pda = next_account_info(accounts_iter)?; // 1
        let asset_state_pda = next_account_info(accounts_iter)?; // 2
        let leaf_delegate = next_account_info(accounts_iter)?; // 3
        let merkle_tree = next_account_info(accounts_iter)?; // 4
        let spl_account_compression_program_id = next_account_info(accounts_iter)?; // 5

        if !signer.is_signer {
            msg!("CERROR: Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut remaining_accounts:  Vec<AccountInfo> = vec![];
        for _n in 0..clock_in_data.proof_length {
            let acct = next_account_info(accounts_iter)?;
            remaining_accounts.push(acct.clone());
        }

        let asset_id = mpl_bubblegum::utils::get_asset_id(merkle_tree.key, clock_in_data.nonce);

        let leaf = mpl_bubblegum::types::LeafSchema::V1 { 
            id: asset_id, 
            owner: *signer.key, 
            delegate: *leaf_delegate.key, 
            nonce: clock_in_data.nonce, 
            data_hash: clock_in_data.data_hash.to_bytes(), 
            creator_hash: clock_in_data.creator_hash.to_bytes(),
        };

        let verify_leaf_cpi = mpl_bubblegum::instructions::VerifyLeafCpi::new(
            spl_account_compression_program_id, 
            mpl_bubblegum::instructions::VerifyLeafCpiAccounts {
                merkle_tree,
            }, 
            mpl_bubblegum::instructions::VerifyLeafInstructionArgs {
                index: clock_in_data.nonce.try_into().unwrap(),
                leaf: leaf.hash(),
                root: clock_in_data.root.to_bytes(),
            }
        );
        verify_leaf_cpi.invoke_with_remaining_accounts(
            remaining_accounts
                .iter()
                .map(|account| (account, false, false))
                .collect::<Vec<_>>()
                .as_slice()
        )?;

        Ok(())
    }
}