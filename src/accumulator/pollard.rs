// Rustreexo

use std::collections::HashMap;

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
    pub Roots: Vec<PolNode>,

    /// CachedLeaves are the cached leaves to the LeafData.
    /// This is done for efficiency purposes as the utxos that
    /// are going to be spent soon doesn't have to be redownloaded
    /// saving networking usage
    pub CachedLeaves: HashMap<Leaf, LeafData>,
}

impl Pollard {
    /// Modify changes the Utreexo tree state given the utxos and stxos
    /// stxos are denoted by their value
    pub fn Modify(&self, utxos: Vec<Leaf>, stxos: Vec<u64>) -> Option<&str> {
        self.Remove(stxos);

        self.Add(utxos);
    }

    //
    fn Add(&self) -> Result {
        for add in adds {
            if add.Remember {
                // TODO Should cache the add data
            }
            err = AddSingle().unwrap();
        }
        return err;
    }

    // AddSingle adds a single given utxo to the tree
    fn AddSingle(&self, utxo: [u8; 32], remember: bool) -> Result {}

    fn Remove() -> Result {}
}

/// PolNode represents a node in the utreexo pollard tree. It points
/// to its nieces
struct PolNode {
    pub data: [u8; 32],
    pub niece: [polNode; 2],
}

impl PolNode {
    /// aunt_op returns the hash of a nodes' nieces. Errors if called on nieces
    /// that are nil.
    fn aunt_op(&self) -> [u8; 32] {
        return parent_hash(self.niece[0].data, n.niece[1].data);
    }

    // auntable returns if aunt_op is callable on this PolNode
    fn auntable(&self) -> bool {
        return n.niece[0] != nil && n.niece[1] != nil;
    }
}
