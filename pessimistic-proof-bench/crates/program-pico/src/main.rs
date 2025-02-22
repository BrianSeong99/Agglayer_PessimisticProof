#![no_main]

use bincode::Options;
use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState, PessimisticProofOutput};
use valida_rs::io::InputTape;
use serde::Deserialize;

valida_rs::entrypoint!(main);

#[no_mangle]

pub fn main() {
    // Use serde_json deserializer with InputTape for reading input
    let mut de = serde_json::Deserializer::from_reader(InputTape);
    let input: ProofInput = ProofInput::deserialize(&mut de).expect("could not parse input");

    // Generate the proof
    let outputs = generate_pessimistic_proof(input.initial_state, &input.batch_header).unwrap();

    // Serialize the outputs
    let pp_inputs = PessimisticProofOutput::bincode_options()
        .serialize(&outputs)
        .unwrap();

    // Print the results
    println!("Proof generated successfully");
    println!("Output size: {} bytes", pp_inputs.len());
}

#[derive(Deserialize)]
pub struct ProofInput {
    initial_state: NetworkState,
    batch_header: MultiBatchHeader<Keccak256Hasher>,
}