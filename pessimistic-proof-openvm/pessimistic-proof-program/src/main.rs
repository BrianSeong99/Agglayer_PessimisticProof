use bincode::Options;
use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof::{generate_pessimistic_proof, LocalNetworkState, PessimisticProofOutput};

use openvm::io::{read, reveal};

openvm::entry!(main);

use {
    openvm_bigint_guest, // trigger extern u256 (this may be unneeded)
    openvm_keccak256_guest, // trigger extern native-keccak256
};

pub fn main() {
    let initial_state: LocalNetworkState = read();
    let batch_header: MultiBatchHeader<Keccak256Hasher> = read();

    let outputs = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_options()
        .serialize(&outputs)
        .unwrap();

    for (index, &element) in pp_inputs.iter().enumerate() {
        reveal(element as u32, index);
    }
}
