// Rustreexo

use std::collections::HashMap;

use bitcoin::hashes::sha256;

use super::types;

// TODO maybe there is a better rusty way of doing this...
pub const EMPTY: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Pollard is the sparse representation of the utreexo forest
/// It is a collection of multitude of trees with leaves that are
/// power of two.
///
/// However, the allocated tree is always a power of two. The nodes
/// that are not necessary are kept as empty nodes.
///
/// Its structure resembles of that of a binary tree, except that
/// the pointers point to aunts - nieces, not parents - children
pub struct Pollard {
    /// Roots are the top-most nodes of the tree
    /// There may be multiple roots as Utreexo is organized as a
    /// collection of perfect trees.
    pub roots: Vec<PolNode>,

    /// CachedLeaves are the cached leaves to the LeafData.
    /// This is done for efficiency purposes as the utxos that
    /// are going to be spent soon doesn't have to be redownloaded
    /// saving networking usage
    pub cached_leaves: HashMap<types::Leaf, types::LeafData>,
}

impl Pollard {
    /*
    /// Modify changes the Utreexo tree state given the utxos and stxos
    /// stxos are denoted by their value
    pub fn Modify(&self, utxos: Vec<types::Leaf>, stxos: Vec<u64>) -> Result<(), u8> {
        self.Remove(stxos);

        let err = self.Add(utxos).unwrap();
    }

    //
    fn Add(&self, adds: Vec<types::Leaf>) -> Result {
        for add in adds {
            if add.Remember {
                // TODO Should cache the add data
            }
            let err = AddSingle().unwrap();
            Err(err);
        }
    }

    // AddSingle adds a single given utxo to the tree
    fn AddSingle(&self, utxo: [u8; 32], remember: bool) -> Result {}

    fn Remove() -> Result {}
    */
}

/// polNode represents a node in the utreexo pollard tree. It points
/// to its nieces
pub struct PolNode {
    pub data: [u8; 32],
    pub niece: [Box<PolNode>; 2],
}

impl PolNode {
    /// aunt_op returns the hash of a nodes' nieces. Errors if called on nieces
    /// that are nil.
    fn aunt_op(&self) -> sha256::Hash {
        types::parent_hash(self.niece[0].data, self.niece[1].data)
    }

    /*
    // auntable returns if aunt_op is callable on this PolNode
    fn auntable(&self) -> bool {
        self.niece[0] != nil && self.niece[1] != EMPTY;
    }
    */
}
