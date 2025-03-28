use solana_program::program_error::ProgramError;
use crate::error::FlashloanArbitrageError::{InvalidInstruction, InstructionUnpackError};
use std::{convert::TryInto, mem::size_of};

pub enum FlashloanArbitrageInstruction {
    /// Initializes the flash loan arbitrage program account
    ///
    /// Accounts expected:
    /// 0. `[signer]` The account of the person initializing the program
    /// 1. `[writable]` Program's token account for holding flash loan funds (created prior to this instruction)
    /// 2. `[writable]` The program's account to hold state
    /// 3. `[]` The rent sysvar
    /// 4. `[]` The token program
    /// 5. `[]` Buy DEX program ID (e.g., Raydium)
    /// 6. `[]` Sell DEX program ID (e.g., Orca)
    InitFlashloanArbitrage,

    /// Executes the arbitrage operation after receiving a flash loan
    ///
    /// Accounts expected:
    /// 0. `[]` Lending program ID
    /// 1. `[]` Token program ID
    /// 2. `[writable]` Program's state account
    /// 3. `[writable]` Program's token account (to approve transfer)
    /// 4. `[]` Buy DEX program ID (e.g., Raydium)
    /// 5. `[]` Sell DEX program ID (e.g., Orca)
    /// 6. `[writable]` Buy pool address
    /// 7. `[writable]` Sell pool address
    /// 8. `[writable]` Profit wallet
    ExecuteOperation {
        amount: u64, // Amount borrowed
    },

    /// Requests a flash loan and executes arbitrage
    ///
    /// Accounts expected:
    /// 0. `[writable]` Destination liquidity token account (program's token account)
    /// 1. `[writable]` Borrow reserve account
    /// 2. `[writable]` Borrow reserve liquidity supply SPL Token account
    /// 3. `[]` Lending market account
    /// 4. `[]` Derived lending market authority
    /// 5. `[]` Buy DEX program ID (e.g., Raydium)
    /// 6. `[]` Sell DEX program ID (e.g., Orca)
    /// 7. `[writable]` Buy pool address
    /// 8. `[writable]` Sell pool address
    /// 9. `[writable]` Profit wallet
    /// 10. `[]` Token program ID
    /// 11. `[]` Lending program ID
    FlashloanArbitrage {
        amount: u64, // Amount to borrow
        execute_operation_ix_data: Vec<u8>, // Data for the execute operation instruction
        expected_profit: u64, // Minimum expected profit to ensure the trade is worthwhile
    },
}

impl FlashloanArbitrageInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitFlashloanArbitrage,
            1 => {
                let (amount, _rest) = Self::unpack_u64(rest)?;
                Self::ExecuteOperation { amount }
            },
            2 => {
                let (amount, rest) = Self::unpack_u64(rest)?;
                let (expected_profit, execute_operation_ix_data_slice) = Self::unpack_u64(rest)?;
                let execute_operation_ix_data = execute_operation_ix_data_slice.to_vec();
                Self::FlashloanArbitrage {
                    amount,
                    execute_operation_ix_data,
                    expected_profit,
                }
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() >= 8 {
            let (amount, rest) = input.split_at(8);
            let amount = amount
                .get(..8)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(InstructionUnpackError)?;
            Ok((amount, rest))
        } else {
            Err(InstructionUnpackError.into())
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self { // Changed from `match *self` to `match self`
            Self::InitFlashloanArbitrage => {
                buf.push(0);
            }
            Self::ExecuteOperation { amount } => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::FlashloanArbitrage {
                amount,
                execute_operation_ix_data,
                expected_profit,
            } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&expected_profit.to_le_bytes());
                buf.extend_from_slice(execute_operation_ix_data);
            }
        }
        buf
    }
}