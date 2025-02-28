use bincode::Options;
use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState, PessimisticProofOutput};

use risc0_zkvm::guest::env;

pub fn main() {
    // Read the input states from the host
    let initial_state: NetworkState = env::read();
    let batch_header: MultiBatchHeader<Keccak256Hasher> = env::read();

    // Generate the proof
    let outputs = generate_pessimistic_proof(initial_state, &batch_header)
        .expect("Failed to generate pessimistic proof");

    // Commit the outputs to the journal
    env::commit(&outputs);
}