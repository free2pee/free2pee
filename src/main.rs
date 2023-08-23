use futures::channel::oneshot;
use js_sys;
use js_sys::WebAssembly::Table;
use leptos::*;
use leptos::{error::Result, *};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::log_1;
use web_sys::{
    console, window, Geolocation, Navigator, Position, PositionError, PositionOptions, Window,
};
use itertools::Itertools;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverpassResponse {
    pub elements: Vec<Element>,
    pub generator: String,
    pub osm3s: Osm3s,
    pub version: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub id: i64,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub tags: HashMap<String, String>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub nodes: Option<Vec<i64>>,
    pub center: Option<Center>,
    // pub center: Option<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Center {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Osm3s {
    pub copyright: String,
    #[serde(rename = "timestamp_osm_base")]
    pub timestamp_osm_base: String,
}

impl Element {
    pub fn get_coords(&self) -> Option<(f64, f64)> {
    // pub fn get_coords(&self) -> (f64, f64) {
        if let (Some(lat), Some(lon)) = (self.lat, self.lon) {
            Some((lat, lon))
            // (lat, lon)
        } else if let Some(center) = &self.center {
            let val = serde_wasm_bindgen::to_value(center).unwrap();
            log_1(&val);
            Some((center.lat, center.lon))
            // (center.lat, center.lon)
            // Some((center["lat"].as_f64().unwrap(), center["lon"].as_f64().unwrap()))
        } else {
            // let val = serde_wasm_bindgen::to_value(self).unwrap();
            // panic!("No coords found for element: {:?}", self);
            None
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum BathroomError {
    #[error("Failed to fetch bathrooms. Please ensure that location services are enabled for device and browser.")]
    FetchBathroomsFailed,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRoot {
    pub code: String,
    pub routes: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub distance: f64,
    pub duration: f64,
    pub geometry: String,
    pub legs: Vec<Leg>,
    pub weight: f64,
    #[serde(rename = "weight_name")]
    pub weight_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leg {
    pub distance: f64,
    pub duration: f64,
    pub steps: Vec<Value>,
    pub summary: String,
    pub weight: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Waypoint {
    pub distance: f64,
    pub hint: String,
    pub location: Vec<f64>,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRoot {
    pub code: String,
    pub distances: Vec<Vec<f64>>,
    pub destinations: Vec<OSRMLocation>,
    pub durations: Vec<Vec<f64>>,
    pub sources: Vec<OSRMLocation>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OSRMLocation {
    pub hint: String,
    pub distance: f64,
    pub name: String,
    pub location: Vec<f64>,
}

pub async fn fetch_walking_data(
    origin: (f64, f64),
    destinations: Vec<(f64, f64)>,
) -> Result<RouteRoot> {
    let route_url = generate_route_url(origin, destinations);
    let response = reqwasm::http::Request::get(&route_url).send().await?;
    let json = response.json().await?;

    Ok(json)
}

pub async fn fetch_table_data(
    origin: (f64, f64),
    destinations: Vec<(f64, f64)>,
) -> Result<TableRoot> {
    let route_url = generate_table_url(origin, destinations);
    let response = reqwasm::http::Request::get(&route_url).send().await?;
    // let jsonval: Value = response.json().await?;
    // let val = serde_wasm_bindgen::to_value(&jsonval).unwrap();
    // console::log_1(&val);
    let json = response.json().await?;

    Ok(json)
}

fn generate_route_url(origin: (f64, f64), destinations: Vec<(f64, f64)>) -> String {
    let (lat, lon) = origin;
    let mut route_url = format!(
        "https://routing.openstreetmap.de/routed-foot/route/v1/driving/{},{}",
        lon, lat
    );

    for (lat_dest, lon_dest) in destinations {
        route_url.push_str(&format!(";{},{}", lon_dest, lat_dest));
    }

    route_url
}

fn generate_table_url(origin: (f64, f64), destinations: Vec<(f64, f64)>) -> String {
    let (lat, lon) = origin;
    // let mut https://router.project-osrm.org/table/v1/driving/13.388860,52.517037;13.397634,52.529407;13.428555,52.523219?annotations=distance,duration&sources=0
    let mut route_url = format!(
        "https://router.project-osrm.org/table/v1/driving/{},{}",
        lon, lat
    );

    for (lat_dest, lon_dest) in destinations {
        route_url.push_str(&format!(";{},{}", lon_dest, lat_dest));
    }
    route_url.push_str("?annotations=distance,duration&sources=0");

    route_url
}

pub fn extract_distances(json: &RouteRoot) -> Result<Vec<f64>> {
    let distances: Vec<f64> = json.routes[0].legs.iter().map(|leg| leg.distance).collect();

    Ok(distances)
}

pub async fn walking_time_distance(
    origin: (f64, f64),
    destinations: Vec<(f64, f64)>,
) -> Result<Vec<f64>> {
    let json = fetch_walking_data(origin, destinations).await?;
    let distances = extract_distances(&json)?;
    Ok(distances)
}

fn format_query(radius: i32, lat: f64, lon: f64, q: &str) -> String {
    format!(
        "[out:json];(node[\"amenity\"=\"{q}\"](around:{},{},{});way[\"amenity\"=\"{q}\"](around:{},{},{}););out body;out center;",
        radius, lat, lon, radius, lat, lon
    )
}

fn query_url(radius: i32, lat: f64, lon: f64, q: &str) -> String {
    let query = format_query(radius, lat, lon, q);
    console::log_1(&serde_wasm_bindgen::to_value(&query).unwrap());
    format!("https://overpass-api.de/api/interpreter?data={}", query)
}

pub async fn fetch_bathrooms(_: ()) -> Result<(Vec<Element>, TableRoot, (u32, f64, f64, String))> {
    let (sender, receiver) = oneshot::channel::<Result<(f64, f64), BathroomError>>();
    let sender = Arc::new(Mutex::new(Some(sender)));

    let sender_clone = Arc::clone(&sender);
    let success_callback = Closure::wrap(Box::new(move |pos: Position| {
        let lat = pos.coords().latitude();
        let lon = pos.coords().longitude();
        log!("lat: {}, lon: {}", lat, lon);
        if let Some(sender) = sender_clone.lock().unwrap().take() {
            let _ = sender.send(Ok((lat, lon)));
        }
    }) as Box<dyn FnMut(Position)>);

    let sender_clone = Arc::clone(&sender);
    let error_callback = Closure::wrap(Box::new(move |_err: PositionError| {
        if let Some(sender) = sender_clone.lock().unwrap().take() {
            let _ = sender.send(Err(BathroomError::FetchBathroomsFailed));
        }
    }) as Box<dyn FnMut(PositionError)>);

    let navigator = window().unwrap().navigator();
    let geolocation = navigator.geolocation().unwrap();
    geolocation
        .get_current_position_with_error_callback(
            success_callback.as_ref().unchecked_ref(),
            Some(error_callback.as_ref().unchecked_ref()),
        )
        .unwrap();

    success_callback.forget();
    error_callback.forget();

    let coords = receiver.await.unwrap()?; // Propagate the BathroomError if we got one

    let (mut lat, mut lon) = coords;

    let location = window().unwrap().location();
    let search = location.search().unwrap();

    // Parse the query parameters.
    let search_params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();

    // Get the radius, or default to 2000 if not provided.
    let radius: u32 = search_params
        .get("around")
        .unwrap_or_else(|| "2000".to_string())
        .parse()
        .unwrap_or(2000);

    if let Some(lat_qsp) = search_params.get("lat") {
        if let Ok(new_lat) = lat_qsp.parse::<f64>() {
            lat = new_lat;
        }
    }

    if let Some(lon_qsp) = search_params.get("lon") {
        if let Ok(new_lon) = lon_qsp.parse::<f64>() {
            lon = new_lon;
        }
    }
    let mut amenity = "toilets".to_string();
    if let Some(q_qp) = search_params.get("q") {
        amenity = q_qp;        
    }

    let url = query_url(radius as i32, lat, lon, &amenity);
    let res: OverpassResponse = reqwasm::http::Request::get(&url)
        .send()
        .await
        .unwrap()
        .json::<OverpassResponse>()
        .await
        .unwrap();

    // let res2 = reqwasm::http::Request::get(&url)
    //     .send()
    //     .await
    //     .unwrap()
    //     .json::<Value>()
    //     .await
    //     .unwrap();

    // let val = serde_wasm_bindgen::to_value(&res).unwrap();
    // console::log_1(&val);
    // let val2 = serde_wasm_bindgen::to_value(&res2).unwrap();
    // console::log_1(&val2);
    // let nels = res.elements.len();
    let els = &res.elements;
    let good_elements = els.iter().filter(|e| e.get_coords().is_some()).cloned().collect::<Vec<_>>();
    // let dedup = good_elements.
    let unique_elements: Vec<_> = good_elements.iter()
    .unique_by(|e| e.id).cloned() // Replace with your uniqueness criteria
    .collect();
    let val2 = serde_wasm_bindgen::to_value(&format!("Number of elements: {}", els.len())).unwrap();
    console::log_1(&val2);
    let val3 = serde_wasm_bindgen::to_value(&format!("Number of good elements: {}", good_elements.len())).unwrap();
    console::log_1(&val3);
    let val4 = serde_wasm_bindgen::to_value(&format!("Number of good unique: {}", unique_elements.len())).unwrap();
    console::log_1(&val4);
    // let destinations: Vec<_> = res.elements.iter().map(|e| e.get_coords()).collect();
    let destinations: Vec<_> = unique_elements.iter().map(|e| e.get_coords().unwrap()).collect();
    let val3 = serde_wasm_bindgen::to_value(&format!("Number of destinations: {}", destinations.len())).unwrap();
    console::log_1(&val3);

    let json = fetch_table_data((lat, lon), destinations).await?;

    let val = serde_wasm_bindgen::to_value(&json).unwrap();
    console::log_1(&val);
    

    Ok((unique_elements.to_owned(), json, (radius, lat, lon, amenity)))
}

pub fn fetch_example(cx: Scope) -> impl IntoView {
    let bathrooms = create_local_resource(cx, || {}, fetch_bathrooms);

    let fallback = move |cx, errors: RwSignal<Errors>| {
        let error_list = move || {
            errors.with(|errors| {
                errors
                    .iter()
                    .map(|(_, e)| view! { cx, <li>{e.to_string()}</li> })
                    .collect_view(cx)
            })
        };

        view! { cx,
            <div class="error">
                <h2>"Error: you have been deemed unworthy of peeing!"</h2>
                <ul>{error_list}</ul>
                <p>
                    If youre seeing missing field distances at line 1 column 85 then this is likely just that there are no bathrooms around you! In this case it is recommended to pee outside!! Take it in, each piss is a gift
                </p>
                <p>
                    Submit suggestions/bugs at
                    <a href="https://github.com/free2pee/free2pee" target="_blank">
                        GitHub Source
                    </a>
                </p>
            </div>
        }
    };

    let bathrooms_view = move || {
        bathrooms.read(cx).map(|data| {
            data.map(|data| {
                let (elements, routing_json, (radius, lat, lon, q)) = data;
                let now = js_sys::Date::new_0(); //.to_json();
                let date_string = now.to_locale_time_string("en-US"); //.to_string();
                // let route_str = serde_json::to_string_pretty(&routing_json).unwrap();
                let dists = &routing_json.distances[0];
                let x = serde_wasm_bindgen::to_value(&format!("nels: {}, ndists: {:?}", elements.len(), dists.len())).unwrap();
                log_1(&x);
                assert_eq!(dists.len(), elements.len() + 1);
                // let durs = &routing_json.durations[0];

                let mut bathroom_data: Vec<_> = elements
                    .iter()
                    .zip(dists.iter().skip(1))
                    // .zip(durs.iter().skip(1))
                    .collect();
                
                bathroom_data.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

                let bathroom_elements = bathroom_data.iter().map(|(element, dist)| {
                // let bathroom_elements = bathroom_data.iter().filter_map(|(element, dist)| {
                    let s = format!("{:?}", element.tags);
                    let val = serde_wasm_bindgen::to_value(&element).unwrap();
                    log_1(&val);
                    let (el_lat, el_lon) = element.get_coords().unwrap();
                    // if let Some((el_lat, el_lon)) = element.get_coords() {
                        // Some(view! { cx,
                        view! { cx,
                            <tr>
                                <td>
                                    <a
                                        href=format!(
                                            "https://www.openstreetmap.org/{}/{}", element.type_field,
                                            element.id
                                        )

                                        target="_blank"
                                    >
                                        OSM:
                                        {element.id}
                                    </a>
                                </td>
                                <td>
                                    <a
                                        // using origin looks more accurate on desktop, but i think current location origin is better for mobile
                                        href=format!(
                                            "https://www.google.com/maps/dir/?api=1&destination={},{}",
                                            el_lat, el_lon
                                        )

                                        target="_blank"
                                    >
                                        "Google Maps"
                                    </a>
                                </td>
                                <td>{format!("{:?}", dist)}</td>
                            // <td>{format!("{:?}", dur)}</td>
                            </tr>
                            <p>{s}</p>
                        }
                    
                }).collect_view(cx);

                view! { cx,
                    <h2>
                        <a href="https://github.com/free2pee/free2pee" target="_blank">FREE2PEE</a>
                    </h2>
                    <h2>
                        {format!("Bathrooms accessed at {} around {},{}", date_string, lat, lon)}
                    </h2>
                    // <span style="width: 10px; display: inline-block;"></span>
                    <table>
                        <thead>
                            <tr>
                                // <th>"Node lat,lon"</th>
                                <th>"OSM Node"</th>
                                <th>"Directions"</th>
                                <th title="Distance in meters">"Distance [m]"</th>
                            // <th>"Duration [s]"</th>
                            </tr>
                        </thead>
                        <tbody>{bathroom_elements}</tbody>
                    </table>
                    <a
                        href=format!(
                            "https://mapcomplete.osm.be/toilets.html?z=18&lat={lat}&lon=-{lon}"
                        )

                        target="_blank"
                    >
                        View a map of nearby bathrooms
                    </a>
                }
            })
        })
    };

    view! { cx,
        <div>
            <ErrorBoundary fallback>
                <Transition fallback=move || {
                    view! { cx, <div>"Loading (Suspense Fallback)..."</div> }
                }>
                    <div>{bathrooms_view}</div>
                </Transition>
            </ErrorBoundary>
        </div>
    }
}

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(fetch_example)
}

fn f2p_url(lat: f64, lon: f64) -> String {
    format!(
        "https://free2pee.github.io/free2pee/?lat={}&lon={}",
        lat, lon
    )
}

fn f2plocal_url(lat: f64, lon: f64) -> String {
    format!(
        "https://127.0.0.1:8080/free2pee/?lat={}&lon={}",
        lat, lon
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_points() {
        let pts = vec![
        (38.5448, -121.7523), // davis tennis court 
        (38.085731, -122.110744), // chevron 
        (38.5412308, -121.7485762) // idk 
        ];
        
        for (lat, lon) in pts {
            let url = query_url(2000, lat, lon, "toilets");
            println!("{}", url);
            println!("{}", f2plocal_url(lat, lon));
            // let res = reqwasm::http::Request::get(&url)
            //     .send()
            //     .await
            //     .unwrap()
            //     .json::<OverpassResponse>()
            //     .await
            //     .unwrap();
            // let destinations: Vec<_> = res.elements.iter().filter_map(|e| e.get_coords()).collect();
            // let json = fetch_table_data((lat, lon), destinations).await.unwrap();
        }
        // let (lat, lon)
    }
}
