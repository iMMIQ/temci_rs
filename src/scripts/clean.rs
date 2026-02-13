//! Clean command for build artifacts

use anyhow::Result;

/// Clean build artifacts
pub async fn clean(all: bool) -> Result<()> {
    tracing::info!("Clean: all={}", all);
    // TODO: Implement cleaning
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clean_placeholder() {
        let result = clean(false).await;
        assert!(result.is_ok());
    }
}
