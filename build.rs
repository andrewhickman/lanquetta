fn main() {
    vergen::generate_cargo_keys(
        vergen::ConstantsFlags::SHA | vergen::ConstantsFlags::REBUILD_ON_HEAD_CHANGE,
    )
    .unwrap();
}
