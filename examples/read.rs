use std::path::Path;

use lucene_utils::load_index;

/// Reads contents from an index
fn main() {
    load_index(Path::new("./test-data/simple-index"));
}
