//! Token-budget system for knowledge query commands.
//!
//! Agents can pass `--depth brief|default|full` to control how much
//! knowledge is returned per query — trading completeness for token cost.

/// How much knowledge to include in a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Depth {
    /// One-liner answer + suggested command. ~50–70 tokens.
    Brief,
    /// Answer + numbered steps. ~100–200 tokens. (Default)
    Default,
    /// Full walkthrough: answer + steps + files_affected + dependencies + learn_more. ~400 tokens.
    Full,
}

impl Depth {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "brief" | "short" | "b" => Depth::Brief,
            "full" | "verbose" | "f" | "v" => Depth::Full,
            _ => Depth::Default,
        }
    }

    /// Approximate token ceiling for this depth level.
    pub fn token_limit(&self) -> u32 {
        match self {
            Depth::Brief => 70,
            Depth::Default => 200,
            Depth::Full => 500,
        }
    }

    /// Whether to include numbered steps in the response.
    pub fn include_steps(&self) -> bool {
        !matches!(self, Depth::Brief)
    }

    /// Whether to include files_affected, dependencies, and learn_more.
    pub fn include_extras(&self) -> bool {
        matches!(self, Depth::Full)
    }
}

impl std::fmt::Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Depth::Brief => write!(f, "brief"),
            Depth::Default => write!(f, "default"),
            Depth::Full => write!(f, "full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_depth_variants() {
        assert_eq!(Depth::from_str("brief"), Depth::Brief);
        assert_eq!(Depth::from_str("full"), Depth::Full);
        assert_eq!(Depth::from_str("default"), Depth::Default);
        assert_eq!(Depth::from_str("anything"), Depth::Default);
    }

    #[test]
    fn brief_excludes_steps() {
        assert!(!Depth::Brief.include_steps());
        assert!(Depth::Default.include_steps());
        assert!(Depth::Full.include_steps());
    }
}
