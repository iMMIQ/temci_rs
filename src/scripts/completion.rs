//! Shell completion generation

use anyhow::Result;

/// Generate shell completion
pub async fn completion(shell: Option<String>) -> Result<()> {
    let shell_name = shell.as_deref().unwrap_or("bash");

    match shell_name {
        "bash" => {
            // Bash completion script
            println!("_temci_completion() {{
    local cur prev words cword
    _init_completion || return

    case ${{prev}} in
        --config)
            _filedir
            return
            ;;
        --format)
            COMPREPLY=($(compgen -W 'console csv json' -- \"${{cur}}\"))
            return
            ;;
        --driver)
            COMPREPLY=($(compgen -W 'basic perf perf_stat perf_record valgrind' -- \"${{cur}}\"))
            return
            ;;
        --shell)
            COMPREPLY=($(compgen -W 'bash zsh fish elvish' -- \"${{cur}}\"))
            return
            ;;
    esac

    if [[ ${{cword}} -eq 1 ]]; then
        COMPREPLY=($(compgen -W 'exec short-exec build clean report setup completion help' -- \"${{cur}}\"))
    fi
}}

complete -F _temci_completion temci");
        }
        "zsh" => {
            // Zsh completion script
            println!("#compdef temci

_temci() {{
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
                _arguments '--format[Report type]:format:(console csv json)' \
                          '--output[Output file]:file:_files' \
                          '--input[Input data file]:file:_files'
                ;;
            completion)
                _arguments '--shell[Shell type]:shell:(bash zsh fish elvish powershell)'
                ;;
        esac
    fi
}}

_temci");
        }
        "fish" => {
            // Fish completion script
            println!("complete -c temci -f

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
complete -c temci -n '__fish_seen_subcommand_from report' -l format -d 'Report type' -k -a 'console csv json'
complete -c temci -n '__fish_seen_subcommand_from report' -l output -d 'Output file'
complete -c temci -n '__fish_seen_subcommand_from report' -l input -d 'Input file'

complete -c temci -n '__fish_use_subcommand' -a setup -d 'Setup configuration'
complete -c temci -n '__fish_seen_subcommand_from setup' -l overwrite -d 'Overwrite existing'

complete -c temci -n '__fish_use_subcommand' -a completion -d 'Generate completions'
complete -c temci -n '__fish_seen_subcommand_from completion' -l shell -d 'Shell type' -k -a 'bash zsh fish elvish powershell'");
        }
        "elvish" => {
            // Elvish completion script
            println!("use str");

            println!("edit:completion:arg-completer[temci] = [@words] {{");
            println!("    var command = $words[2]");
            println!("    if (eq $words[-1] '') {{");
            println!("        # Complete subcommands");
            println!("        if (eq (count $words) 2) {{");
            println!("            put exec short-exec build clean report setup completion help");
            println!("        }}");
            println!("    }} else {{");
            println!("        # Complete options");
            println!("        if (eq $command exec) {{");
            println!("            put --suite --runs --driver --no-affinity --summary");
            println!("        }} elif (eq $command short-exec) {{");
            println!("            put --runs --warmup --summary");
            println!("        }} elif (eq $command report) {{");
            println!("            put --format --output --input");
            println!("        }} elif (eq $command completion) {{");
            println!("            put --shell");
            println!("        }}");
            println!("    }}");
            println!("}}");
        }
        "powershell" => {
            // PowerShell completion script
            println!("Register-ArgumentCompleter -Native -CommandName temci -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @('exec', 'short-exec', 'build', 'clean', 'report', 'setup', 'completion', 'help')

    if ($commandAst.CommandElements.Count -eq 1) {{
        $commands | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }}
    }} else {{
        $subcommand = $commandAst.CommandElements[1].Extent.Text
        switch ($subcommand) {{
            'exec' {{
                $options = @('--suite', '--runs', '--driver', '--no-affinity', '--summary')
                $options | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }}
            }}
            'report' {{
                $options = @('--format', '--output', '--input')
                $options | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }}
            }}
            'completion' {{
                $options = @('--shell')
                $options | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }}
            }}
        }}
    }}
}}");
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported shell: {}", shell_name));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_completion_bash() {
        let result = completion(Some("bash".to_string())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_completion_invalid() {
        let result = completion(Some("invalid".to_string())).await;
        assert!(result.is_err());
    }
}
