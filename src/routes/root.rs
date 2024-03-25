use crate::{AppState, TEMPLATES};
use axum::{extract::State, response::Html};
use chrono::{DateTime, Local, TimeZone};
use std::sync::Arc;

/// Handler to return the website's index
pub async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut context = tera::Context::new();

    context.insert("title", &state.title);

    let data = reqwest::get("https://www.ndbc.noaa.gov/data/realtime2/MLWW3.txt")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let latest = data.lines().collect::<Vec<_>>();
    let latest = latest.get(2).unwrap();

    let (time, measurements) = latest.split_at(16);

    let time = time.trim().split(" ").collect::<Vec<_>>();
    let time: DateTime<Local> = chrono::Utc
        .with_ymd_and_hms(
            time.get(0).unwrap().parse::<i32>().unwrap(),
            time.get(1).unwrap().parse::<u32>().unwrap(),
            time.get(2).unwrap().parse::<u32>().unwrap(),
            time.get(3).unwrap().parse::<u32>().unwrap(),
            time.get(4).unwrap().parse::<u32>().unwrap(),
            00,
        )
        .unwrap()
        .try_into()
        .unwrap();

    let mut measurements = measurements.trim().split_whitespace();
    let wind_direction = measurements.next().unwrap();

    let wind_speed = measurements.next().unwrap();
    let gusts = measurements.next().unwrap();

    let mut time = time.to_rfc2822();
    time = time.split(" -").next().unwrap().to_string();

    context.insert("time", &time);
    context.insert("wind_direction", wind_direction);
    context.insert("wind_speed", wind_speed);
    context.insert("gusts", gusts);

    match TEMPLATES.render("index.html", &context) {
        Ok(s) => Html(s),
        Err(_) => Html("<html><body>Error</body></html>".to_string()),
    }
}
