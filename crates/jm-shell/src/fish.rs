pub fn init_script(data_dir: &str) -> String {
    format!(
        r#"# jm shell integration for fish
# Add to ~/.config/fish/config.fish: jm shell init fish | source

set -gx JM_DATA_DIR "{data_dir}"

# Add current JDK to PATH
if test -d "$JM_DATA_DIR/current/bin"
    fish_add_path --prepend "$JM_DATA_DIR/current/bin"
    set -gx JAVA_HOME "$JM_DATA_DIR/current"
end

# Auto-switch hook
function __jm_auto_switch --on-variable PWD
    set -l env_output (command jm env --detect --shell 2>/dev/null)
    if test -n "$env_output"
        eval $env_output
    end
end

# Run once on init
__jm_auto_switch
"#,
        data_dir = data_dir,
    )
}
