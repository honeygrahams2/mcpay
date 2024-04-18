use borsh::{
    BorshDeserialize, 
    BorshSerialize
};
use solana_program::program_error::ProgramError;

use crate::error::McPayError;
use crate::state::{
    ClockInData, 
    ClockOutData,
    TransferPickleData,
    TransferSOLData,
    UpdateStateData,
};

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum McPayInstruction {
    ClockIn {
        clock_in_data: ClockInData,
    },
    ClockOut {
        clock_out_data: ClockOutData,
    },
    CloseProgramState{},
    TransferPickle {
        transfer_pickle_data: TransferPickleData,
    },
    TransferSOL{
        transfer_sol_data: TransferSOLData,
    },
    UpdateState {
        update_state_data: UpdateStateData,
    }
}

impl McPayInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(McPayError::InvalidInstruction)?;
        Ok(match tag {
            0 => Self::ClockIn {
                clock_in_data: ClockInData::try_from_slice(rest).unwrap()
            },
            1 => Self::ClockOut {
                clock_out_data: ClockOutData::try_from_slice(rest).unwrap()
            },
            2 => Self::UpdateState {
                update_state_data: UpdateStateData::try_from_slice(rest).unwrap()
            },
            3 => Self::CloseProgramState {},
            4 => Self::TransferPickle {
                transfer_pickle_data: TransferPickleData::try_from_slice(rest).unwrap()
            },
            5 => Self::TransferSOL {
                transfer_sol_data: TransferSOLData::try_from_slice(rest).unwrap()
            },
            _ => return Err(McPayError::InvalidInstruction.into()),
        })
    }
}
