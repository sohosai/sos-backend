fn main() -> Result<(), Box<dyn std::error::Error>> {
    vergen::vergen(vergen::Config::default())?;

    Ok(())
}
