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
