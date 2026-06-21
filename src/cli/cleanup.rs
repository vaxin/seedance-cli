use anyhow::Result;

use crate::core::tos;

pub async fn execute() -> Result<()> {
    let deleted = tos::cleanup_expired().await?;
    if deleted > 0 {
        println!("Cleaned up {deleted} expired TOS temp file(s)");
    } else {
        println!("No expired TOS temp files found");
    }
    Ok(())
}
