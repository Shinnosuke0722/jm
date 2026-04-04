/// Supported shell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl Shell {
    /// Parse a shell name from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "powershell" | "pwsh" => Some(Self::PowerShell),
            _ => None,
        }
    }

    /// Generate the shell initialization script.
    pub fn init_script(&self, data_dir: &str) -> String {
        match self {
            Self::Bash => crate::bash::init_script(data_dir),
            Self::Zsh => crate::zsh::init_script(data_dir),
            Self::Fish => crate::fish::init_script(data_dir),
            Self::PowerShell => crate::powershell::init_script(data_dir),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_shells() {
        assert_eq!(Shell::parse("bash"), Some(Shell::Bash));
        assert_eq!(Shell::parse("zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::parse("fish"), Some(Shell::Fish));
        assert_eq!(Shell::parse("powershell"), Some(Shell::PowerShell));
        assert_eq!(Shell::parse("pwsh"), Some(Shell::PowerShell));
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(Shell::parse("BASH"), Some(Shell::Bash));
        assert_eq!(Shell::parse("Zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::parse("PowerShell"), Some(Shell::PowerShell));
    }

    #[test]
    fn parse_unknown_shell() {
        assert_eq!(Shell::parse("tcsh"), None);
        assert_eq!(Shell::parse(""), None);
    }

    #[test]
    fn bash_script_contains_required_elements() {
        let script = Shell::Bash.init_script("/home/user/.jm");
        assert!(script.contains("JM_DATA_DIR=\"/home/user/.jm\""));
        assert!(script.contains("JAVA_HOME"));
        assert!(script.contains("PATH"));
        assert!(script.contains("__jm_auto_switch"));
        assert!(script.contains("cd"));
    }

    #[test]
    fn zsh_script_contains_required_elements() {
        let script = Shell::Zsh.init_script("/home/user/.jm");
        assert!(script.contains("JM_DATA_DIR=\"/home/user/.jm\""));
        assert!(script.contains("JAVA_HOME"));
        assert!(script.contains("PATH"));
        assert!(script.contains("__jm_auto_switch"));
        assert!(script.contains("chpwd"));
    }

    #[test]
    fn fish_script_contains_required_elements() {
        let script = Shell::Fish.init_script("/home/user/.jm");
        assert!(script.contains("JM_DATA_DIR \"/home/user/.jm\""));
        assert!(script.contains("JAVA_HOME"));
        assert!(script.contains("fish_add_path"));
        assert!(script.contains("__jm_auto_switch"));
        assert!(script.contains("--on-variable PWD"));
    }

    #[test]
    fn powershell_script_contains_required_elements() {
        let script = Shell::PowerShell.init_script("C:\\Users\\user\\.jm");
        assert!(script.contains("JM_DATA_DIR = \"C:\\Users\\user\\.jm\""));
        assert!(script.contains("JAVA_HOME"));
        assert!(script.contains("PATH"));
        assert!(script.contains("__jm_prompt_hook"));
        assert!(script.contains("prompt"));
    }

    #[test]
    fn data_dir_is_interpolated() {
        let custom = "/opt/custom/jm";
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let script = shell.init_script(custom);
            assert!(
                script.contains(custom),
                "{:?} script should contain data_dir '{}'",
                shell,
                custom
            );
        }
    }

    #[test]
    fn scripts_are_not_empty() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let script = shell.init_script("/tmp/jm");
            assert!(
                script.len() > 100,
                "{:?} script is too short ({} chars)",
                shell,
                script.len()
            );
        }
    }
}
