use std::fs;

fn main() {
    let path = orca::base_dir().join("collection.json");

    if path.exists() {
        fs::remove_file(&path).expect("failed to remove collection.json");
        println!("collection cleared");
    } else {
        println!("no collection to clear");
    }
}
