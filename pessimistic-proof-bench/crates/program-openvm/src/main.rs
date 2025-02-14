use bincode::Options;
use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState, PessimisticProofOutput};

use openvm::io::{read, reveal};

openvm::entry!(main);

pub fn main() {
    let initial_state: NetworkState = read();
    let batch_header: MultiBatchHeader<Keccak256Hasher> = read();

    let outputs = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_options()
        .serialize(&outputs)
        .unwrap();

    for (index, &element) in pp_inputs.iter().enumerate() {
        reveal(element as u32, index);
    }
}
