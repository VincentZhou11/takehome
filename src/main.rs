use std::fs;
use std::io::{Read, Write};
use std::time::Duration;
use axum::{http::StatusCode, Json, Router, routing::get};
use axum::body::{Body, Bytes, Empty};
use axum::extract::{Query, RawBody, State};
use axum::http::Uri;
use flate2::read::GzDecoder;
use hyper::{body, Client, Method, Request};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use urlencoding;

use carbon::CarbonResponse;
use covid::CovidResponse;
use crate::constants::CovidRegion;

mod carbon;
mod constants;
mod covid;

#[derive(Clone)]
pub struct AppState {
    carbon_url: String,
    covid_url: String,
    client: Client<HttpsConnector<HttpConnector>, Body>
}

async fn index(State(state): State<AppState>) -> String {
    String::from("homepage")
}

#[derive(Serialize, Deserialize)]
struct RegionQuery {
    region_id: u32
}

#[derive(Serialize, Deserialize)]
struct RegionResponse {
    cumulative_covid_cases: u32,
    carbon_intensity: u32
}

async fn carbon_endpoint(State(app_state): State<AppState>, Query(region_query): Query<RegionQuery>) -> Json<CarbonResponse> {
    Json(query_carbon(&app_state.client, &region_query.region_id).await)
}
async fn covid_endpoint(State(app_state): State<AppState>, Query(region_query): Query<RegionQuery>) -> Json<CovidResponse> {
    Json(query_covid(&app_state.client, &region_query.region_id).await)
}

async fn main_endpoint(app_state: State<AppState>, Query(region_query): Query<RegionQuery>) -> Json<RegionResponse> {
    let carbon_data = query_carbon(&app_state.client, &region_query.region_id).await;
    let covid_data = query_covid(&app_state.client, &region_query.region_id).await;

    Json(
        RegionResponse {
            carbon_intensity: carbon_data.data[0].data[0].intensity.forecast.clone(),
            cumulative_covid_cases: covid_data.data[0].cumulativeCases.clone()
        }
    )
}

async fn query_carbon(client: &Client<HttpsConnector<HttpConnector>, Body>, region_id: &u32) -> CarbonResponse {
    let uri = Uri::builder()
        .scheme("https")
        .authority("api.carbonintensity.org.uk")
        .path_and_query(format!("/regional/regionid/{region_id}"))
        .build()
        .unwrap();

    println!("{:?}", uri.to_string());

    let request = Request::builder()
        .uri(uri)
        .method(Method::GET)
        // .header("User-Agent", "my-awesome-agent/1.0")
        // .header("Accept", "json")
        .body(Body::default())
        .unwrap();
    
    let resp = client.request(request).await.unwrap();

    let bytes = body::to_bytes(resp.into_body()).await.unwrap();

    println!("{:?}", std::str::from_utf8(&bytes));

    serde_json::from_slice(&bytes).unwrap()
}

async fn query_covid(client: &Client<HttpsConnector<HttpConnector>, Body>, region_id: &u32) -> CovidResponse {
    let CovidRegion { region, region_type } = constants::UK_CARBON_TO_COVID_REGIONS.get(&region_id).unwrap();

    let uri = Uri::builder()
        .scheme("https")
        .authority("api.coronavirus.data.gov.uk")
        .path_and_query(
            format!("/v1/data?filters=areaName={region};areaType={region_type}&structure=").to_string() +
            urlencoding::encode("{\"date\":\"date\",\"name\":\"areaName\",\"dailyCases\":\"newCasesByPublishDate\",\"cumulativeCases\":\"cumCasesByPublishDate\",\"dailyDeaths\":\"newDeaths28DaysByPublishDate\",\"cumulativeDeaths\":\"cumDeaths28DaysByPublishDate\"}").as_ref())
        .build()
        .unwrap();

    println!("URI: {:?}", uri.to_string());

    let request = Request::builder()
        .uri(uri)
        .method(Method::GET)
        // .header("User-Agent", "PostmanRuntime/7.32.3")
        // .header("Accept", "*/*")
        // .header("Accept-Encoding", "gzip, deflate, br")
        // .header("Connection", "keep-alive")
        .body(Body::empty())
        .unwrap();

    let resp = client.request(request).await.unwrap();

    println!("Status: {:?}", resp.status());

    let encoded_bytes = body::to_bytes(resp.into_body()).await.unwrap();

    let mut gz = GzDecoder::new(&*encoded_bytes);

    let mut decoded_bytes: Vec<u8> = vec!();

    gz.read_to_end(&mut decoded_bytes);

    serde_json::from_slice(&decoded_bytes).unwrap()
}

#[tokio::main]
async fn main() {

    let app_state = AppState{
        carbon_url: "https://api.carbonintensity.org.uk/regional/regionid/".to_string(),
        covid_url: "https://api.coronavirus.data.gov.uk/v1/data".to_string(),
        client: Client::builder().build::<HttpsConnector<HttpConnector>, Body>(HttpsConnector::new())
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/carbon", get(carbon_endpoint))
        .route("/covid", get(covid_endpoint))
        .route("/main", get(main_endpoint))

        .with_state(app_state.clone());

    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}