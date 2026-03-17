use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkRegistry {
    pub framework: String,
    pub version: String,
    pub slug: String,
    pub category: FrameworkCategory,
    pub docs: String,
    #[serde(default)]
    pub github: Option<String>,
    pub structure: ProjectStructure,
    #[serde(default)]
    pub generators: Vec<GeneratorInfo>,
    pub conventions: Conventions,
    #[serde(default)]
    pub injection_points: Vec<InjectionPoint>,
    #[serde(default)]
    pub integrations: Vec<Integration>,
    #[serde(default)]
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FrameworkCategory {
    Framework,
    Orm,
    Auth,
    Ui,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStructure {
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub src: Option<String>,
    #[serde(default)]
    pub routes: Option<String>,
    #[serde(default)]
    pub components: Option<String>,
    #[serde(default)]
    pub lib: Option<String>,
    #[serde(default)]
    pub config: Option<String>,
    #[serde(default)]
    pub server_functions: Option<String>,
    #[serde(default)]
    pub db: Option<String>,
    #[serde(default)]
    pub queries: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorInfo {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Conventions {
    #[serde(default)]
    pub files: std::collections::HashMap<String, FileConvention>,
    #[serde(default)]
    pub naming: NamingConvention,
    #[serde(default)]
    pub patterns: Vec<PatternInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConvention {
    pub pattern: String,
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamingConvention {
    #[serde(default)]
    pub files: Option<String>,
    #[serde(default)]
    pub components: Option<String>,
    #[serde(default)]
    pub functions: Option<String>,
    #[serde(default)]
    pub variables: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInfo {
    pub name: String,
    pub pattern: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionPoint {
    pub region: String,
    pub marker: String,
    #[serde(default)]
    pub end_marker: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub package: String,
    #[serde(default)]
    pub install: Option<String>,
    #[serde(default)]
    pub setup: Vec<SetupStep>,
    #[serde(default)]
    pub patterns: Vec<PatternInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStep {
    pub file: String,
    pub template: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub topic: String,
    pub answer: String,
    #[serde(default)]
    pub steps: Vec<QuestionStep>,
    #[serde(default)]
    pub files_affected: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub learn_more: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionStep {
    pub action: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkInfo {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub docs: String,
    pub category: FrameworkCategory,
    #[serde(default)]
    pub github: Option<String>,
}

impl From<&FrameworkRegistry> for FrameworkInfo {
    fn from(reg: &FrameworkRegistry) -> Self {
        Self {
            slug: reg.slug.clone(),
            name: reg.framework.clone(),
            version: reg.version.clone(),
            description: format!("{} v{}", reg.framework, reg.version),
            docs: reg.docs.clone(),
            category: reg.category.clone(),
            github: reg.github.clone(),
        }
    }
}
