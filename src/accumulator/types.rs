// Rustreexo

use sha2::Sha256;

use bitcoin::blockdata::transaction;

type Hash = [u8; 32];

/// Leaf represents a utxo in the utreexo tree. These are the bottommost
/// nodes in the tree.
struct Leaf {
    ///  Hash is the
    Hash: Hash,

    /// Remember is whether or not the UTXO this Leaf represents should
    /// be cached or not.
    Remember: bool,
}

/// LeafData is all the data that goes into the hashing the leaf.
/// The data included is needed for transaction script validation.
/// The rest of the data is for hardening against hash collisions.
struct LeafData {
    BlockHeader: Hash,
    Outpoint: transaction::OutPoint,
    Height: i32,
    isCoinBase: bool,
    Amt: i64,
    PkScript: Vec<u8>,
}

/// Arrow is used to describe the movement of a leaf to a different
/// position. This is used for batch deletions during removal
struct Arrow {
    from: u64,
    to: u64,
}

// parent_hash return the merkle parent of the two passed in nodes
fn parent_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    if left == EMPTY || right == EMPTY {
        panic("emtpy niece")
    }

    // TODO there must be a better way to do this...
    let mut hasher = Sha256::new();
    hasher.update([first, second].concat());
    // read hash digest and consume hasher
    let result = hasher.finalize();

    return result;
}
