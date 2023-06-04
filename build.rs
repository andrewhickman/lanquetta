fn main() -> anyhow::Result<()> {
    vergen::EmitBuilder::builder().git_sha(true).emit()
}
