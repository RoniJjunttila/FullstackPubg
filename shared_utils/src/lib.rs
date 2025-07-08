pub mod redis_loader {
    use redis::Commands;
    use std::fs::File;
    use std::io::Read;

    pub fn load_matches_summary_data_into_redis() -> redis::RedisResult<()> {
        let file_name = "data/matches.json";

        let mut file = File::open(&file_name).expect("Failed to open JSON file");

        let mut json_data = String::new();
        file.read_to_string(&mut json_data).expect("Failed to read data from file");

        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string())).expect("Failed to connect to Redis");
        let mut connection = client.get_connection().expect("Failed to get Redis conncetion");

        let _ : () = connection.set("match_summaries", json_data)?;
        println!("Updated Redis with summary data.");
        Ok(())
    }

    pub fn load_single_match_to_redis(match_id: &str) -> redis::RedisResult<()> {
        let file_path = format!("data/matches/{}.json", match_id);
        let mut file = File::open(&file_path).expect("Failed to open match JSON file");

        let mut json_data = String::new();
        file.read_to_string(&mut json_data).expect("Failed to read data from file");
        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string())).expect("Failed to connect to Redis");
        let mut connection = client.get_connection().expect("Failed to get Redis conncetion");

        let key = format!("match:{}", match_id);
        let _ : () = connection.set(key, json_data)?;
         println!("Loaded match id {:?} into Redis", match_id);

         Ok(())
    }

    pub fn delete_single_match_from_redis(match_id: &str) -> redis::RedisResult<()> {
        let client = redis::Client::open(
               std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string())
           ).expect("Failed to connect to Redis");

        let mut connection = client.get_connection().expect("Failed to get Redis connection");

        let key = format!("match:{}", match_id);
        let _: () = connection.del(key)?;

        println!("Deleted match id {:?} from Redis", match_id);

        Ok(())
    }
}
