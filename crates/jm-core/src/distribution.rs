use serde::{Deserialize, Serialize};
use std::fmt;

/// Known JDK distributions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Distribution {
    Temurin,
    Corretto,
    Zulu,
    Oracle,
    OracleOpenJdk,
    Liberica,
    SapMachine,
    Semeru,
    Dragonwell,
    GraalVmCe,
    GraalVmOracle,
    Mandrel,
    Microsoft,
    /// Any distribution not explicitly enumerated.
    Other(String),
}

impl Distribution {
    /// The Foojay Disco API parameter name for this distribution.
    pub fn api_parameter(&self) -> &str {
        match self {
            Self::Temurin => "temurin",
            Self::Corretto => "corretto",
            Self::Zulu => "zulu",
            Self::Oracle => "oracle",
            Self::OracleOpenJdk => "oracle_open_jdk",
            Self::Liberica => "liberica",
            Self::SapMachine => "sap_machine",
            Self::Semeru => "semeru",
            Self::Dragonwell => "dragonwell",
            Self::GraalVmCe => "graalvm_ce",
            Self::GraalVmOracle => "graalvm",
            Self::Mandrel => "mandrel",
            Self::Microsoft => "microsoft",
            Self::Other(name) => name.as_str(),
        }
    }

    /// Parse a distribution from a user-supplied string.
    /// Supports common aliases and SDKMAN suffixes.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "temurin" | "tem" | "adoptium" | "eclipse" => Self::Temurin,
            "corretto" | "amzn" | "amazon" => Self::Corretto,
            "zulu" | "azul" => Self::Zulu,
            "oracle" => Self::Oracle,
            "oracle_open_jdk" | "oracle-openjdk" | "openjdk" => Self::OracleOpenJdk,
            "liberica" | "librca" | "bellsoft" => Self::Liberica,
            "sap_machine" | "sapmachine" | "sapmchn" => Self::SapMachine,
            "semeru" | "ibm" => Self::Semeru,
            "dragonwell" | "alibaba" => Self::Dragonwell,
            "graalvm_ce" | "graalvm-ce" => Self::GraalVmCe,
            "graalvm" | "graalvm_oracle" | "graalvm-oracle" => Self::GraalVmOracle,
            "mandrel" => Self::Mandrel,
            "microsoft" | "ms" => Self::Microsoft,
            other => Self::Other(other.to_string()),
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Temurin => "Eclipse Temurin",
            Self::Corretto => "Amazon Corretto",
            Self::Zulu => "Azul Zulu",
            Self::Oracle => "Oracle JDK",
            Self::OracleOpenJdk => "Oracle OpenJDK",
            Self::Liberica => "BellSoft Liberica",
            Self::SapMachine => "SAP Machine",
            Self::Semeru => "IBM Semeru",
            Self::Dragonwell => "Alibaba Dragonwell",
            Self::GraalVmCe => "GraalVM CE",
            Self::GraalVmOracle => "GraalVM Oracle",
            Self::Mandrel => "Mandrel",
            Self::Microsoft => "Microsoft OpenJDK",
            Self::Other(name) => name.as_str(),
        }
    }
}

impl fmt::Display for Distribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.api_parameter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_common_names() {
        assert_eq!(Distribution::parse("temurin"), Distribution::Temurin);
        assert_eq!(Distribution::parse("tem"), Distribution::Temurin);
        assert_eq!(Distribution::parse("corretto"), Distribution::Corretto);
        assert_eq!(Distribution::parse("amzn"), Distribution::Corretto);
        assert_eq!(Distribution::parse("zulu"), Distribution::Zulu);
        assert_eq!(Distribution::parse("Temurin"), Distribution::Temurin);
    }

    #[test]
    fn parse_unknown_distribution() {
        assert_eq!(
            Distribution::parse("custom"),
            Distribution::Other("custom".to_string())
        );
    }

    #[test]
    fn api_parameter_roundtrip() {
        let dist = Distribution::Temurin;
        assert_eq!(dist.api_parameter(), "temurin");
        assert_eq!(Distribution::parse(dist.api_parameter()), dist);
    }
}
