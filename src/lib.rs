use anyhow::{bail, Result};

pub fn add(a: u8, b: u8) -> Result<u8> {
    if a + b == 3 {
        bail!("3 is not allowed")
    }
    Ok(a + b)
}
