#[derive(Copy, Clone)]
pub enum Field {
    DataScience,
    HumanComputerInteraction,
    HighPerformanceComputing,
    ScientificComputing,
    Algorithms,
    TheoryOfComputation,
    ComputerArchitecture,
    Networking,
    Robotics,
    NaturalLanguageProcessing,
    ComputerVision,
    EmbeddedSystems,
    MachineLearning,
    Cybersecurity,
    SoftwareEngineering,
    LanguagesAndCompilers,
    OperatingSystems,
    Databases,
    DistributedSystems,
    ComputerGraphics,
    QuantumComputing,
}

pub struct FieldName {
    #[expect(unused)]
    pub title_case: &'static str,
    pub lower_case: &'static str,
}
impl From<(&'static str, &'static str)> for FieldName {
    fn from((title_case, lower_case): (&'static str, &'static str)) -> Self {
        Self {
            title_case,
            lower_case,
        }
    }
}

impl Field {
    pub fn name(self) -> FieldName {
        match self {
            Field::DataScience => ("Data Science", "data science").into(),
            Field::ScientificComputing => ("Scientific Computing", "scientific computing").into(),
            Field::Algorithms => ("Algorithms", "algorithms").into(),
            Field::TheoryOfComputation => ("Computational Theory", "computational theory").into(),
            Field::OperatingSystems => ("Operating Systems", "operating systems").into(),
            Field::Databases => ("Databases", "databases").into(),
            Field::DistributedSystems => ("Distributed Systems", "distributed systems").into(),
            Field::ComputerGraphics => ("Graphics", "graphics").into(),
            Field::QuantumComputing => ("Quantum Computing", "quantum computing").into(),
            Field::Networking => ("Networking", "networking").into(),
            Field::Robotics => ("Robotics", "robotics").into(),
            Field::ComputerVision => ("Computer Vision", "computer vision").into(),
            Field::EmbeddedSystems => ("Embedded Systems", "embedded systems").into(),
            Field::MachineLearning => ("Machine Learning", "machine learning").into(),
            Field::Cybersecurity => ("Cybersecurity", "cybersecurity").into(),
            Field::SoftwareEngineering => ("Software Engineering", "software engineering").into(),
            Field::NaturalLanguageProcessing => {
                ("Natural Language Processing", "natural language processing").into()
            }
            Field::ComputerArchitecture => {
                ("Computer Architecture", "computer architecture").into()
            }
            Field::LanguagesAndCompilers => {
                ("Languages and Compilers", "languages and compilers").into()
            }
            Field::HumanComputerInteraction => {
                ("Human-Computer Interaction", "human-computer interaction").into()
            }
            Field::HighPerformanceComputing => {
                ("High-Performance Computing", "high-performance computing").into()
            }
        }
    }
}
