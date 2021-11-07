fn main() -> Result<(), Box<dyn std::error::Error>> {
    built::write_built_file()?;

    Ok(())
}
