use borsh::{
    BorshDeserialize, 
    BorshSerialize,
};
use solana_program::{
    account_info::{
        AccountInfo,
        next_account_info,
    }, 
    clock::Clock,
    entrypoint::ProgramResult, 
    program::invoke_signed,
    msg, 
    program_error::ProgramError, 
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::McPayError, 
    instruction::McPayInstruction, 
    state::{
        AssetState, 
        ClockInData, 
        ClockOutData, 
        ProgramState
    }
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
        let program_state_pda = next_account_info(accounts_iter)?; // 1
        let asset_state_pda = next_account_info(accounts_iter)?; // 2
        let leaf_delegate = next_account_info(accounts_iter)?; // 3
        let merkle_tree = next_account_info(accounts_iter)?; // 4
        let spl_account_compression_program_id = next_account_info(accounts_iter)?; // 5
        let system_program_id = next_account_info(accounts_iter)?; // 6

        let mut remaining_accounts:  Vec<AccountInfo> = vec![];
        for _n in 0..clock_in_data.proof_length {
            let acct = next_account_info(accounts_iter)?;
            remaining_accounts.push(acct.clone());
        }

        if !signer.is_signer {
            msg!("CERROR: Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (program_state, _program_state_bump) = Pubkey::find_program_address(
            &[b"program-state"], 
            program_id,
        );
        assert_true(
            program_state == *program_state_pda.key,
            ProgramError::from(McPayError::InvalidProgramStatePDA),
            "CERROR: Invalid program state pda",
        )?;

        let program_state_data: ProgramState = ProgramState::try_from_slice(&program_state_pda.data.borrow())?;
        assert_true(
            program_state_data.is_initialized,
            ProgramError::from(McPayError::ProgramStateNotInitialized),
            "CERROR: Program state not initialized",
        )?;

        assert_true(
            *merkle_tree.key == program_state_data.merkle_tree,
            ProgramError::from(McPayError::InvalidMerkleTree),
            "CERROR: Invalid merkle tree",
        )?;

        assert_true(
            program_state_data.clock_in_is_enabled == 1,
            ProgramError::from(McPayError::ClockInDisabled),
            "CERROR: Clock in disabled",
        )?;

        let asset_id = mpl_bubblegum::utils::get_asset_id(merkle_tree.key, clock_in_data.nonce);

        let (asset_state, asset_state_bump) = Pubkey::find_program_address(
            &[
                b"asset-state",
                asset_id.as_ref(),
            ],
            program_id,
        );
        assert_true(
            asset_state == *asset_state_pda.key,
            ProgramError::from(McPayError::InvalidAssetStatePDA),
            "CERROR: Invalid asset state pda",
        )?;

        assert_true(
            *spl_account_compression_program_id.key == spl_account_compression::id(),
            ProgramError::from(McPayError::InvalidSPLAccountCompressionProgramID),
            "CERROR: Invalid SPL Account Compression Program ID",
        )?;
        
        assert_true(
            *system_program_id.key == solana_program::system_program::id(),
            ProgramError::from(McPayError::InvalidSystemProgramID),
            "CERROR: Invalid System Program ID",
        )?;

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
        
        let clock = Clock::get()?;
        let utime = clock.unix_timestamp;

        let mut clock_out_utime = utime;        
        let mut chips_due = 0;
        if clock_in_data.level == 1 {
            clock_out_utime += 86_400;
            chips_due = program_state_data.level_one_rate;
        } else if clock_in_data.level == 7 {
            clock_out_utime += 86_400 * clock_in_data.level as i64;
            chips_due = program_state_data.level_seven_rate;
        } else if clock_in_data.level == 30 {
            clock_out_utime += 86_400 * clock_in_data.level as i64;
            chips_due = program_state_data.level_thirty_rate;
        }         
        assert_true(
            chips_due > 0,
            ProgramError::from(McPayError::InvalidLevel),
            "CERROR: Invalid level",
        )?;

        if asset_state_pda.data_is_empty() {
            msg!("Creating Asset State");
            let asset_state_size = 1 + 8 + 32 + 8 + 1 + 8;
            invoke_signed(
                &system_instruction::create_account(
                    signer.key,
                    &asset_state_pda.key,
                    Rent::get()?.minimum_balance(asset_state_size),
                    asset_state_size as u64,
                    program_id,
                ),
                &[
                    signer.clone(),
                    asset_state_pda.clone(),
                    system_program_id.clone(),
                ],
                &[&[
                    b"asset-state",
                    asset_id.as_ref(),
                    &[asset_state_bump],
                ]],
            )?;

            let mut asset_state_data: AssetState = AssetState::try_from_slice(&asset_state_pda.data.borrow())?;
            asset_state_data.is_initialized = true;
            asset_state_data.clock_in_utime = utime;
            asset_state_data.clock_in_wallet = *signer.key;
            asset_state_data.clock_out_utime = clock_out_utime;
            asset_state_data.level = clock_in_data.level;
            asset_state_data.chips_due = chips_due;
            asset_state_data.serialize(&mut &mut asset_state_pda.data.borrow_mut()[..])?;
        } else {
            msg!("CERROR: Asset Already Clocked In");
            return Err(McPayError::AlreadyClockedIn.into());
        }

        Ok(())
    }
}