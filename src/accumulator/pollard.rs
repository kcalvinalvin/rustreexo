// Rustreexo

use std::collections::HashMap;
use std::mem;

use super::{
    types,
    util,
    transform
};

use bitcoin::hashes::{sha256, Hash, HashEngine};

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
    /// Modify changes the Utreexo tree state given the utxos and stxos
    /// stxos are denoted by their value
    pub fn modify(&mut self, utxos: Vec<types::Leaf>, stxos: Vec<u64>) {
        Pollard::remove(self, stxos);
        Pollard::add(self, utxos);
    }


    pub fn add(&mut self, adds: Vec<types::Leaf>) {
        // General algo goes:
        // 1 make a new node & assign data (no nieces; at bottom)
        // 2 if this node is on a row where there's already a root,
        // then swap nieces with that root, hash the two datas, and build a new
        // node 1 higher pointing to them.
        // goto 2.

        for add in adds {
            //if add.remember {
            //    // TODO Should cache the add data
            //}
            Pollard::add_single(self, add.hash, false);
        }
    }

    // AddSingle adds a single given utxo to the tree
    fn add_single(&mut self, utxo: sha256::Hash, remember: bool) {
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

    fn remove(&mut self, dels: Vec<u64>) {
        // if there is nothing to delete, return
        if dels.len() == 0 {
            return
        }

        let pollard_rows = util::tree_rows(self.num_leaves);

        let leaves_after_del = self.num_leaves - dels.len() as u64;

        // get all the swaps, then apply them all
        let swap_rows = transform::transform(dels, self.num_leaves, pollard_rows);

        let hash_dirt: Vec<u64>;
        let next_hash_dirt: Vec<u64>;
        let prev_hash: u64;

        for row in 0..pollard_rows {
        }

    }

    fn grab_pos(&mut self, pos: u64) -> Option<(PolNode, PolNode, HashableNode)> {
        // Grab the tree that the position is at
        let (tree, branch_len, bits) = util::detect_offset(pos, self.num_leaves);
            if tree as usize >= self.roots.len() {
                return None
            }

            if branch_len == 0 {
                let node = self.roots[tree as usize].clone();
                let node_sib = self.roots[tree as usize].clone();

                let hn = HashableNode {
                    sib: node.clone(), dest: node_sib.clone(), position: pos
                };

                return Some((node, node_sib, hn))
            }

            let mut node = Some(&self.roots[tree as usize]);
            let mut node_sib = Some(&self.roots[tree as usize]);

            for h in (0+1..branch_len).rev() {
                let lr = bits>>h & 1;

                // grab the sibling of lr
                let lr_sib = lr ^ 1;

                // if a sib doesn't exist, need to create it and hook it in
                //if n.nieces[lr_sib].is_none() {
                //   n.nieces[lr_sib] = Box::polNode{}
                //}
                let n = &node.unwrap().nieces[lr as usize].as_ref().unwrap();
                node = Some(n);
                //node = Some(&node.unwrap().nieces[lr as usize].unwrap());
                node_sib = Some(&node.unwrap().nieces[lr_sib as usize].as_ref().unwrap());

                if node.is_none() {
                    // if a node doesn't exist, crash
                    // no niece in this case
                    // TODO error message could be better
                    return None;
                }

            }

            let lr = bits & 1;

            // grab the sibling of lr
            let lr_sib = lr ^ 1;

            let hn = Some(HashableNode{
                sib: node.unwrap().clone(),
                dest: node_sib.unwrap().clone(),
                position: pos
            });
            //hn.unwrap().dest = *node_sib.unwrap(); // this is kind of confusing eh?
            //hn.unwrap().sib = *node.unwrap();     // but yeah, switch siblingness
            let n = node.unwrap().nieces[lr_sib as usize].clone();
            let nsib = node.unwrap().nieces[lr as usize].clone();

            Some((*n.unwrap(), *nsib.unwrap(), hn.unwrap()))
        }

    fn swap_nodes(&mut self, swaps: types::Arrow, row: u8) -> HashableNode {
        if !util::in_forest(swaps.from, self.num_leaves, util::tree_rows(self.num_leaves)) ||
            !util::in_forest(swaps.to, self.num_leaves, util::tree_rows(self.num_leaves)) {
        }
        let (a, asib, _) = self.grab_pos(swaps.from).unwrap();

        // currently swaps the "values" instead of changing what parents point
        // // to.  Seems easier to reason about but maybe slower?  But probably
        // // doesn't matter that much because it's changing 8 bytes vs 30-something
        //
        // // TODO could be improved by getting the highest common ancestor
        // // and then splitting instead of doing 2 full descents
        // TODO (rust): Actually unwrap and check if None
        let(mut a, mut asib, _) = self.grab_pos(swaps.from).unwrap();
        let(mut b, mut bsib, mut bhn) = self.grab_pos(swaps.to).unwrap();

        let position = util::parent(swaps.to, util::tree_rows(self.num_leaves));
        bhn.position = position;

        // do the actual swap here
        pol_swap(&mut a, &mut asib, &mut b, &mut bsib);


        return bhn;
    }
}

/// PolNode represents a node in the utreexo pollard tree. It points
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

    fn dead_end(&self) -> bool {
        self.nieces[0].is_none() && self.nieces[1].is_none()
    }

    fn chop(&mut self) {
        self.nieces[0] = None;
        self.nieces[1] = None;
    }

    fn prune(&mut self) {
        if self.nieces[0].clone().unwrap().dead_end() {
            self.nieces[0] = None;
        }

        if self.nieces[1].clone().unwrap().dead_end() {
            self.nieces[1] = None;
        }
    }
}

//// hashableNode is the data needed to perform a hash
//pub struct HashableNode<'a> {
//    sib: &'a PolNode,
//    dest: &'a PolNode,
//    position: u64 // doesn't really need to be there, but convenient for debugging
//}

// hashableNode is the data needed to perform a hash
pub struct HashableNode {
    pub sib: PolNode,
    pub dest: PolNode,
    pub position: u64 // doesn't really need to be there, but convenient for debugging
}

// polSwap swaps the contents of two polNodes & leaves pointers
fn pol_swap<'a, 'b>(mut a: &'a mut PolNode, mut asib: &'b mut PolNode, mut b: &'a mut PolNode, mut bsib: &'b mut PolNode) {
    mem::swap(&mut a, &mut b);
    mem::swap(&mut asib,&mut bsib);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pol_swap() {
        use bitcoin::hashes::{sha256, Hash, HashEngine};
        use std::mem;

        let mut engine = bitcoin::hashes::sha256::Hash::engine();
        let num: &[u8; 1] = &[1 as u8];
        engine.input(num);
        let h1 = sha256::Hash::from_engine(engine);
        let h1_copy = h1.clone();

        let mut engine1 = bitcoin::hashes::sha256::Hash::engine();
        let num2: &[u8; 1] = &[2 as u8];
        engine1.input(num2);
        let h2 = sha256::Hash::from_engine(engine1);
        let h2_copy = h2.clone();

        let mut engine2 = bitcoin::hashes::sha256::Hash::engine();
        let num3: &[u8; 1] = &[3 as u8];
        engine2.input(num3);
        let h3 = sha256::Hash::from_engine(engine2);
        let h3_copy = h3.clone();

        let mut engine3 = bitcoin::hashes::sha256::Hash::engine();
        let num4: &[u8; 1] = &[3 as u8];
        engine3.input(num4);
        let h4 = sha256::Hash::from_engine(engine3);
        let h4_copy = h4.clone();

        let mut a = super::PolNode{
            data: h1,
            nieces: [None, None],
        };

        assert_eq!(a.data, h1_copy); // sanity

        let mut b = super::PolNode{
            data: h2,
            nieces: [None, None],
        };

        assert_eq!(b.data, h2_copy); // sanity

        let mut asib = super::PolNode{
            data: h3,
            nieces: [None, None],
        };

        let mut bsib = super::PolNode{
            data: h4,
            nieces: [None, None],
        };

        super::pol_swap(&mut a, &mut b, &mut asib, &mut bsib);

        assert_eq!(a.data, h1_copy);
        assert_eq!(b.data, h2_copy);

        assert_eq!(asib.data, h3_copy);
        assert_eq!(bsib.data, h4_copy);

        mem::swap(&mut a, &mut b);

        assert_eq!(a.data, h2_copy);
        assert_eq!(b.data, h1_copy);
    }
}
