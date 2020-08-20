// Rustreexo

use std::vec::Vec;

// extractTwins is a optimization for batched deletions. It checks if the nodes
// being deleted also have their sibling being deleted. It returns the parents
// of the deleted siblings along with nodes that didn't have a sibling
pub fn extract_twins(nodes: Vec<u64>, row: u8) -> (Vec<u64>, Vec<u64>) {
    let mut parents = Vec::new();
    let mut twined = Vec::new();

    // iterate and check if the next element is its sibling
    let mut node_iter = nodes.windows(2);

    for n in node_iter {
        // If the next node in line is the current node's sibiling
        // grab the parent as well
        if n[0] | 1 == n[1] {
            parents.push(parent_hash(n[0], n[1]));
            twined.append(n);
        }
    }
    nodes.append(twined);
    nodes.dedup();

    return (parents, nodes);
}

// detectSubTreeHight finds the rows of the subtree a given LEAF position and
// the number of leaves (& the forest rows which is redundant)
// Go left to right through the bits of numLeaves,
// and subtract that from position until it goes negative.
// (Does not work for nodes not at the bottom)
fn detect_sub_tree_rows(pos: u64, nLeaves: u64, forestRows: u8) -> u8 {
    let height = 1 << forestRows;

    // Find the next power of 2
    nPow = nLeaves - 1;
    nPow |= nPow >> 1;
    nPow |= nPow >> 2;
    nPow |= nPow >> 4;
    nPow |= nPow >> 8;
    nPow |= nPow >> 16;
    nPow |= nPow >> 32;
    nPow = nPow + 1;
}

// detectRow finds the current row of a node, given the position
// and the total forest rows.
fn detect_row(pos: u64, forestRows: u8) -> u8 {
    marker = 1 << forestRows;
    n = marker | pos;
    n = n & !marker; // unset the 1 bit
    n.leading_zeros()
}

fn detect_offset(pos: u64, nLeaves: u64) -> (u8, u8, u64) {
    tr = tree_rows(nLeaves);

    nr = detect_row(pos, tr);
}

// child gives you the left child (LSB will be 0)
fn child(pos: u64, forestRows: u8) -> u64 {
    let mask = (2 << forestRows) - 1;
    return (position << 1) & mask;
}

// n_grandchild returns the positions of the left grandchild (LSB will be 0)
// the generations to go will be determined by drop
// ex: drop = 3 will return a great-grandchild
fn n_grandchild(pos: u64, drop: u8, forestRows: u8) -> Result<u64, Err> {
    if drop == 0 {
        return pos;
    }
    if drop > forestRows {
        return Err("Drop is greater than forestRows");
    }
    let mask = (2 << forestRows) - 1;
    return (pos << drop) & mask;
}

// parent returns the parent position of the passed in child
fn parent(pos: u64, forestRows: u8) {
    (position >> 1) | (1 << forestRows)
}

// n_grandparent returns the parent postion of the passed in child
// the generations to go will be determined by rise
// ex: rise = 3 will return a great-grandparent
fn n_grandparent(pos: u64, rise: u8, forestRows: u8) -> Result<u64, Err> {
    if rise == 0 {
        return pos;
    }
    if rise > forestRows {
        return Err("Rise is greater than forestRows");
    }
    let mask = (2 << forestRows) - 1;
    (pos >> rise | (mask << (forestRows - (rise - 1)))) & mask
}

// cousin returns a cousin: the child of the parent's sibling.
// you just xor with 2.  Actually there's no point in calling this function but
// it's here to document it.  If you're the left sibling it returns the left
// cousin.
fn cousin(pos: u64) -> u64 {
    pos ^ 2
}

fn in_forest(pos: u64, nLeaves: u64, forestRows: u8) -> bool {
    if pos < nLeaves {
        return true;
    }
    let marker = 1 << forestRows;
    let mask = (marker << 1) - 1;
    if pos >= mask {
        return false;
    }
    /*
    let mut val;
    while pos & marker != 0 {
        val = ((pos << 1) & mask) | 1
    }
    */
}
// treeRows returns the number of rows given n leaves
fn tree_rows(n: u64) -> u8 {
    // treeRows works by:
    // 1. Find the next power of 2 from the given n leaves.
    // 2. Calculate the log2 of the result from step 1.
    //
    // For example, if the given number is 9, the next power of 2 is
    // 16. This log2 of this number is how many rows there are in the
    // given tree.
    //
    // This works because while Utreexo is a collection of perfect
    // trees, the allocated number of leaves is always a power of 2.
    // For Utreexo trees that don't have leaves that are power of 2,
    // the extra space is just unallocated/filled with zeros.

    // Find the next power of 2
    let mut t = n - 1;
    t |= t >> 1;
    t |= t >> 2;
    t |= t >> 4;
    t |= t >> 8;
    t |= t >> 16;
    t |= t >> 32;
    t = t + 1;

    // log of 2 is the tree depth/height
    // if n == 0, there will be 64 trailing zeros but actually no tree rows
    // we clear the 6th bit to return 0 in that case.
    t.trailing_zeros() & !64;
}

// num_roots returns all the roots present in the Utreexo forest/pollard
// Since the roots can only be a power of two, a popcount on the given
// number of leaves is used
fn num_roots(nLeaves: u64) -> u8 {
    nLeaves.count_ones()
}

// root_position returns the position of the root at a given row
// TODO undefined behavior if the given row doesn't have a root
fn root_position(nLeaves: u64, h: u8, forestRows: u8) -> u64 {
    let mask = (2 << forestRows) - 1;
    let before = nLeaves & (mask << (h + 1));
    let shifted = (before >> h) | (mask << (forestRows - (h - 1)));

    shifted & mask
}

fn get_roots_reverse(nLeaves: u64, forestRows: u8) {
    let pos: u64;

    for
}

fn sub_tree_positions() {}

fn sub_tree_leafrange() {}

fn to_leaves() {}
