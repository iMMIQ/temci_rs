//! Shell completion generation
//!
//! This module provides functionality to generate shell completion scripts
//! for bash, zsh, fish, elvish, and PowerShell.

#![allow(dead_code)]

use anyhow::{Result, anyhow};
use tracing::info;

/// Supported shell types for completion generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// Elvish shell
    Elvish,
    /// PowerShell
    PowerShell,
}

impl ShellType {
    /// Parse a string into a ShellType
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(ShellType::Bash),
            "zsh" => Some(ShellType::Zsh),
            "fish" => Some(ShellType::Fish),
            "elvish" => Some(ShellType::Elvish),
            "powershell" | "pwsh" => Some(ShellType::PowerShell),
            _ => None,
        }
    }

    /// Get the shell name as a string
    pub fn name(&self) -> &str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::Elvish => "elvish",
            ShellType::PowerShell => "powershell",
        }
    }

    /// Detect the current shell from environment
    pub fn detect_from_env() -> Option<Self> {
        // Check SHELL environment variable
        if let Ok(shell_path) = std::env::var("SHELL") {
            let shell_name = std::path::Path::new(&shell_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            return match shell_name {
                "bash" => Some(ShellType::Bash),
                "zsh" => Some(ShellType::Zsh),
                "fish" => Some(ShellType::Fish),
                "elvish" => Some(ShellType::Elvish),
                _ => None,
            };
        }

        None
    }
}

/// Generate shell completion
///
/// # Arguments
///
/// * `shell` - Optional shell type (bash, zsh, fish, elvish, powershell)
///   If None, attempts to detect from environment
pub async fn completion(shell: Option<String>) -> Result<()> {
    // Determine shell type
    let shell_type = if let Some(shell_name) = shell {
        ShellType::from_str(&shell_name)
            .ok_or_else(|| anyhow!("Unsupported shell: {}", shell_name))?
    } else {
        // Try to detect from environment, default to bash
        ShellType::detect_from_env().unwrap_or(ShellType::Bash)
    };

    info!("Generating {} completion", shell_type.name());

    // Generate and print completion script
    let script = generate_completion_script(shell_type);
    print!("{}", script);

    Ok(())
}

/// Generate the completion script for the given shell
fn generate_completion_script(shell: ShellType) -> String {
    match shell {
        ShellType::Bash => generate_bash_completion(),
        ShellType::Zsh => generate_zsh_completion(),
        ShellType::Fish => generate_fish_completion(),
        ShellType::Elvish => generate_elvish_completion(),
        ShellType::PowerShell => generate_powershell_completion(),
    }
}

/// Generate bash completion script
fn generate_bash_completion() -> String {
    r#"_temci_completion() {
    local cur prev words cword
    _init_completion || return

    case ${{prev}} in
        --config|-c)
            _filedir
            return
            ;;
        --format|-f)
            COMPREPLY=($(compgen -W 'console csv json markdown' -- "${{cur}}"))
            return
            ;;
        --driver|--suite|-s)
            _filedir
            return
            ;;
    esac

    if [[ ${{cword}} -eq 1 ]]; then
        COMPREPLY=($(compgen -W 'exec short-exec build clean report setup completion help' -- "${{cur}}"))
    fi
}

complete -F _temci_completion temci"#.to_string()
}

/// Generate zsh completion script
fn generate_zsh_completion() -> String {
    r#"#compdef temci

_temci() {
    local -a commands
    commands=(
        'exec:Execute benchmarks with full configuration'
        'short-exec:Quick execution of commands'
        'build:Build benchmark executables'
        'clean:Clean build artifacts'
        'report:Generate benchmark report'
        'setup:Setup initial configuration'
        'completion:Generate shell completion'
        'help:Print this message or the help of the given subcommand(s)'
    )

    if (( CURRENT == 2 )); then
        _describe 'command' commands
    else
        case ${{words[2]}} in
            exec)
                _arguments '--suite[Benchmark suite configuration]' \
                          '--runs[Number of executions]' \
                          '--driver[Run driver to use]:driver:(basic perf perf_stat perf_record valgrind)' \
                          '--no-affinity[Disable CPU affinity]' \
                          '--summary[Show only summary]'
                ;;
            short-exec)
                _arguments '--runs[Number of executions]:runs:' \
                          '--warmup[Warmup runs]:warmup:' \
                          '--summary[Show summary]' \
                          '*:commands:_command_names'
                ;;
            report)
                _arguments '--format[Report type]:format:(console csv json markdown)' \
                          '--output[Output file]:file:_files' \
                          '--input[Input data file]:file:_files'
                ;;
            completion)
                _arguments '--shell[Shell type]:shell:(bash zsh fish elvish powershell)'
                ;;
        esac
    fi
}

_temci"#.to_string()
}

/// Generate fish completion script
fn generate_fish_completion() -> String {
    r#"complete -c temci -f

complete -c temci -n '__fish_use_subcommand' -a exec -d 'Execute benchmarks'
complete -c temci -n '__fish_seen_subcommand_from exec' -l suite -d 'Benchmark suite configuration'
complete -c temci -n '__fish_seen_subcommand_from exec' -l runs -d 'Number of executions'
complete -c temci -n '__fish_seen_subcommand_from exec' -l driver -d 'Run driver' -k -a 'basic perf perf_stat perf_record valgrind'
complete -c temci -n '__fish_seen_subcommand_from exec' -l no-affinity -d 'Disable CPU affinity'
complete -c temci -n '__fish_seen_subcommand_from exec' -s s -l summary -d 'Show only summary'

complete -c temci -n '__fish_use_subcommand' -a short-exec -d 'Quick execution'
complete -c temci -n '__fish_seen_subcommand_from short-exec' -l runs -d 'Number of executions'
complete -c temci -n '__fish_seen_subcommand_from short-exec' -l warmup -d 'Warmup runs'
complete -c temci -n '__fish_seen_subcommand_from short-exec' -s s -l summary -d 'Show summary'

complete -c temci -n '__fish_use_subcommand' -a build -d 'Build executables'
complete -c temci -n '__fish_seen_subcommand_from build' -l force -d 'Force rebuild'
complete -c temci -n '__fish_seen_subcommand_from build' -l release -d 'Release build'

complete -c temci -n '__fish_use_subcommand' -a clean -d 'Clean artifacts'
complete -c temci -n '__fish_seen_subcommand_from clean' -l all -d 'Clean all artifacts'

complete -c temci -n '__fish_use_subcommand' -a report -d 'Generate report'
complete -c temci -n '__fish_seen_subcommand_from report' -l format -d 'Report type' -k -a 'console csv json markdown'
complete -c temci -n '__fish_seen_subcommand_from report' -l output -d 'Output file'
complete -c temci -n '__fish_seen_subcommand_from report' -l input -d 'Input file'

complete -c temci -n '__fish_use_subcommand' -a setup -d 'Setup configuration'
complete -c temci -n '__fish_seen_subcommand_from setup' -l overwrite -d 'Overwrite existing'

complete -c temci -n '__fish_use_subcommand' -a completion -d 'Generate completions'
complete -c temci -n '__fish_seen_subcommand_from completion' -l shell -d 'Shell type' -k -a 'bash zsh fish elvish powershell'"#.to_string()
}

/// Generate elvish completion script
fn generate_elvish_completion() -> String {
    r#"use str

edit:completion:arg-completer[temci] = [@words] {
    var command = $words[2]
    if (eq $words[-1] '') {
        # Complete subcommands
        if (eq (count $words) 2) {
            put exec short-exec build clean report setup completion help
        }
    } else {
        # Complete options
        if (eq $command exec) {
            put --suite --runs --driver --no-affinity --summary
        } elif (eq $command short-exec) {
            put --runs --warmup --summary
        } elif (eq $command report) {
            put --format --output --input
        } elif (eq $command completion) {
            put --shell
        }
    }
}
"#.to_string()
}

/// Generate PowerShell completion script
fn generate_powershell_completion() -> String {
    r#"
Register-ArgumentCompleter -Native -CommandName temci -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @('exec', 'short-exec', 'build', 'clean', 'report', 'setup', 'completion', 'help')

    if ($commandAst.CommandElements.Count -eq 1) {
        $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
    } else {
        $subcommand = $commandAst.CommandElements[1].Extent.Text
        switch ($subcommand) {
            'exec' {
                $options = @('--suite', '--runs', '--driver', '--no-affinity', '--summary')
                $options | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
            }
            'report' {
                $options = @('--format', '--output', '--input')
                $options | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
            }
            'completion' {
                $options = @('--shell')
                $options | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
            }
        }
    }
}
"#.to_string()
}

/// Print installation instructions for the given shell
pub fn print_install_instructions(shell: ShellType) {
    println!("\n{} Installation Instructions:", shell.name());
    println!("{}", "=".repeat(60));

    match shell {
        ShellType::Bash => {
            println!("Add the following to your ~/.bashrc or ~/.bash_profile:");
            println!("\n  source <(temci completion bash)\n");
            println!("Or for a system-wide installation:");
            println!("\n  temci completion bash > /etc/bash_completion.d/temci.bash\n");
        }
        ShellType::Zsh => {
            println!("Add the following to your ~/.zshrc:");
            println!("\n  # Load temci completion");
            println!("  source <(temci completion zsh)\n");
            println!("Or place the completion script in your completions path:");
            println!("\n  temci completion zsh > ~/.zsh/completion/_temci\n");
        }
        ShellType::Fish => {
            println!("Add the following to your ~/.config/fish/config.fish:");
            println!("\n  temci completion fish | source\n");
            println!("Or place the completion script in your completions directory:");
            println!("\n  temci completion fish > ~/.config/fish/completions/temci.fish\n");
        }
        ShellType::Elvish => {
            println!("Add the following to your ~/.elvish/rc.elv:");
            println!("\n  temci completion elvish | slurp\n");
        }
        ShellType::PowerShell => {
            println!("Add the following to your PowerShell profile:");
            println!("\n  temci completion powershell | Out-String | Invoke-Expression\n");
            println!("To find your PowerShell profile location, run:");
            println!("\n  echo $PROFILE\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_parsing() {
        assert_eq!(ShellType::from_str("bash"), Some(ShellType::Bash));
        assert_eq!(ShellType::from_str("zsh"), Some(ShellType::Zsh));
        assert_eq!(ShellType::from_str("fish"), Some(ShellType::Fish));
        assert_eq!(ShellType::from_str("elvish"), Some(ShellType::Elvish));
        assert_eq!(ShellType::from_str("powershell"), Some(ShellType::PowerShell));
        assert_eq!(ShellType::from_str("pwsh"), Some(ShellType::PowerShell));
        assert_eq!(ShellType::from_str("invalid"), None);
    }

    #[test]
    fn test_shell_type_name() {
        assert_eq!(ShellType::Bash.name(), "bash");
        assert_eq!(ShellType::Zsh.name(), "zsh");
        assert_eq!(ShellType::Fish.name(), "fish");
        assert_eq!(ShellType::Elvish.name(), "elvish");
        assert_eq!(ShellType::PowerShell.name(), "powershell");
    }

    #[tokio::test]
    async fn test_completion_bash() -> Result<()> {
        let result = completion(Some("bash".to_string())).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_completion_invalid() -> Result<()> {
        let result = completion(Some("invalid".to_string())).await;
        assert!(result.is_err());
        Ok(())
    }
}
