use derive_builder::Builder;

#[derive(Debug, Clone, Copy, Builder)]
pub struct Point {
    pub latitude: f64,
    pub longitude: f64,
}
