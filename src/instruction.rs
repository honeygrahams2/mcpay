use borsh::{
    BorshDeserialize, 
    BorshSerialize
};
use solana_program::program_error::ProgramError;

use crate::error::McPayError;
use crate::state::ClockInData;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum McPayInstruction {
    ClockIn {
        clock_in_data: ClockInData,
    },
}

impl McPayInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(McPayError::InvalidInstruction)?;
        match tag {
            0 => McPayInstruction::clock_in(rest),
            _ => Err(McPayError::InvalidInstruction.into()),
        }
    }

    fn clock_in (input: &[u8]) -> Result<Self, ProgramError> {
        let clock_in_data = ClockInData::try_from_slice(input).unwrap();

        Ok(McPayInstruction::ClockIn {                          
            clock_in_data,
        })
    }
}
