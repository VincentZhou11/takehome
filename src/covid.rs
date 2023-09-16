use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CovidResponse {
    pub(crate) data: Vec<Data>
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Data {
    pub(crate) date: String,
    pub(crate) name: String,
    pub(crate) dailyCases: u32,
    pub(crate) cumulativeCases: u32,
    pub(crate) dailyDeaths: Option<u32>,
    pub(crate) cumulativeDeaths: Option<u32>
}