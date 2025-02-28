use bincode::config::DefaultOptions;
use bincode::Options;
use openvm::io::{read_vec, reveal};
use pessimistic_proof_core::{
    generate_pessimistic_proof, local_exit_tree::hasher::Keccak256Hasher,
    multi_batch_header::MultiBatchHeader, NetworkState,
};

fn main() {
    let config = DefaultOptions::new();
    let input_bytes = read_vec();

    let (initial_state, offset): (NetworkState, usize) = config
        .deserialize_from(&input_bytes[..])
        .expect("Failed to deserialize NetworkState");
    let (batch_header, _): (MultiBatchHeader<Keccak256Hasher>, usize) = config
        .deserialize_from(&input_bytes[offset..])
        .expect("Failed to deserialize MultiBatchHeader");

    let outputs =
        generate_pessimistic_proof(initial_state, &batch_header).expect("Failed to generate proof");

    let output_bytes = config
        .serialize(&outputs)
        .expect("Failed to serialize output");

    for (i, chunk) in output_bytes.chunks(4).enumerate() {
        let mut bytes = [0u8; 4];
        bytes[..chunk.len()].copy_from_slice(chunk);
        let value = u32::from_le_bytes(bytes);
        reveal(value, i);
    }
}
