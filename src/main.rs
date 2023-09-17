use std::fs;
use std::io::{Read, Write};
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
use chrono::{DateTime, Utc, serde::ts_seconds_option, NaiveDateTime, ParseResult, Days, Duration, NaiveDate, NaiveTime};
use serde_json::Error;

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
    region_id: u32,
    from: String,
    to: Option<String>
}

struct ParsedRegionQuery {
    region_id: u32,
    from: NaiveDateTime,
    to: NaiveDateTime
}

impl RegionQuery {
    fn parse(&self) -> ParsedRegionQuery {
        let from = if let Ok(time) = NaiveDate::parse_from_str(&self.from, "%Y-%m-%d") {
            time
        } else {
            println!("{:?}", self.from);
            panic!("Unable to parse `from` date")
        };

        let to = if let Some(time) = &self.to {
            if let Ok(time) = NaiveDate::parse_from_str(&time, "%Y-%m-%d") {
                time
            } else {
                panic!("Unable to parse `to` date")
            }
        } else {
            if let Ok(time) = NaiveDate::parse_from_str(&self.from, "%Y-%m-%d") {
                time + Duration::days(1)
            } else {
                panic!("Unable to parse default `to` date")
            }
        };

        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();


        ParsedRegionQuery{region_id: self.region_id.clone(), from: NaiveDateTime::new(from, time), to: NaiveDateTime::new(to, time) }
    }
}

#[derive(Serialize, Deserialize)]
struct RegionResponse {
    region: Option<String>,
    data: Option<Vec<RegionData>>,
    error: Option<String>
}
#[derive(Serialize, Deserialize)]
struct RegionData {
    date: NaiveDate,
    cumulative_covid_cases: Option<u32>,
    carbon_intensity: Option<u32>,
}

async fn carbon_endpoint(State(app_state): State<AppState>, Query(region_query): Query<RegionQuery>) -> Json<CarbonResponse> {
    let parsed_region_query = region_query.parse();
    Json(query_carbon(&app_state.client, &region_query.region_id, parsed_region_query.from).await.unwrap())
}
async fn covid_endpoint(State(app_state): State<AppState>, Query(region_query): Query<RegionQuery>) -> Json<CovidResponse> {
    let parsed_region_query = region_query.parse();
    Json(query_covid(&app_state.client, &region_query.region_id,parsed_region_query.from).await.unwrap())
}

async fn main_endpoint(app_state: State<AppState>, Query(region_query): Query<RegionQuery>) -> Json<RegionResponse> {
    let parsed_region_query = region_query.parse();

    if !constants::UK_CARBON_REGIONS.contains_key(&parsed_region_query.region_id) {
        return Json(RegionResponse {
            region: None,
            data: None,
            error: Some("Invalid region id".to_string()),
        })
    }
    if parsed_region_query.from > parsed_region_query.to {
        return Json(RegionResponse {
            region: None,
            data: None,
            error: Some("`to` date is before `from` date ".to_string()),
        })
    }

    // let carbon_data = query_carbon(&app_state.client, &region_query.region_id, parsed_region_query.from).await;
    // let covid_data = query_covid(&app_state.client, &region_query.region_id, parsed_region_query.from).await;

    println!("{:?}, {:?}", parsed_region_query.from, parsed_region_query.to);

    let mut region_data_vec:Vec<RegionData> = vec!();
    let mut curr = parsed_region_query.from;
    while curr <= parsed_region_query.to {
        if let Ok(carbon_data) = query_carbon(&app_state.client, &region_query.region_id, curr).await {
            if let Ok(covid_data) = query_covid(&app_state.client, &region_query.region_id, curr).await {
                region_data_vec.push(RegionData{
                    date:curr.date(),
                    cumulative_covid_cases: Some(covid_data.data[0].cumulativeCases.clone()),
                    carbon_intensity: Some(carbon_data.data.data[0].intensity.forecast.clone())
                })
            }
            else {
                println!("None covid data")
            }
        }
        else {
            println!("No carbon data")
        }

        println!("{:?}", curr);
        curr += Duration::days(1);
    }


    Json(
        RegionResponse {
            region: Some(constants::UK_CARBON_REGIONS.get(&parsed_region_query.region_id).unwrap().to_string()),
            data: Some(region_data_vec),
            error: None,
        }
    )
}

async fn query_carbon(client: &Client<HttpsConnector<HttpConnector>, Body>, region_id: &u32, date: NaiveDateTime) -> Result<CarbonResponse, Error> {
    let from_string = date.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let to_string = (date + Duration::days(1)).format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let uri = Uri::builder()
        .scheme("https")
        .authority("api.carbonintensity.org.uk")
        .path_and_query(format!("/regional/intensity/{from_string}/{to_string}/regionid/{region_id}"))
        .build()
        .unwrap();

    println!("URI: {:?}", uri.to_string());

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

    serde_json::from_slice(&bytes)
}

async fn query_covid(client: &Client<HttpsConnector<HttpConnector>, Body>, region_id: &u32, date: NaiveDateTime) -> Result<CovidResponse, Error> {
    let CovidRegion { region, region_type } = constants::UK_CARBON_TO_COVID_REGIONS.get(&region_id).unwrap();

    let encoded_region = urlencoding::encode(region);

    let date_string = date.format("%Y-%m-%d").to_string();

    println!("URI elements: {:?}", format!("/v1/data?filters=areaName={encoded_region};areaType={region_type};date={date_string}&structure=").to_string());

    let uri = Uri::builder()
        .scheme("https")
        .authority("api.coronavirus.data.gov.uk")
        .path_and_query(
            format!("/v1/data?filters=areaName={encoded_region};areaType={region_type};date={date_string}&structure=").to_string() +
            urlencoding::encode("{\"date\":\"date\",\"name\":\"areaName\",\"dailyCases\":\"newCasesByPublishDate\",\"cumulativeCases\":\"cumCasesByPublishDate\"}").as_ref())
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

    // println!("Status: {:?}", resp.status());

    let encoded_bytes = body::to_bytes(resp.into_body()).await.unwrap();

    let mut gz = GzDecoder::new(&*encoded_bytes);

    let mut decoded_bytes: Vec<u8> = vec!();
    gz.read_to_end(&mut decoded_bytes);

    println!("{:?}", std::str::from_utf8(&decoded_bytes));

    serde_json::from_slice(&decoded_bytes)
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