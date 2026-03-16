/// The four-tier template hierarchy used by forge.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Indivisible code fragments — cannot include other atoms.
    Atom,
    /// Atoms composed into a self-contained logical block.
    Molecule,
    /// File-level shell — wraps molecules with import hoisting.
    Layout,
    /// Top-level feature template — wires molecules into a layout.
    Feature,
    /// Markdown knowledge file for the Framework Protocol.
    Knowledge,
    /// Tier could not be determined from the path.
    Unknown,
}

impl Tier {
    /// Infer tier from template path (e.g. "atoms/drizzle/column.jinja" → Atom).
    pub fn from_path(path: &str) -> Self {
        let p = path.replace('\\', "/");
        if p.starts_with("atoms/") || p.contains("/atoms/") {
            Tier::Atom
        } else if p.starts_with("molecules/") || p.contains("/molecules/") {
            Tier::Molecule
        } else if p.starts_with("layouts/") || p.contains("/layouts/") {
            Tier::Layout
        } else if p.starts_with("features/") || p.contains("/features/") {
            Tier::Feature
        } else if p.ends_with(".md") {
            Tier::Knowledge
        } else {
            Tier::Unknown
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::Atom => write!(f, "atom"),
            Tier::Molecule => write!(f, "molecule"),
            Tier::Layout => write!(f, "layout"),
            Tier::Feature => write!(f, "feature"),
            Tier::Knowledge => write!(f, "knowledge"),
            Tier::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_atom_from_path() {
        assert_eq!(Tier::from_path("atoms/drizzle/column.jinja"), Tier::Atom);
        assert_eq!(Tier::from_path("templates/atoms/zod/field.jinja"), Tier::Atom);
    }

    #[test]
    fn infers_molecule_from_path() {
        assert_eq!(Tier::from_path("molecules/drizzle/table_body.jinja"), Tier::Molecule);
    }

    #[test]
    fn infers_layout_from_path() {
        assert_eq!(Tier::from_path("layouts/base.jinja"), Tier::Layout);
    }

    #[test]
    fn infers_feature_from_path() {
        assert_eq!(Tier::from_path("features/schema.jinja"), Tier::Feature);
    }

    #[test]
    fn infers_knowledge_from_md() {
        assert_eq!(Tier::from_path("knowledge/overview.md"), Tier::Knowledge);
    }
}
