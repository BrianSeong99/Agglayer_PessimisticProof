use nexus_sdk::{
    compile::CompileOpts,
    nova::seq::{Generate, Nova, PP},
    Local, Prover, Verifiable
};
use std::{path::PathBuf, time::Instant};
use tracing::{info, warn};
use tracing_subscriber::{self, EnvFilter};
use clap::Parser;

use agglayer_types::{Certificate, U256};
use pessimistic_proof::bridge_exit::{NetworkId, TokenInfo};
use pessimistic_proof::PessimisticProofOutput;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState};
use pessimistic_proof_test_suite::sample_data::{self as data};
use uuid::Uuid;
use hex;

const PACKAGE: &str = "pp-nexus-guest";

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

fn init_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();
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

fn main() {
    // Initialize logger
    init_logger();

    let args = PPGenArgs::parse();

    // Prepare the state and input data
    info!("Preparing initial state and input data...");
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

    // Validate inputs
    info!("Validating inputs...");
    match generate_pessimistic_proof(old_network_state.clone(), &multi_batch_header) {
        Ok(_) => info!("Input validation successful, proceeding with proof generation"),
        Err(e) => panic!("Input validation failed: {:?}", e),
    }

    info!(
        "Generating the proof for {} bridge exit(s) and {} imported bridge exit(s)",
        bridge_exits.len(),
        imported_bridge_exits.len()
    );
    
    info!("Setting up Nova public parameters...");
    let start = Instant::now();
    let pp: PP = PP::generate().expect("failed to generate parameters");
    info!(
        "Generated public parameters with latency of {:?}",
        start.elapsed()
    );

    let opts = CompileOpts::new(PACKAGE);
    // opts.set_memlimit(8); // use an 8mb memory

    info!("Compiling guest program...");
    let start = Instant::now();
    let prover: Nova<Local> = Nova::compile(&opts).expect("failed to compile guest program");
    info!(
        "Successfully compiled guest program with latency of {:?}",
        start.elapsed()
    );

    info!("Proving execution of vm...");
    let start = Instant::now();
    let input = (old_network_state, multi_batch_header);
    let proof = prover
        .prove_with_input(&pp, &input)
        .expect("failed to prove program");
    let output = proof
        .output::<PessimisticProofOutput>()
        .expect("failed to deserialize output");
    info!(
        "Successfully generated the proof with latency of {:?}",
        start.elapsed()
    );

    info!(">>>>> Program Logs\n{}<<<<<", proof.logs().join(""));

    info!("Verifying execution...");
    let start = Instant::now();
    match proof.verify(&pp) {
        Ok(_) => info!(
            "Verification succeeded with latency of {:?}",
            start.elapsed()
        ),
        Err(e) => panic!("Verification failed: {:?}", e),
    }

}
