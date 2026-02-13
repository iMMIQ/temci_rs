//! Short execution command for quick benchmarks

use anyhow::Result;

/// Quick execution of commands
pub async fn short_exec(
    commands: Vec<String>,
    runs: usize,
    warmup: usize,
    summary: bool,
) -> Result<()> {
    tracing::info!(
        "ShortExec: commands={:?}, runs={}, warmup={}, summary={}",
        commands, runs, warmup, summary
    );
    // TODO: Implement short execution
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_short_exec_placeholder() {
        let result = short_exec(vec!["echo test".to_string()], 5, 0, false).await;
        assert!(result.is_ok());
    }
}
