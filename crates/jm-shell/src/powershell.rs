pub fn init_script(data_dir: &str) -> String {
    // PowerShell double-quoted strings use the backtick as their escape
    // character. Escape interpolation-sensitive characters before embedding
    // the data directory in the generated profile script.
    let data_dir = data_dir
        .replace('`', "``")
        .replace('$', "`$")
        .replace('"', "`\"");

    format!(
        r#"# jm shell integration for PowerShell
# Add to $PROFILE: jm shell init powershell | Invoke-Expression

$env:JM_DATA_DIR = "{data_dir}"

# Track the environment values managed by jm so repeated prompt hooks do not
# grow PATH and so stale values can be removed without touching user changes.
if (-not (Get-Variable -Name __jm_active_bin -Scope Global -ErrorAction SilentlyContinue)) {{
    $global:__jm_active_bin = $null
}}
if (-not (Get-Variable -Name __jm_active_java_home -Scope Global -ErrorAction SilentlyContinue)) {{
    $global:__jm_active_java_home = $null
}}

# Auto-switch on directory change
function __jm_prompt_hook {{
    $pathSeparator = [IO.Path]::PathSeparator
    $escapedPathSeparator = [Regex]::Escape([string]$pathSeparator)

    # Ask jm for data, not executable shell source. This avoids trying to run
    # the POSIX `export` statements emitted by `jm env --shell`.
    $javaHome = & jm env --detect --java-home-only 2>$null | Select-Object -First 1
    if (-not $javaHome) {{
        if ($global:__jm_active_bin) {{
            $env:PATH = @($env:PATH -split $escapedPathSeparator | Where-Object {{
                $_ -and $_ -ne $global:__jm_active_bin
            }}) -join $pathSeparator
            $global:__jm_active_bin = $null
        }}
        if ($global:__jm_active_java_home -and
            $env:JAVA_HOME -eq $global:__jm_active_java_home) {{
            Remove-Item Env:JAVA_HOME -ErrorAction SilentlyContinue
        }}
        $global:__jm_active_java_home = $null
        return
    }}

    $javaHome = $javaHome.Trim()
    $javaBin = Join-Path $javaHome "bin"
    $pathEntries = @($env:PATH -split $escapedPathSeparator | Where-Object {{
        $_ -and $_ -ne $global:__jm_active_bin -and $_ -ne $javaBin
    }})

    $env:JAVA_HOME = $javaHome
    $env:PATH = (@($javaBin) + $pathEntries) -join $pathSeparator
    $global:__jm_active_bin = $javaBin
    $global:__jm_active_java_home = $javaHome
}}

# Register prompt hook
if (-not (Get-Variable -Name __jm_original_prompt -ErrorAction SilentlyContinue)) {{
    $global:__jm_original_prompt = $function:prompt
    function global:prompt {{
        __jm_prompt_hook
        & $global:__jm_original_prompt
    }}
}}

# Run once on init
__jm_prompt_hook
"#,
        data_dir = data_dir,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_switch_consumes_a_path_instead_of_posix_shell_source() {
        let script = init_script(r"C:\Users\user\.jm");

        assert!(script.contains("jm env --detect --java-home-only"));
        assert!(!script.contains("$envOutput"));
        assert!(!script.contains("Invoke-Expression $envOutput"));
    }

    #[test]
    fn path_updates_use_the_platform_separator() {
        let script = init_script(r"C:\Users\user\.jm");

        assert!(script.contains("$pathSeparator = [IO.Path]::PathSeparator"));
        assert!(script.contains("-split $escapedPathSeparator"));
        assert!(script.contains("-join $pathSeparator"));
        assert!(!script.contains("-split \";\""));
        assert!(!script.contains("-join \";\""));
    }

    #[test]
    fn missing_selection_only_clears_values_managed_by_jm() {
        let script = init_script(r"C:\Users\user\.jm");

        assert!(script.contains("$_ -and $_ -ne $global:__jm_active_bin"));
        assert!(script.contains("$env:JAVA_HOME -eq $global:__jm_active_java_home"));
        assert!(script.contains("Remove-Item Env:JAVA_HOME -ErrorAction SilentlyContinue"));
        assert!(script.contains("$global:__jm_active_bin = $null"));
        assert!(script.contains("$global:__jm_active_java_home = $null"));
    }

    #[test]
    fn data_dir_is_safe_in_a_double_quoted_powershell_string() {
        let script = init_script("C:\\Users\\$name\\`cache\\\"quoted\"");
        let expected = "$env:JM_DATA_DIR = \"C:\\Users\\`$name\\``cache\\`\"quoted`\"\"";

        assert!(script.contains(expected));
    }
}
