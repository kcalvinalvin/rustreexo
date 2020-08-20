// Rustreexo

/// Transform is the
pub fn Transform(dels: Vec<u64>, nLeaves: u64, rows: u8) -> Vec<Vec<arrow>> {
    nNextLeaves = nLeaves - length(dels);

    for r in 0..rows {
        delRoot();
        extractTwins();
        swapNextDels();
    }
    return swaps;
}
