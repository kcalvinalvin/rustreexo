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
    pub roots: Option<Vec<PolNode>>,

    /// Total number of leaves (nodes on the bottom row) in the Pollard
    pub num_leaves: u64,
}

impl Pollard {
    /// Returns a new pollard
    pub fn new() -> Pollard {
        Pollard{roots: None, num_leaves:0, }
    }

    /// Modify changes the Utreexo tree state given the utxos and stxos
    /// stxos are denoted by their value
    pub fn modify(&mut self, utxos: Vec<types::Leaf>, stxos: Vec<u64>) {
        // Order matters here. Adding then removing will result in a different
        // tree vs deleting then adding. For ease of use, only modify is visible
        // for external crates. This is consensus critical.
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
    // TODO activate caching (use remember). This isn't done in the
    // Go repo either yet
    fn add_single(&mut self, utxo: sha256::Hash, remember: bool) {

        // recurse from the right side of the tree until we hit a tree with no root
        // Destorys roots along the way
        fn add(pol: &mut Pollard, node: &mut PolNode, num_leaves: u64) -> PolNode{
            let mut return_node = node.clone();

            if num_leaves & 1 == 1 {
                match &mut pol.roots {
                    // if num_leaves & 1 is true, pol.roots can't be none
                    None => (),
                    Some(roots) => {
                        let before_len = roots.clone().len();
                        let mut left_root = roots.pop().unwrap();
                        assert_ne!(roots.clone().len(), before_len); // sanity

                        mem::swap(&mut left_root.l_niece, &mut node.l_niece);
                        mem::swap(&mut left_root.r_niece, &mut node.r_niece);

                        let n_hash = types::parent_hash(&left_root.data.clone(), &node.data.clone());
                        let new_node = &mut PolNode {
                            data: n_hash,
                            l_niece: Some(Box::new(left_root)),
                            r_niece: Some(Box::new(node.clone())),
                        };

                        new_node.prune();
                        return_node = add(pol, new_node, num_leaves>>1);
                    },
                }
            }

            return return_node;
        }

        // init node. If the Pollard is perfect (meaning only one root), this will become a
        // new root
        let mut node = &mut PolNode {
            data: utxo,
            l_niece: None,
            r_niece: None,
        };

        let add_node = add(self, &mut node, self.num_leaves);

        match &mut self.roots {
            None => {
                self.roots = Some(vec![add_node; 1]);
            },
            Some(root) => {
                root.push(add_node)
            }
        }

        // increment leaf count
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

        fn grab_niece(mut p_node: PolNode, mut p_node_sib: PolNode, branch_len: u8, bits: u64) -> Option<(PolNode, PolNode, u8)> {

            // calculate the left root. 0 means left_niece, 1 means right_niece
            let lr = bits>>branch_len & 1;

            // 0 is the left, 1 is the right as
            if lr == 0 {
                match &mut p_node.l_niece {
                    None => return None,
                    Some(niece) => {
                        p_node = *niece.clone()
                    }
                }

                match &mut p_node.r_niece {
                    None => return None,
                    Some(niece) => {
                        p_node_sib = *niece.clone()
                    }
                }
            } else {
                match &mut p_node.r_niece {
                    None => return None,
                    Some(niece) => {
                        p_node = *niece.clone()
                    }
                }

                match &mut p_node.l_niece {
                    None => return None,
                    Some(niece) => {
                        p_node_sib = *niece.clone()
                    }
                }
            }

            // Check if next recurse is gonna be at the bottom
            // Recurse as long as we're not at bottom
            if branch_len-1 != 0 {
                grab_niece(p_node.clone(), p_node_sib.clone(), branch_len-1, bits);
            }

            return Some((p_node, p_node_sib, branch_len-1));
        }

        // Grab the tree that the position is at
        let (tree, branch_len, bits) = util::detect_offset(pos, self.num_leaves);

        match &self.roots {
            None => (),
            Some(root) => {
                if tree as usize >= root.len() {
                    return None
                }

                let node = root[tree as usize].clone();
                let node_sib = root[tree as usize].clone();

                match grab_niece(node, node_sib, branch_len, bits) {
                    None => return None,
                    Some(res) => {
                        let hnode = HashableNode {
                            sib: Some(Box::new(res.0.clone() )),
                            dest: Some(Box::new(res.1.clone() )),
                            position: pos
                        };

                        return Some((res.0, res.1, hnode));

                    }
                }
            }
        }
        return None
    }
}

/// PolNode represents a node in the utreexo pollard tree. It points
/// to its nieces
#[derive(Clone)]
pub struct PolNode {
    // The hash
    pub data: sha256::Hash,

    pub l_niece: Option<Box<PolNode>>,
    pub r_niece: Option<Box<PolNode>>,
}

impl PolNode {
    /// aunt_op returns the hash of a nodes' nieces. Errors if called on nieces
    /// that are nil.
    fn aunt_op(&self) -> sha256::Hash {
        types::parent_hash(&self.l_niece.as_ref().unwrap().data, &self.r_niece.as_ref().unwrap().data)
    }

    fn dead_end(&self) -> bool {
        self.l_niece.is_none() && self.r_niece.is_none()
    }

    fn chop(&mut self) {
        self.l_niece = None;
        self.r_niece = None;
    }

    fn prune(&mut self) {
        match &mut self.l_niece {
            None => (),
            Some(node) =>  {
                if node.dead_end() {
                    node.chop()
                }
            }
        }

        match &mut self.r_niece {
            None => (),
            Some(node) =>  {
                if node.dead_end() {
                    node.chop()
                }
            }
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
    pub sib: Option<Box<PolNode>>,
    pub dest: Option<Box<PolNode>>,
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
    fn test_pol_add() {
        use bitcoin::hashes::{sha256, Hash, HashEngine};
        use super::types;

        let mut engine = bitcoin::hashes::sha256::Hash::engine();
        let num: &[u8; 1] = &[0 as u8];
        engine.input(num);
        let h1 = sha256::Hash::from_engine(engine);
        let leaf1 = types::Leaf{hash: h1, remember: false};

        let mut engine1 = bitcoin::hashes::sha256::Hash::engine();
        let num2: &[u8; 1] = &[1 as u8];
        &engine1.input(num2);
        let h2 = sha256::Hash::from_engine(engine1);
        let leaf2 = types::Leaf{hash: h2, remember: false};

        let mut engine3 = bitcoin::hashes::sha256::Hash::engine();
        let num3: &[u8; 1] = &[2 as u8];
        engine3.input(num3);
        let h3 = sha256::Hash::from_engine(engine3);
        let leaf3 = types::Leaf{hash: h3, remember: false};

        let mut engine4 = bitcoin::hashes::sha256::Hash::engine();
        let num4: &[u8; 1] = &[3 as u8];
        engine4.input(num4);
        let h4 = sha256::Hash::from_engine(engine4);
        let leaf4 = types::Leaf{hash: h4, remember: false};

        let mut engine5 = bitcoin::hashes::sha256::Hash::engine();
        let num5: &[u8; 1] = &[4 as u8];
        engine5.input(num5);
        let h5 = sha256::Hash::from_engine(engine5);
        let leaf5 = types::Leaf{hash: h5, remember: false};

        let mut pollard = super::Pollard::new();

        &pollard.modify(vec![leaf1], vec![]);

        assert_eq!(pollard.num_leaves, 1);
        assert_eq!(pollard.roots.clone().unwrap()[0].data, h1);

        &pollard.modify(vec![leaf2], vec![]);
        assert_ne!(pollard.roots.clone().unwrap()[0].data, h1);
        assert_ne!(pollard.roots.clone().unwrap()[0].data, h2);

        let root0 = pollard.roots.clone().unwrap()[0].data;
        //println!("h1: {:?}", h1);
        //println!("h2: {:?}", h2);
        //println!("root[0]: {:?}", pollard.roots.clone().unwrap()[0].data);
        //println!("pollard leaves: {:?}", &pollard.num_leaves);
        //println!("pollard root len: {:?}", pollard.roots.clone().unwrap().len());

        &pollard.modify(vec![leaf3], vec![]);

        println!("root[0]: {:?}", pollard.roots.clone().unwrap()[0].data);
        println!("root[1]: {:?}", pollard.roots.clone().unwrap()[1].data);
        println!("h2: {:?}", h3);
        assert_eq!(pollard.roots.clone().unwrap()[1].data, h3);

        println!("pollard leaves: {:?}", &pollard.num_leaves);
        println!("pollard root len: {:?}", pollard.roots.clone().unwrap().len());

        &pollard.modify(vec![leaf4], vec![]);

        println!("pollard leaves: {:?}", &pollard.num_leaves);
        println!("pollard root len: {:?}", pollard.roots.clone().unwrap().len());
        println!("root[0]: {:?}", pollard.roots.clone().unwrap()[0].data);
        //assert_ne!(pollard.roots.clone().unwrap()[1].data, h3);

        &pollard.modify(vec![leaf5], vec![]);
        println!("pollard leaves: {:?}", &pollard.num_leaves);
        println!("pollard root len: {:?}", pollard.roots.clone().unwrap().len());
        println!("root[0]: {:?}", pollard.roots.clone().unwrap()[0].data);
        assert_eq!(pollard.roots.clone().unwrap()[1].data, h5);
    }

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

        //let mut a = super::PolNode{
        //    data: h1,
        //    nieces: [None, None],
        //};

        //assert_eq!(a.data, h1_copy); // sanity

        //let mut b = super::PolNode{
        //    data: h2,
        //    nieces: [None, None],
        //};

        //assert_eq!(b.data, h2_copy); // sanity

        //let mut asib = super::PolNode{
        //    data: h3,
        //    nieces: [None, None],
        //};

        //let mut bsib = super::PolNode{
        //    data: h4,
        //    nieces: [None, None],
        //};

        //super::pol_swap(&mut a, &mut b, &mut asib, &mut bsib);

        //assert_eq!(a.data, h1_copy);
        //assert_eq!(b.data, h2_copy);

        //assert_eq!(asib.data, h3_copy);
        //assert_eq!(bsib.data, h4_copy);

        //mem::swap(&mut a, &mut b);

        //assert_eq!(a.data, h2_copy);
        //assert_eq!(b.data, h1_copy);
    }
}
