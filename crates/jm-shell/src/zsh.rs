pub fn init_script(data_dir: &str) -> String {
    format!(
        r#"# jm shell integration for zsh
# Add to ~/.zshrc: eval "$(jm shell init zsh)"

export JM_DATA_DIR="{data_dir}"

# Add current JDK to PATH
if [ -d "$JM_DATA_DIR/current/bin" ]; then
    export PATH="$JM_DATA_DIR/current/bin:$PATH"
    export JAVA_HOME="$JM_DATA_DIR/current"
fi

# Auto-switch hook using chpwd
__jm_auto_switch() {{
    local env_output
    env_output="$(command jm env --detect --shell 2>/dev/null)"
    if [ -n "$env_output" ]; then
        eval "$env_output"
    fi
}}

autoload -U add-zsh-hook
add-zsh-hook chpwd __jm_auto_switch

# Run once on init
__jm_auto_switch
"#,
        data_dir = data_dir,
    )
}
