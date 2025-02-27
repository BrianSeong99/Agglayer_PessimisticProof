#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

use bincode::Options;
use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState, PessimisticProofOutput};


use nexus_rt::{println, read_private_input, write_output};

#[nexus_rt::main]
fn main() {
    let input = read_private_input::<(NetworkState, MultiBatchHeader<Keccak256Hasher>)>();

    // Generate the proof
    let outputs = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    write_output::<PessimisticProofOutput>(&outputs)
}