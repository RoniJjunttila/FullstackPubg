use reqwest::{Client, header};
use serde_json;
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::env;
use dotenv::dotenv;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

const PLAYER_NAMES: [&str; 4] = [
    "E1_Duderino",
    "keken_viikset",
    "HlGHLANDER",
    "bold_moves_bob",
];

fn player_id_merge () -> String {

    let mut players_ids_merged = String::new();

    for (index, name) in PLAYER_NAMES.iter().enumerate() {
        if index == PLAYER_NAMES.len() -1 {
            players_ids_merged.push_str(name);
        } else {
            players_ids_merged.push_str(name);
            players_ids_merged.push_str("%2C");
        }
    }
    return players_ids_merged;
}

fn make_player_id_url () -> String {
    let mut player_url = String::new();
    player_url.push_str("https://api.pubg.com/shards/steam/players?filter[playerNames]=");
    let player_ids = player_id_merge();
    player_url.push_str(&player_ids);
    return player_url;
}

async fn fetch_player_url (api_key: String , season_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let player_ids_url = make_player_id_url();
    let client = Client::new();

    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/vnd.api+json".parse()?);
    headers.insert(header::AUTHORIZATION, format!("Bearer {}", api_key).parse()?);

    let response = client
          .get(player_ids_url)
          .headers(headers)
          .send()
          .await?
          .text()
          .await?;

     extract_user_id(&response, api_key, season_id).await?;

     Ok(())
}

#[derive(Debug, Deserialize)]
struct PlayerIdData {
    data: Vec<PlayerId>,
}

#[derive(Debug, Deserialize)]
struct PlayerId {
    id: String,
}

async fn extract_user_id(response: &String, api_key: String, season_id: String) -> Result<(), Box<dyn Error>> {
    let parsed: PlayerIdData = serde_json::from_str(&response)?;

    let mut player_ids: Vec<String> = Vec::new();

        for player in parsed.data {
            player_ids.push(player.id);
        }

        let combined_ids = combine_user_ids(player_ids);
        fetch_season_data(&combined_ids, api_key, season_id).await?;

        Ok(())
}

#[derive(Debug, Deserialize)]
struct SeasonData {
    data: Vec<PlayerData>,
}

#[derive(Debug, Deserialize)]
struct PlayerData {
    #[serde(rename = "type")]
    type_: String,
    attributes: PlayerAttributes,
    relationships: Relationships
}

#[derive(Debug, Deserialize)]
struct PlayerAttributes {
    #[serde(rename = "gameModeStats")]
        game_mode_stats: HashMap<String, GameModeStats>,
}

#[derive(Debug, Deserialize)]
struct Relationships {
    player: Player,
}

#[derive(Debug, Deserialize)]
struct Player {
    data: PlayerIdFromSeasonData,
}

#[derive(Debug, Deserialize)]
struct PlayerIdFromSeasonData {
    id: String,
}

#[derive(Serialize, Debug, Deserialize)]
struct GameModeStats {
    assists: u32,
    boosts: u32,
    #[serde(rename = "dBNOs")]
        dbnos: u32,

        #[serde(rename = "dailyKills")]
        daily_kills: u32,

        #[serde(rename = "dailyWins")]
        daily_wins: u32,

        #[serde(rename = "damageDealt")]
        damage_dealt: f64,

        days: u32,

        #[serde(rename = "headshotKills")]
        headshot_kills: u32,

        heals: u32,
        kills: u32,

        #[serde(rename = "longestKill")]
        longest_kill: f64,

        #[serde(rename = "longestTimeSurvived")]
        longest_time_survived: u32,

        losses: u32,

        #[serde(rename = "maxKillStreaks")]
        max_kill_streaks: u32,

        #[serde(rename = "mostSurvivalTime")]
        most_survival_time: u32,

        #[serde(rename = "rankPoints")]
        rank_points: i32,

        #[serde(rename = "rankPointsTitle")]
        rank_points_title: String,

        revives: u32,

        #[serde(rename = "rideDistance")]
        ride_distance: f64,

        #[serde(rename = "roadKills")]
        road_kills: u32,

        #[serde(rename = "roundMostKills")]
        round_most_kills: u32,

        #[serde(rename = "roundsPlayed")]
        rounds_played: u32,

        suicides: u32,

        #[serde(rename = "swimDistance")]
        swim_distance: f64,

        #[serde(rename = "teamKills")]
        team_kills: u32,

        #[serde(rename = "timeSurvived")]
        time_survived: u32,

        top10s: u32,

        #[serde(rename = "vehicleDestroys")]
        vehicle_destroys: u32,

        #[serde(rename = "walkDistance")]
        walk_distance: f64,

        #[serde(rename = "weaponsAcquired")]
        weapons_acquired: u32,

        #[serde(rename = "weeklyKills")]
        weekly_kills: u32,

        #[serde(rename = "weeklyWins")]
        weekly_wins: u32,

        #[serde(rename = "winPoints")]
        win_points: u32,

        wins: u32,
}

async fn fetch_season_data (player_id_query_params: &str, api_key: String, season_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let player_stat_url = "https://api.pubg.com/shards/steam/seasons/".to_owned() + &season_id + "/gameMode/squad-fpp/players?filter[playerIds]=" + &player_id_query_params;

    let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/vnd.api+json".parse()?);
        headers.insert(header::AUTHORIZATION, format!("Bearer {}", api_key).parse()?);

    let response = client
        .get(player_stat_url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let parsed: SeasonData = serde_json::from_str(&response)?;
    let mut player_data_map: HashMap<String, Option<&GameModeStats>> = HashMap::new();
    for (index, player_data) in parsed.data.iter().enumerate() {
      //  let player_id = &player_data.relationships.player.data.id;
        let player_season_data = &player_data.attributes;
        let player_name = PLAYER_NAMES[index];
        let player_stats = player_season_data.game_mode_stats.get("squad-fpp");
        player_data_map.insert(player_name.to_string(), player_stats);
    }
    const FILENAME: &str = "test.json" ;
    let _ = save_to_json(player_data_map, FILENAME);
    Ok(())
}

fn save_to_json(data: HashMap<String, Option<&GameModeStats>>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_data = serde_json::to_string_pretty(&data)?;
    let mut file = File::create(filename)?;
    file.write_all(json_data.as_bytes())?;
    println!("success");
    Ok(())
}

fn combine_user_ids(player_ids: Vec<String>) -> String {
    let mut combined_ids = String::new();
    for (index, id) in player_ids.iter().enumerate() {
        if index == player_ids.len() - 1 {
            combined_ids.push_str(id);
        } else {
            combined_ids.push_str(id);
            combined_ids.push_str("%2C");
        }
    }
    return combined_ids;
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = env::var("api_key").expect("Api key not found...");
    let season_id = env::var("season_id").expect("Season id not found...");
    if let Err(e) = fetch_player_url(api_key, season_id).await {
        eprintln!("Error {}", e);
    }
}
