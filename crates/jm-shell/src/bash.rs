pub fn init_script(data_dir: &str) -> String {
    format!(
        r#"# jm shell integration for bash
# Add to ~/.bashrc: eval "$(jm shell init bash)"

export JM_DATA_DIR="{data_dir}"

# Add current JDK to PATH
if [ -d "$JM_DATA_DIR/current/bin" ]; then
    export PATH="$JM_DATA_DIR/current/bin:$PATH"
    export JAVA_HOME="$JM_DATA_DIR/current"
fi

# Auto-switch hook on directory change
__jm_cd() {{
    builtin cd "$@" || return
    __jm_auto_switch
}}

__jm_auto_switch() {{
    local env_output
    env_output="$(command jm env --detect --shell 2>/dev/null)"
    if [ -n "$env_output" ]; then
        eval "$env_output"
    fi
}}

alias cd='__jm_cd'

# Run once on init
__jm_auto_switch
"#,
        data_dir = data_dir,
    )
}
