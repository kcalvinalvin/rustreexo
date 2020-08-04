// Rustreexo

use sha2::Sha256;

type Hash = [u8; 32];

struct Leaf {
    ///  Hash
    Hash: Hash,

    /// Remember is whether or not the UTXO this Leaf represents should
    /// be cached or not.
    Remember: bool,
}

struct Arrow {
    from: u64,
    to: u64,
}

// parent_hash return the merkle parent of the two passed in nodes
fn parent_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    if left == EMPTY | right == EMPTY {
        panic("emtpy niece")
    }

    // TODO there must be a better way to do this...
    let mut hasher = Sha256::new();
    hasher.update([first, second].concat());
    // read hash digest and consume hasher
    let result = hasher.finalize();

    return result;
}
