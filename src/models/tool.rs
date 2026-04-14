#[derive(Clone, Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub source: String,
    pub version: String,
    pub binary: Option<String>,
    pub install_command: String,
    pub run_command: String,
}
