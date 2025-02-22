#![no_main]

use valida_rs::io::InputTape;
use serde::Deserialize;
use pessimistic_proof::NetworkState;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::generate_pessimistic_proof;


#[derive(Deserialize)]
struct BenchmarkInput {
    initial_state: NetworkState,
    multi_batch_header: MultiBatchHeader,
}

#[no_mangle]
fn main() {
    // Read prepared benchmark input
    let mut de = serde_json::Deserializer::from_reader(InputTape);
    let input: BenchmarkInput = BenchmarkInput::deserialize(&mut de)
        .expect("could not parse benchmark input");

    // Generate proof
    let outputs = generate_pessimistic_proof(
        input.initial_state,
        &input.multi_batch_header
    ).expect("proving failed");

    // Print minimal output to verify execution
    println!("Proof generation completed");
    println!("Output roots: {} {}", 
        hex::encode(&outputs.prev_local_exit_root),
        hex::encode(&outputs.new_local_exit_root)
    );
}