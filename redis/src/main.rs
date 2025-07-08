use redis::Commands;
use std::fs::File;
use std::io::Read;
use warp::Filter;
use serde_json::Value;
use std::collections::HashMap;
use shared_utils::redis_loader::load_matches_summary_data_into_redis;
use shared_utils::redis_loader::load_single_match_to_redis;

fn get_matches() -> Vec<String> {
    let mut contents = String::new();
    let file_path = "data/matches.json";
    if let Ok(mut file) = File::open(file_path) {
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(parsed_data) = serde_json::from_str::<Vec<HashMap<String, Value>>>(&contents) {
                return parsed_data.iter()
                    .filter_map(|single_match| single_match.get("id")?.as_str().map(String::from))
                    .collect();
            }
        }
    }
    Vec::new()
}

#[tokio::main]
async fn main() {
    let match_ids = get_matches();
    dotenv::dotenv().ok();

    let _ = load_matches_summary_data_into_redis();

    for file_id in &match_ids {
        let _ = load_single_match_to_redis(&file_id);
        println!("file name {:?}", file_id);
    }

    println!("Starting warp...");
    let get_match_data = warp::path!("match" / String)
        .map(|file_id: String| {
            let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1/".to_string());
            let client = redis::Client::open(redis_url).expect("Failed to connect to Redis");
            let mut connection = client.get_connection().expect("Failed to get Redis connection");

            let key = format!("match:{}", file_id);

            match connection.get::<_, Option<String>>(&key) {
                Ok(Some(json_data)) => {
                    match serde_json::from_str::<serde_json::Value>(&json_data) {
                        Ok(parsed_data) => warp::reply::json(&parsed_data),
                        Err(_) => warp::reply::json(&serde_json::json!({"error": "Invalid JSON format"})),
                    }
                }
                Ok(None) => warp::reply::json(&serde_json::json!({"error": format!("Data not found for file_id: {}", file_id)})),
                Err(_) => warp::reply::json(&serde_json::json!({"error": "Failed to fetch data from Redis"})),
            }
        });

    let matches_summary_data = warp::path!("matches")
        .and_then(|| async move {
            let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1/".to_string());
            let client = redis::Client::open(redis_url).expect("Failed to connect to Redis");
            let mut connection = client.get_connection().expect("Failed to get Redis connection");

            let json_data: String = connection.get("match_summaries").expect("Failed to fetch data from Redis");

            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::from_str::<serde_json::Value>(&json_data).unwrap()))
        });

    let redis_url3 = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1/".to_string());
    println!("Server running at {:?}", redis_url3);

    warp::serve(get_match_data.or(matches_summary_data))
        .run(([0, 0, 0, 0], 3030))
        .await;
}
