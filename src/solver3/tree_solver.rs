struct MineRegion {
    child_regions: Vec<usize>,
    arrangements: Vec<u8>,
}

struct MineArrangements {
    regions: Vec<Vec<Option<MineRegion>>>,
}
