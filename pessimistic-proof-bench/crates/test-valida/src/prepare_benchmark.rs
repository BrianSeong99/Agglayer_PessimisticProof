use std::path::PathBuf;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::NetworkState;
use pessimistic_proof_test_suite::sample_data as data;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BenchmarkInput {
    pub initial_state: NetworkState,
    pub multi_batch_header: MultiBatchHeader,
}

pub fn prepare_benchmark_input(n_exits: usize, n_imported_exits: usize) -> BenchmarkInput {
    // Setup initial state
    let mut state = data::sample_state_00();
    let old_state = state.state_b.clone();

    // Generate events based on input parameters
    let bridge_exits = data::sample_bridge_exits_01()
        .cycle()
        .take(n_exits)
        .map(|e| (e.token_info, e.amount))
        .collect::<Vec<_>>();

    let imported_bridge_exits = data::sample_bridge_exits_01()
        .cycle()
        .take(n_imported_exits)
        .map(|e| (e.token_info, e.amount))
        .collect::<Vec<_>>();

    // Apply events and generate certificate
    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);
    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = old_state
        .make_multi_batch_header(&certificate, state.get_signer(), l1_info_root)
        .unwrap();

    BenchmarkInput {
        initial_state: old_state.into(),
        multi_batch_header,
    }
}

pub fn save_benchmark_input(input: &BenchmarkInput, path: &PathBuf) -> std::io::Result<()> {
    std::fs::write(
        path,
        serde_json::to_string(input).unwrap(),
    )
}