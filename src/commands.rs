#[derive(Debug, Clone)]
pub enum Function {
    OpenApp(String),
    RunShellCommand(Vec<String>),
    RandomVar(i32),
    GoogleSearch(String),
    Quit,
}
