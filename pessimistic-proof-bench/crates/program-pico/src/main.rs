#![no_main]

use bincode::Options;
use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState, PessimisticProofOutput};

pico_sdk::entrypoint!(main);
use pico_sdk::io::{commit_bytes, read_as};

pub fn main() {
    // Read the input states
    let initial_state = read_as::<NetworkState>();
    let batch_header = read_as::<MultiBatchHeader<Keccak256Hasher>>();

    // Generate the proof
    let outputs = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    // Serialize the outputs
    let pp_inputs = PessimisticProofOutput::bincode_options()
        .serialize(&outputs)
        .unwrap();

    // Commit the result
    commit_bytes(&pp_inputs);
}