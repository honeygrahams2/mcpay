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
    msg, 
    program::invoke_signed, 
    program_error::ProgramError, 
    program_pack::Pack, 
    pubkey::Pubkey, 
    rent::Rent, 
    system_instruction, 
    sysvar::Sysvar,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account;
use std::str::FromStr;

use crate::{
    error::McPayError, 
    instruction::McPayInstruction, 
    state::{
        AssetState, 
        ClockInData, 
        ClockOutData,
        ProgramState,
        TransferPickleData,
        TransferSOLData,
        UpdateStateData,
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
                Self::process_clock_in(
                    program_id,
                    accounts,
                    clock_in_data,
                )
            },
            McPayInstruction::ClockOut {
                clock_out_data,                
            } => {
                msg!("Clock Out");
                Self::process_clock_out(
                    program_id,
                    accounts,
                    clock_out_data,
                )
            },
            McPayInstruction::UpdateState {
                update_state_data,                
            } => {
                msg!("Update State");
                Self::process_update_state(
                    program_id,
                    accounts,
                    update_state_data,
                )
            },
            McPayInstruction::CloseProgramState {} => {
                msg!("Close Program State");
                Self::process_close_program_state(
                    program_id,
                    accounts,
                )
            },
            McPayInstruction::TransferPickle {
                transfer_pickle_data
            } => {
                msg!("Transfer Pickle");
                Self::process_transfer_pickle(
                    program_id,
                    accounts,
                    transfer_pickle_data,
                )
            },
            McPayInstruction::TransferSOL {
                transfer_sol_data
            } => {
                msg!("Transfer SOL");
                Self::process_transfer_sol(
                    program_id,
                    accounts,
                    transfer_sol_data,
                )
            },
        }?;

        Ok(())
    }

    fn process_clock_in(
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
        msg!("asset_id {}", asset_id);

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
            clock_out_utime += 300; //86_400;
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
            let asset_state_size = 1 + 32 + 32 + 8 + 8 + 1 + 8;
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
            asset_state_data.clock_in_wallet = *signer.key;
            asset_state_data.asset_id = asset_id;
            asset_state_data.clock_in_utime = utime;
            asset_state_data.clock_out_utime = clock_out_utime;
            asset_state_data.level = clock_in_data.level;
            asset_state_data.chips_due = chips_due;
            asset_state_data.serialize(&mut &mut asset_state_pda.data.borrow_mut()[..])?;
        } else {
            msg!("CERROR: Asset already clocked in");
            return Err(McPayError::AlreadyClockedIn.into());
        }

        Ok(())
    }

    fn process_clock_out(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        clock_out_data: ClockOutData,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let signer = next_account_info(accounts_iter)?; // 0
        let program_state_pda = next_account_info(accounts_iter)?; // 1
        let asset_state_pda = next_account_info(accounts_iter)?; // 2
        let leaf_delegate = next_account_info(accounts_iter)?; // 3
        let merkle_tree = next_account_info(accounts_iter)?; // 4
        let spl_account_compression_program_id = next_account_info(accounts_iter)?; // 5
        let spl_token_program_id = next_account_info(accounts_iter)?; // 6
        let mcpay_vault_pda = next_account_info(accounts_iter)?; // 7
        let mcpay_vault_pickle_ata = next_account_info(accounts_iter)?; // 8
        let signer_pickle_ata = next_account_info(accounts_iter)?; // 9
        let clock_in_wallet = next_account_info(accounts_iter)?; // 10

        let mut remaining_accounts:  Vec<AccountInfo> = vec![];
        for _n in 0..clock_out_data.proof_length {
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
            program_state_data.clock_out_is_enabled == 1,
            ProgramError::from(McPayError::ClockOutDisabled),
            "CERROR: Clock out disabled",
        )?;

        let asset_id = mpl_bubblegum::utils::get_asset_id(merkle_tree.key, clock_out_data.nonce);
        let (asset_state, _asset_state_bump) = Pubkey::find_program_address(
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
            *spl_token_program_id.key == spl_token::id(),
            ProgramError::from(McPayError::InvalidSPLTokenProgramID),
            "CERROR: Invalid SPL Token Program ID",
        )?;

        assert_true(
            *mcpay_vault_pda.key == program_state_data.mcpay_vault_pda,
            ProgramError::from(McPayError::InvalidMcPayVaultPDA),
            "CERROR: Invalid mcpay vault pda",
        )?;
        
        assert_true(
            *mcpay_vault_pickle_ata.key == program_state_data.mcpay_vault_pickle_ata,
            ProgramError::from(McPayError::InvalidMcPayVaultPickleATA),
            "CERROR: Invalid mcpay vault pickle ata",
        )?;
 
        let signer_pickle = get_associated_token_address(
            &signer.key, 
            &program_state_data.pickle_mint
        );
        assert_true(
            *signer_pickle_ata.key == signer_pickle,
            ProgramError::from(McPayError::InvalidATA),
            "CERROR: Invalid signer pickle ata",
        )?;
        
        let mcpay_vault_pickle_ata_data = Account::unpack(&mcpay_vault_pickle_ata.try_borrow_data()?)?;
        let asset_state_data: AssetState = AssetState::try_from_slice(&asset_state_pda.data.borrow())?;
        assert_true(                    
            mcpay_vault_pickle_ata_data.amount >= asset_state_data.chips_due,
            ProgramError::from(McPayError::InsufficientVaultPickle),
            "CERROR: Insufficient funds in Pickle Vault",
        )?;

        let leaf = mpl_bubblegum::types::LeafSchema::V1 { 
            id: asset_id, 
            owner: *signer.key, 
            delegate: *leaf_delegate.key, 
            nonce: clock_out_data.nonce, 
            data_hash: clock_out_data.data_hash.to_bytes(), 
            creator_hash: clock_out_data.creator_hash.to_bytes(),
        };

        let verify_leaf_cpi = mpl_bubblegum::instructions::VerifyLeafCpi::new(
            spl_account_compression_program_id, 
            mpl_bubblegum::instructions::VerifyLeafCpiAccounts {
                merkle_tree,
            }, 
            mpl_bubblegum::instructions::VerifyLeafInstructionArgs {
                index: clock_out_data.nonce.try_into().unwrap(),
                leaf: leaf.hash(),
                root: clock_out_data.root.to_bytes(),
            }
        );
        verify_leaf_cpi.invoke_with_remaining_accounts(
            remaining_accounts
                .iter()
                .map(|account| (account, false, false))
                .collect::<Vec<_>>()
                .as_slice()
        )?;

        if !asset_state_pda.data_is_empty() {
            assert_true(
                *clock_in_wallet.key == asset_state_data.clock_in_wallet,
                ProgramError::from(McPayError::InvalidClockInWallet),
                "CERROR: Invalid clock in wallet",
            )?;
                
            let clock = Clock::get()?;
            let utime = clock.unix_timestamp;
            if asset_state_data.clock_out_utime <= utime {
                msg!("Transferring Pickle");
                let transfer_vault_pickle_ix = spl_token::instruction::transfer(
                    spl_token_program_id.key,
                    mcpay_vault_pickle_ata.key,
                    signer_pickle_ata.key,
                    &mcpay_vault_pda.key,
                    &[],
                    asset_state_data.chips_due,
                )?;
                invoke_signed(
                    &transfer_vault_pickle_ix,
                    &[
                        mcpay_vault_pickle_ata.clone(),
                        signer_pickle_ata.clone(),
                        mcpay_vault_pda.clone(),
                    ],
                    &[&[b"mcpay-vault", &[program_state_data.mcpay_vault_bump]]],
                )?;

                msg!("Closing Asset State");
                **clock_in_wallet.try_borrow_mut_lamports()? = clock_in_wallet
                    .lamports()
                    .checked_add(asset_state_pda.lamports())
                    .ok_or(McPayError::AmountOverflow)?;
                **asset_state_pda.try_borrow_mut_lamports()? = 0;
                *asset_state_pda.try_borrow_mut_data()? = &mut [];
            } else {
                msg!("CERROR: To soon");
                return Err(McPayError::TooSoon.into());    
            }
        } else {
            msg!("CERROR: Asset not clocked in");
            return Err(McPayError::NotClockedIn.into());
        }

        Ok(())
    }

    fn process_update_state(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        update_state_data: UpdateStateData,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let signer = next_account_info(accounts_iter)?; // 0
        let program_state_pda = next_account_info(accounts_iter)?; // 1
        let system_program_id = next_account_info(accounts_iter)?; // 2

        let base_account = Pubkey::from_str("25hZAxGdWsP158Y8NG9eZbDwiS5bsku5UEx7ZLzVGhta").unwrap();
        let base2_account = Pubkey::from_str("Aq3Nm72sY2hJScVQ89rzMKF22f3zUCbF2eioUhMeJuDj").unwrap();

        if !signer.is_signer {
            msg!("CERROR: Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }
        assert_true(
            (*signer.key == base_account) || (*signer.key == base2_account),
            ProgramError::from(ProgramError::MissingRequiredSignature),
            "CERROR: Invalid signature",
        )?;

        let (program_state, program_state_bump) = Pubkey::find_program_address(&[b"program-state"], program_id);
        assert_true(
            program_state == *program_state_pda.key,
            ProgramError::from(McPayError::InvalidProgramStatePDA),
            "CERROR: Invalid program state pda",
        )?;

        assert_true(
            *system_program_id.key == solana_program::system_program::id(),
            ProgramError::from(McPayError::InvalidSystemProgramID),
            "CERROR: Invalid System Program ID",
        )?;

        if update_state_data.new_clock_in_is_enabled < 2 || 
            update_state_data.new_clock_out_is_enabled < 2 ||
            update_state_data.new_merkle_tree != Pubkey::from_str("11111111111111111111111111111111").unwrap() ||
            update_state_data.new_level_one_rate > 0 ||
            update_state_data.new_level_seven_rate > 0 ||
            update_state_data.new_level_thirty_rate > 0 ||
            update_state_data.new_pickle_mint != Pubkey::from_str("11111111111111111111111111111111").unwrap() || 
            update_state_data.new_mcdegens_treasury != Pubkey::from_str("11111111111111111111111111111111").unwrap() ||
            update_state_data.new_mcdegens_pickle_ata != Pubkey::from_str("11111111111111111111111111111111").unwrap()
            {                

            if program_state_pda.data_is_empty()
            {
                msg!("Creating Program State Account");
                let program_state_size = 1 + 1 + 1 + 32 + 8 + 8 + 8 + 32 + 32 + 1 + 32 + 32 + 32;
                invoke_signed(
                    &system_instruction::create_account(
                        signer.key,
                        &program_state_pda.key,
                        Rent::get()?.minimum_balance(program_state_size),
                        program_state_size as u64,
                        program_id,
                    ),
                    &[
                        signer.clone(),
                        program_state_pda.clone(),
                        system_program_id.clone(),
                    ],
                    &[&[
                        b"program-state",
                        &[program_state_bump],
                    ]],
                )?;
            }
            
            let mut program_state: ProgramState = ProgramState::try_from_slice(&program_state_pda.data.borrow())?;
            if program_state.is_initialized {
                if update_state_data.new_clock_in_is_enabled < 2 {
                    program_state.clock_in_is_enabled = update_state_data.new_clock_in_is_enabled;
                }
                if update_state_data.new_clock_out_is_enabled < 2 {
                    program_state.clock_out_is_enabled = update_state_data.new_clock_out_is_enabled;
                }
                if update_state_data.new_merkle_tree != Pubkey::from_str("11111111111111111111111111111111").unwrap() {
                    program_state.merkle_tree = update_state_data.new_merkle_tree;
                }
                if update_state_data.new_level_one_rate > 0 {
                    program_state.level_one_rate = update_state_data.new_level_one_rate;
                }
                if update_state_data.new_level_seven_rate > 0 {
                    program_state.level_seven_rate = update_state_data.new_level_seven_rate;
                }
                if update_state_data.new_level_thirty_rate > 0 {
                    program_state.level_thirty_rate = update_state_data.new_level_thirty_rate;
                }
                if update_state_data.new_pickle_mint != Pubkey::from_str("11111111111111111111111111111111").unwrap() {
                    program_state.pickle_mint = update_state_data.new_pickle_mint;
                }
                if update_state_data.new_mcdegens_treasury != Pubkey::from_str("11111111111111111111111111111111").unwrap() {
                    program_state.mcdegens_treasury = update_state_data.new_mcdegens_treasury;
                }
                if update_state_data.new_mcdegens_pickle_ata != Pubkey::from_str("11111111111111111111111111111111").unwrap() {
                    program_state.mcdegens_pickle_ata = update_state_data.new_mcdegens_pickle_ata;
                }
           } else {
                let (mcpay_vault_pda, mcpay_vault_bump) = Pubkey::find_program_address(&[b"mcpay-vault"], program_id);
                let mcpay_vault_pickle_ata = get_associated_token_address(&mcpay_vault_pda, &update_state_data.new_pickle_mint);

                program_state.is_initialized = true;
                program_state.clock_in_is_enabled = update_state_data.new_clock_in_is_enabled;
                program_state.clock_out_is_enabled = update_state_data.new_clock_out_is_enabled;
                program_state.merkle_tree = update_state_data.new_merkle_tree;
                program_state.level_one_rate = update_state_data.new_level_one_rate;
                program_state.level_seven_rate = update_state_data.new_level_seven_rate;
                program_state.level_thirty_rate = update_state_data.new_level_thirty_rate;
                program_state.pickle_mint = update_state_data.new_pickle_mint;
                program_state.mcpay_vault_pda = mcpay_vault_pda;
                program_state.mcpay_vault_bump = mcpay_vault_bump;
                program_state.mcpay_vault_pickle_ata = mcpay_vault_pickle_ata;
                program_state.mcdegens_treasury = update_state_data.new_mcdegens_treasury;
                program_state.mcdegens_pickle_ata = update_state_data.new_mcdegens_pickle_ata;
            }
            program_state.serialize(&mut &mut program_state_pda.data.borrow_mut()[..])?;

            msg!("Success!");
        } else {
            msg!("CERROR: No updates indicated");
            return Err(McPayError::NoUpdatesIndicated.into());
        }

        Ok(())
    }

    fn process_close_program_state(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let signer = next_account_info(accounts_iter)?; // 0
        let program_state_pda = next_account_info(accounts_iter)?; // 1

        let base_account = Pubkey::from_str("25hZAxGdWsP158Y8NG9eZbDwiS5bsku5UEx7ZLzVGhta").unwrap();
        let base2_account = Pubkey::from_str("Aq3Nm72sY2hJScVQ89rzMKF22f3zUCbF2eioUhMeJuDj").unwrap();

        if !signer.is_signer {
            msg!("CERROR: Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }
        assert_true(
            (*signer.key == base_account) || (*signer.key == base2_account),
            ProgramError::from(ProgramError::MissingRequiredSignature),
            "CERROR: Invalid signature",
        )?;

        let (program_state, _program_state_bump) = Pubkey::find_program_address(&[b"program-state"], program_id);
        assert_true(
            program_state == *program_state_pda.key,
            ProgramError::from(McPayError::InvalidProgramStatePDA),
            "CERROR: Invalid program state pda",
        )?;

        msg!("Closing Program State");
        **signer.try_borrow_mut_lamports()? = signer
            .lamports()
            .checked_add(program_state_pda.lamports())
            .ok_or(McPayError::AmountOverflow)?;
        **program_state_pda.try_borrow_mut_lamports()? = 0;
        *program_state_pda.try_borrow_mut_data()? = &mut [];

        msg!("Success!");
        
        Ok(())
    }

    fn process_transfer_pickle(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        transfer_pickle_data: TransferPickleData,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let signer = next_account_info(accounts_iter)?; // 0
        let program_state_pda = next_account_info(accounts_iter)?; // 1
        let mcpay_vault_pda= next_account_info(accounts_iter)?; // 2
        let mcpay_vault_pickle_ata= next_account_info(accounts_iter)?; // 3
        let mcdegens_pickle_ata = next_account_info(accounts_iter)?; // 4
        let spl_token_program_id = next_account_info(accounts_iter)?; // 5

        let base_account = Pubkey::from_str("25hZAxGdWsP158Y8NG9eZbDwiS5bsku5UEx7ZLzVGhta").unwrap();
        let base2_account = Pubkey::from_str("Aq3Nm72sY2hJScVQ89rzMKF22f3zUCbF2eioUhMeJuDj").unwrap();

        if !signer.is_signer {
            msg!("CERROR: Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }
        assert_true(
            (*signer.key == base_account) || (*signer.key == base2_account),
            ProgramError::from(ProgramError::MissingRequiredSignature),
            "CERROR: Invalid signature",
        )?;

        let (program_state, _program_state_bump) = Pubkey::find_program_address(&[b"program-state"], program_id);
        assert_true(
            program_state == *program_state_pda.key,
            ProgramError::from(McPayError::InvalidProgramStatePDA),
            "CERROR: Invalid program state pda",
        )?;

        let program_state: ProgramState = ProgramState::try_from_slice(&program_state_pda.data.borrow())?;
        if !program_state.is_initialized {
            msg!("CERROR: Program state not initialized");
            return Err(McPayError::ProgramStateNotInitialized.into());            
        }

        assert_true(
            program_state.mcdegens_pickle_ata == *mcdegens_pickle_ata.key,
            ProgramError::from(McPayError::InvalidMcDegensPickleATA),
            "CERROR: Invalid mcdegens pickle ATA",
        )?;

        assert_true(
            program_state.mcpay_vault_pickle_ata == *mcpay_vault_pickle_ata.key,
            ProgramError::from(McPayError::InvalidMcPayVaultPickleATA),
            "CERROR: Invalid mcpay vault pickle ata",
        )?;
        
        let mcpay_vault_pickle_ata_data = Account::unpack(&mcpay_vault_pickle_ata.try_borrow_data()?)?;
        assert_true(                    
            mcpay_vault_pickle_ata_data.amount >= transfer_pickle_data.chips,
            ProgramError::from(McPayError::InsufficientVaultPickle),
            "CERROR: Insufficient funds in Pickle Vault",
        )?;

        msg!("Transferring Pickle");
        let transfer_pickle_ix = spl_token::instruction::transfer(
            spl_token_program_id.key,
            mcpay_vault_pickle_ata.key,
            mcdegens_pickle_ata.key,
            &mcpay_vault_pda.key,
            &[],
            transfer_pickle_data.chips,
        )?;
        invoke_signed(
            &transfer_pickle_ix,
            &[
                mcpay_vault_pickle_ata.clone(),
                mcdegens_pickle_ata.clone(),
                mcpay_vault_pda.clone(),
            ],
            &[&[
                b"mcpay-vault",
                &[program_state.mcpay_vault_bump],
            ]],
        )?;

        msg!("Success!");

        Ok(())
    }

    fn process_transfer_sol(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        transfer_sol_data: TransferSOLData,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let signer = next_account_info(accounts_iter)?; // 0
        let program_state_pda = next_account_info(accounts_iter)?; // 1
        let mcpay_vault_pda = next_account_info(accounts_iter)?; // 2
        let mcdegens_treasury = next_account_info(accounts_iter)?; // 3
        let system_program_id = next_account_info(accounts_iter)?; // 4

        let base_account = Pubkey::from_str("25hZAxGdWsP158Y8NG9eZbDwiS5bsku5UEx7ZLzVGhta").unwrap();
        let base2_account = Pubkey::from_str("Aq3Nm72sY2hJScVQ89rzMKF22f3zUCbF2eioUhMeJuDj").unwrap();

        if !signer.is_signer {
            msg!("CERROR: Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }
        assert_true(
            (*signer.key == base_account) || (*signer.key == base2_account),
            ProgramError::from(ProgramError::MissingRequiredSignature),
            "CERROR: Invalid signature",
        )?;

        let (program_state, _program_state_bump) = Pubkey::find_program_address(&[b"program-state"], program_id);
        assert_true(
            program_state == *program_state_pda.key,
            ProgramError::from(McPayError::InvalidProgramStatePDA),
            "CERROR: Invalid program state pda",
        )?;
        
        let program_state: ProgramState = ProgramState::try_from_slice(&program_state_pda.data.borrow())?;
        if !program_state.is_initialized {
            msg!("CERROR: Program state not initialized");
            return Err(McPayError::ProgramStateNotInitialized.into());            
        }

        assert_true(
            program_state.mcpay_vault_pda == *mcpay_vault_pda.key,
            ProgramError::from(McPayError::InvalidMcPayVaultPDA),
            "CERROR: Invalid mcpay vault pda",
        )?;

        assert_true(
            mcpay_vault_pda.lamports() >= transfer_sol_data.lamports,
            ProgramError::from(McPayError::InsufficientVaultSOL),
            "CERROR: Insufficient SOL in McPay Vault",
        )?;

        assert_true(
            program_state.mcdegens_treasury == *mcdegens_treasury.key,
            ProgramError::from(McPayError::InvalidMcDegensTreasury),
            "CERROR: Invalid McDegens Treasury",
        )?;

        msg!("Transferring SOL");
        let transfer_sol_ix = solana_program::system_instruction::transfer(
            mcpay_vault_pda.key, 
            mcdegens_treasury.key, 
            transfer_sol_data.lamports,
        );
        invoke_signed(
            &transfer_sol_ix,
            &[
                mcpay_vault_pda.clone(),
                mcdegens_treasury.clone(),
                system_program_id.clone(),
            ],
            &[
                &[
                    b"mcpay-vault",
                    &[program_state.mcpay_vault_bump],
                ]
            ],
        )?;

        msg!("Success!");

        Ok(())
    }
}