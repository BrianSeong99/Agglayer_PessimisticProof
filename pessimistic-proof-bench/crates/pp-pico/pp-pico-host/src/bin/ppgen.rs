use std::{path::PathBuf, time::Instant};
use pico_sdk::{client::DefaultProverClient, init_logger};
use agglayer_types::{Certificate, U256};
use clap::Parser;
use pessimistic_proof::bridge_exit::{NetworkId, TokenInfo};
use pessimistic_proof::PessimisticProofOutput;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState};
use pessimistic_proof_test_suite::sample_data::{self as data};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

/// The arguments for the pp generator.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct PPGenArgs {
    /// The number of bridge exits.
    #[clap(long, default_value = "10")]
    n_exits: usize,

    /// The number of imported bridge exits.
    #[clap(long, default_value = "10")]
    n_imported_exits: usize,

    /// The optional output directory to write the proofs in JSON. If not set,
    /// the proof is simply logged.
    #[clap(long)]
    proof_dir: Option<PathBuf>,

    /// The optional path to the custom sample data.
    #[clap(long)]
    sample_path: Option<PathBuf>,
}

fn get_events(n: usize, path: Option<PathBuf>) -> Vec<(TokenInfo, U256)> {
    if let Some(p) = path {
        data::sample_bridge_exits(p)
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    } else {
        data::sample_bridge_exits_01()
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    }
}

// fn verify_proof(proof_output: &PessimisticProofOutput, public_values: &PublicValuesStruct) -> bool {
//     let mut proof_output = proof_output.clone();
//     assert_eq!(proof_output.prev_local_exit_root, public_values.prev_local_exit_root);
//     assert_eq!(proof_output.prev_pessimistic_root, public_values.prev_pessimistic_root);
//     assert_eq!(proof_output.l1_info_root, public_values.l1_info_root);
//     assert_eq!(proof_output.origin_network, public_values.origin_network);
//     assert_eq!(proof_output.new_local_exit_root, public_values.new_local_exit_root);
//     assert_eq!(proof_output.new_pessimistic_root, public_values.new_pessimistic_root);
//     true
// }

pub fn main() {
    // Initialize logger
    init_logger();

    let args = PPGenArgs::parse();

    // Load the ELF file
    let elf = std::fs::read("pp-pico-guest/elf/riscv32im-pico-zkvm-elf").expect("Failed to load ELF file");
    info!("Loaded ELF file, length: {}", elf.len());

    let mut state = data::sample_state_00();
    let old_state = state.state_b.clone();
    let old_network_state = NetworkState::from(old_state.clone());

    let bridge_exits = get_events(args.n_exits, args.sample_path.clone());
    let imported_bridge_exits = get_events(args.n_imported_exits, args.sample_path);

    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);

    // info!(
    //     "Certificate {}: [{}]",
    //     certificate.hash(),
    //     serde_json::to_string(&certificate).unwrap()
    // );

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = old_state
        .make_multi_batch_header(&certificate, state.get_signer(), l1_info_root)
        .unwrap();


    // Validate inputs by running generate_pessimistic_proof first
    // let mut proof_output: PessimisticProofOutput = None;
    match generate_pessimistic_proof(old_network_state.clone(), &multi_batch_header) {
        Ok(output) => {
            // proof_output = output;
            info!("Input validation successful, proceeding with proof generation");
        }
        Err(e) => {
            panic!("Input validation failed: {:?}", e);
        }
    }

    info!(
        "Generating the proof for {} bridge exit(s) and {} imported bridge exit(s)",
        bridge_exits.len(),
        imported_bridge_exits.len()
    );

    // Initialize the prover client
    let client = DefaultProverClient::new(&elf);
    let stdin_builder = client.get_stdin_builder();

    // Write inputs to the VM
    stdin_builder.borrow_mut().write(&old_network_state);
    stdin_builder.borrow_mut().write(&multi_batch_header);

    let start = Instant::now();
    let proof = client.prove_fast().expect("proving failed");
    let duration = start.elapsed();
    info!(
        "Successfully generated the proof with a latency of {:?}",
        duration
    );

    // // Decodes public values from the proof's public value stream.
    // let public_buffer = proof.pv_stream.unwrap();
    // let public_values = PublicValuesStruct::abi_decode(&public_buffer, true).unwrap();

    // verify_proof(&proof_output, &public_values);
}
