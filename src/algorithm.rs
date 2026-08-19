pub enum Algorithm {
    MLD,
    CH,
}

impl Algorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::MLD => "MLD",
            Algorithm::CH => "CH",
        }
    }
}

