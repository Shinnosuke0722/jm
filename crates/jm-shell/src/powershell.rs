pub fn init_script(data_dir: &str) -> String {
    format!(
        r#"# jm shell integration for PowerShell
# Add to $PROFILE: jm shell init powershell | Invoke-Expression

$env:JM_DATA_DIR = "{data_dir}"

# Add current JDK to PATH
$jmCurrentBin = Join-Path $env:JM_DATA_DIR "current\bin"
if (Test-Path $jmCurrentBin) {{
    $env:PATH = "$jmCurrentBin;$env:PATH"
    $env:JAVA_HOME = Join-Path $env:JM_DATA_DIR "current"
}}

# Auto-switch on directory change
function __jm_prompt_hook {{
    $envOutput = & jm env --detect --shell 2>$null
    if ($envOutput) {{
        Invoke-Expression $envOutput
    }}
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
