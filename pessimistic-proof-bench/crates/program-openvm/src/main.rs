#![no_std]
#![no_main]

use openvm_sdk::io::{read, reveal};

use bincode::Options;
use pessimistic_proof_core::{
    local_exit_tree::hasher::Keccak256Hasher,
    multi_batch_header::MultiBatchHeader,
    NetworkState,
    generate_pessimistic_proof,
    PessimisticProofOutput,
};

#[no_mangle]
pub fn main() {
    // Read inputs using read
    let initial_state: NetworkState = bincode::deserialize(&read()).unwrap();
    let batch_header: MultiBatchHeader<Keccak256Hasher> = bincode::deserialize(&read()).unwrap();

    // Generate the proof
    let outputs = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    // Serialize and output the result
    let output_bytes = bincode::serialize(&outputs).unwrap();
    openvm_sdk::io::write(&output_bytes);
}