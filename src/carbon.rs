use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CarbonResponse {
    pub(crate) data: Data
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Data {
    pub(crate) data: Vec<RegionData>
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RegionData {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) intensity: Intensity,
    pub(crate) generationmix: Vec<Generation>
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Intensity {
    pub(crate) forecast: u32,
    pub(crate) index: String
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Generation {
    pub(crate) fuel: String,
    pub(crate) perc: f32
}