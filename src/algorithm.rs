/// The preprocessing algorithm used by an OSRM dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Algorithm {
    /// Multi-Level Dijkstra (`osrm-partition` followed by `osrm-customize`).
    MLD,
    /// Contraction Hierarchies (`osrm-contract`).
    CH,
}

impl Algorithm {
    pub const fn as_str(self) -> &'static str {
        match self {
            Algorithm::MLD => "MLD",
            Algorithm::CH => "CH",
        }
    }
}
