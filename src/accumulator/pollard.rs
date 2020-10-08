// Rustreexo

use std::collections::HashMap;

use bitcoin::hashes::sha256::{self, Hash};

use super::types;

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

    /// Total number of leaves (nodes on the bottom row) in the Pollard
    pub num_leaves: u64,

    /// Maps PolNode Hashes to positions
    pub position_map: HashMap<sha256::Hash, u64>,

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
        // General algo goes:
        // 1 make a new node & assign data (no nieces; at bottom)
        // 2 if this node is on a row where there's already a root,
        // then swap nieces with that root, hash the two datas, and build a new
        // node 1 higher pointing to them.
        // goto 2.

        for add in adds {
            if add.Remember {
                // TODO Should cache the add data
            }
            let err = AddSingle().unwrap();
            Err(err);
        }
    }
    */

    // AddSingle adds a single given utxo to the tree
    fn add_single(mut self, utxo: sha256::Hash, remember: bool) {
        // Algo explanation with catchy terms: grab, swap, hash, new, pop
        // we're iterating through the roots of the pollard.  Roots correspond with 1-bits
        // in numLeaves.  As soon as we hit a 0 (no root), we're done.

        // grab: Grab the lowest root.
        // pop: pop off the lowest root.
        // swap: swap the nieces of the node we grabbed and our new node
        // hash: calculate the hashes of the old root and new node
        // new: create a new parent node, with the hash as data, and the old root / prev new node
        // as nieces (not nieces though, children)

        // basic idea: you're going to start at the LSB and move left;
        // the first 0 you find you're going to turn into a 1.
        // make the new leaf & populate it with the actual data you're trying to add
        let mut n = Box::new(PolNode {
            data: utxo,
            nieces: [None, None],
        });

        // FIXME This is ugly. It's currently broken in the reference go code anyways
        //if remember {
        //    // flag this leaf as memorable via it's left pointer
        //    n.nieces[0] = Some(n.clone()); // points to itself (mind blown)
        //}

        // if add is forgetable, forget all the new nodes made
        let mut h: u8 = 0;

        // loop until we find a zero; destroy roots until you make one
        while (self.num_leaves >> h) & 1 == 1 {
            // grab, pop, swap, hash, new
            let mut left_root = self.roots.pop().unwrap();

            //if h == 0 && remember {
            //        // make sure that siblings are always remembered in pairs.
            //        left_root.unwrap().nieces[0] = Some(Box::new(left_root.clone().unwrap()));
            //}

            let tmp = n.nieces;
            n.nieces = left_root.clone().nieces;
            left_root.nieces = tmp;//[Some(tmp[0].unwrap()), Some(tmp[1].unwrap())];
            //left_root.unwrap_or.nieces = tmp;
            //left_root.niece, n.niece = n.niece, leftRoot.niece; // swap

            let n_hash = types::parent_hash(&left_root.data, &n.data); // hash

            n = Box::new(PolNode {
                data: n_hash,
                nieces: [Some(Box::new(left_root)), Some(n)],
            }); // new

            //n.prune();
            h += 1;
        }

        // the new roots are all the 1 bits above where we got to, and nothing below where
        // we got to.  We've already deleted all the lower roots, so append the new
        // one we just made onto the end.

        self.roots.push(*n);
        self.num_leaves += 1;
    }

    //fn Remove() -> Result {}
}

/// polNode represents a node in the utreexo pollard tree. It points
/// to its nieces
#[derive(Clone)]
pub struct PolNode {
    pub data: sha256::Hash,
    //pub data: sha256::Hash,

    pub nieces: [Option<Box<PolNode>>; 2],
}

impl PolNode {
    /// aunt_op returns the hash of a nodes' nieces. Errors if called on nieces
    /// that are nil.
    fn aunt_op(&self) -> sha256::Hash {
        types::parent_hash(&self.nieces[0].as_ref().unwrap().data, &self.nieces[1].as_ref().unwrap().data)
    }
}
