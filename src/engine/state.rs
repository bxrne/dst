use crate::substrate::Substrate;

pub struct EngineState<S: Substrate> {
    pub subjects: Vec<(String, S::SubjectData)>,
    pub subject_hosts: std::collections::HashMap<String, String>,
    pub seed: Option<u64>,
}

impl<S: Substrate> Default for EngineState<S> {
    fn default() -> Self {
        Self {
            subjects: Vec::new(),
            subject_hosts: std::collections::HashMap::new(),
            seed: None,
        }
    }
}
