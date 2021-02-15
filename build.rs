fn main() {
    vergen::gen(
        vergen::ConstantsFlags::SHA | vergen::ConstantsFlags::REBUILD_ON_HEAD_CHANGE,
    )
    .unwrap();
}
