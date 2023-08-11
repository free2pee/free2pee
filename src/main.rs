use js_sys::WebAssembly::Table;
use leptos::*;
use leptos::{error::Result, *};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, Geolocation, Navigator, Position, PositionOptions, PositionError, Window};
use futures::channel::oneshot;
use std::sync::{Arc, Mutex};
use js_sys;

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
pub struct Osm3s {
    pub copyright: String,
    #[serde(rename = "timestamp_osm_base")]
    pub timestamp_osm_base: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub id: i64,
    pub lat: f64,
    pub lon: f64,
    pub tags: HashMap<String, String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Error, Clone, Debug)]
pub enum BathroomError {
    #[error("Failed to fetch bathrooms.")]
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

pub async fn fetch_walking_data(origin: (f64, f64), destinations: Vec<(f64, f64)>) -> Result<RouteRoot> {
    let route_url = generate_route_url(origin, destinations);
    let response = reqwasm::http::Request::get(&route_url).send().await?;
    let json = response.json().await?;

    Ok(json)
}
pub async fn fetch_table_data(origin: (f64, f64), destinations: Vec<(f64, f64)>) -> Result<TableRoot> {
    let route_url = generate_table_url(origin, destinations);
    let response = reqwasm::http::Request::get(&route_url).send().await?;
    let json = response.json().await?;

    Ok(json)
}

fn generate_route_url(origin: (f64, f64), destinations: Vec<(f64, f64)>) -> String {
    let (lat, lon) = origin;
    let mut route_url = format!("https://routing.openstreetmap.de/routed-foot/route/v1/driving/{},{}", lon, lat);

    for (lat_dest, lon_dest) in destinations {
        route_url.push_str(&format!(";{},{}", lon_dest, lat_dest));
    }

    route_url
}

fn generate_table_url(origin: (f64, f64), destinations: Vec<(f64, f64)>) -> String {
    let (lat, lon) = origin;
    // let mut https://router.project-osrm.org/table/v1/driving/13.388860,52.517037;13.397634,52.529407;13.428555,52.523219?annotations=distance,duration&sources=0
    let mut route_url = format!("https://router.project-osrm.org/table/v1/driving/{},{}", lon, lat);

    for (lat_dest, lon_dest) in destinations {
        route_url.push_str(&format!(";{},{}", lon_dest, lat_dest));
    }
    route_url.push_str("?annotations=distance,duration&sources=0");

    route_url
}

pub fn extract_distances(json: &RouteRoot) -> Result<Vec<f64>> {
    let distances: Vec<f64> = json.routes[0].legs
        .iter()
        .map(|leg| leg.distance)
        .collect();

    Ok(distances)
}

pub async fn walking_time_distance(origin: (f64, f64), destinations: Vec<(f64, f64)>) -> Result<Vec<f64>> {
    let json = fetch_walking_data(origin, destinations).await?;
    let distances = extract_distances(&json)?;
    Ok(distances)
}

pub async fn fetch_bathrooms(_: ()) -> Result<(OverpassResponse, TableRoot, (f64, f64))> {
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
    geolocation.get_current_position_with_error_callback(
        success_callback.as_ref().unchecked_ref(),
        Some(error_callback.as_ref().unchecked_ref()),
    ).unwrap();

    success_callback.forget();
    error_callback.forget();

    let coords = receiver.await.unwrap()?; // Propagate the BathroomError if we got one

    let (lat, lon) = coords;

    let location = window().unwrap().location();
    let search = location.search().unwrap();

    // Parse the query parameters.
    let search_params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();

    // Get the radius, or default to 2000 if not provided.
    let radius: i64 = search_params.get("around").unwrap_or_else(|| "1000".to_string()).parse().unwrap_or(1000);

    let res = reqwasm::http::Request::get(&format!(
        "https://overpass-api.de/api/interpreter?data=[out:json];node[\"amenity\"=\"toilets\"](around:{radius},{lat},{lon});out;"
    ))
    .send()
    .await.unwrap()
    .json::<OverpassResponse>()
    .await.unwrap();
    let destinations = res.elements.iter().map(|e| (e.lat, e.lon)).collect();

    let json = fetch_table_data((lat, lon), destinations).await?;
    let val = serde_wasm_bindgen::to_value(&json).unwrap();

    console::log_1(&val);
    Ok((res, json, (lat, lon)))
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
                <h2>"Error"</h2>
                <ul>{error_list}</ul>
            </div>
        }
    };

    let bathrooms_view = move || {
        bathrooms.read(cx).map(|data| {
            data.map(|data| {
                let (el_data, routing_json, (lat, lon)) = data;
                    let now = js_sys::Date::new_0();//.to_json();
                    let date_string = now.to_locale_time_string("en-US");//.to_string();
                    // let routes = routing_json["routes"].as_array().unwrap();
                    // log!("routes: {:?}", routes);
                    // let route_str = format!("{:?}", routing_json.clone());
                    let route_str = serde_json::to_string_pretty(&routing_json).unwrap();
                    let dists = &routing_json.distances[0];
                    let durs = &routing_json.durations[0];
                    let mut bathroom_data: Vec<_> = el_data.elements.iter()
                        .zip(dists.iter().skip(1))
                        .zip(durs.iter().skip(1))
                        .collect();
                    bathroom_data.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

                    let bathroom_elements = bathroom_data.iter().map(|((element, dist), dur)| {
                    let s = format!("{:?}", element.tags);
                    view! { cx,
                        <tr>
                        // <td>
                        // {format!("{},{}",element.lat, element.lon)}
                        // </td>
                        <td>
                        <a href={format!("https://www.openstreetmap.org/node/{}", element.id)} target="_blank">OSM:{element.id}</a>
                        </td>
                        <td>
                        // using origin looks more accurate on desktop, but i think current location origin is better for mobile
                            // <a href={format!("https://www.google.com/maps/dir/?api=1&origin={lat},{lon}&destination={},{}", element.lat, element.lon)} target="_blank">"Google Maps"</a>
                            <a href={format!("https://www.google.com/maps/dir/?api=1&destination={},{}", element.lat, element.lon)} target="_blank">"Google Maps"</a>
                        </td>
                        // <td>
                        // <a href={format!("https://www.openstreetmap.org/edit?node={}", element.id)} target="_blank">Edit OSM</a>
                        // </td>
                        <td>
                            {format!("{:?}", dist)}
                        </td>
                        <td>
                            {format!("{:?}", dur)}
                        </td>
                        </tr>
                        <p>{s}</p>
                        }
                    }).collect_view(cx);
                    
                    view! { cx,
                        <h2> {format!("FREE2PEE: Bathrooms accessed at {} around {},{}", date_string, lat, lon)} </h2>
                        <a href={format!("https://mapcomplete.osm.be/toilets.html?z=18&lat={lat}&lon=-{lon}")} target="_blank">Open in MapComplete</a>
                        <table>
                        <thead>
                        <tr>
                        // <th>"Node lat,lon"</th>
                        <th>"OSM Node"</th>
                        <th>"Directions"</th>
                        <th>"Distance [m]"</th>
                        <th>"Duration [s]"</th>
                        </tr>
                        </thead>
                        <tbody>
                        {bathroom_elements}
                        </tbody>
                        </table>
                        // <p>{route_str}</p>

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
                <div>
                    {bathrooms_view}
                </div>
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
