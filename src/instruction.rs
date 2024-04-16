use borsh::{
    BorshDeserialize, 
    BorshSerialize
};
use solana_program::program_error::ProgramError;

use crate::error::McPayError;
use crate::state::{
    ClockInData, 
    ClockOutData,
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
            _ => return Err(McPayError::InvalidInstruction.into()),
        })
    }
}
