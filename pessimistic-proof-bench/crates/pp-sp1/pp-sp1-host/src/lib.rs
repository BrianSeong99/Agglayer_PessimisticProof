//! A collection of shared testing utilities.

pub mod runner;

/// The ELF we want to execute inside the zkVM.
pub const PESSIMISTIC_PROOF_ELF: &[u8] =
    include_bytes!("../../pp-sp1-guest/elf/riscv32im-succinct-zkvm-elf");
