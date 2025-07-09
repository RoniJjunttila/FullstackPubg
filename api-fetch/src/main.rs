use reqwest::{Client, header};
use serde_json::{json, Value, from_str};
use serde::{Serialize, Deserialize, Deserializer};
//use std::error::Error;
use std::env;
use dotenv::dotenv;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashSet;
use chrono::{DateTime};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
mod constants;
use shared_utils::redis_loader::load_matches_summary_data_into_redis;
use shared_utils::redis_loader::load_single_match_to_redis;
use shared_utils::redis_loader::delete_single_match_from_redis;

fn player_id_merge () -> String {
    let mut players_ids_merged = String::new();

    for (index, name) in constants::player_names::PLAYER_NAMES.iter().enumerate() {
        if index == constants::player_names::PLAYER_NAMES.len() -1 {
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

#[derive(Debug, Deserialize)]
struct PlayerIdData {
    data: Vec<PlayerId>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PlayerId {
    id: String,
    relationships: Relationships,
}

#[derive(Debug, Deserialize)]
struct Relationships {
    matches: MatchData,
}

#[derive(Debug, Deserialize)]
struct MatchData {
    data: Vec<Match>,
}

#[derive(Debug, Deserialize)]
struct Match {
    id: String,
}

async fn extract_user_id(response: &str) -> Vec<String> {
    let parsed: PlayerIdData = serde_json::from_str(response).expect("Failed to parse JSON");

    let mut match_ids: Vec<String> = Vec::new();

    for player in &parsed.data {
        if player.relationships.matches.data.is_empty() {
            println!("No nothing found (*_*)");
        } else {
            for match_entry in &player.relationships.matches.data {
                match_ids.push(match_entry.id.to_string());
            }
        }
    }
    return match_ids;
}

fn make_headers(api_key: &String) -> Result<header::HeaderMap, Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/vnd.api+json".parse()?);
    headers.insert(header::AUTHORIZATION, format!("Bearer {}", api_key).parse()?);
    return Ok(headers);
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchOverview {
    data: MatchOverviewData,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchOverviewData {
    #[serde(rename = "type")]
    match_type: String,
    id: String,
    attributes: MatchOverviewAttributes,
    relationships: MatchOverviewRelationships,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchOverviewAttributes {
    #[serde(rename = "createdAt")]
    created_at: String,

    #[serde(rename = "gameMode")]
    game_mode: String,

    #[serde(rename = "mapName")]
    map_name: String,
}

#[derive(Debug, Deserialize)]
struct MatchOverviewInclude {
    included: Vec<MatchOverviewIncluded>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum MatchOverviewIncluded {
    #[serde(rename = "asset")]
    Asset {
        id: String,
        attributes: AssetAttributes,
    },
    #[serde(rename = "participant")]
    Participant {
        id: String,
        attributes: ParticipantAttributes,
    },
    #[serde(rename = "roster")]
    Roster {
        id: String,
        attributes: RosterAttributes,
    },
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RosterAttributes {
    stats: RosterStats,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RosterStats {
    rank: i32,
    #[serde(rename = "teamId")]
    team_id: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ParticipantAttributes {
    stats: ParticipantStats,
}

#[allow(dead_code)]
#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParticipantStats {
    kills: i32,
    #[serde(rename = "DBNOs")]
    dbnos: i32,
    assists: i32,
    boosts: i32,
    #[serde(rename = "damageDealt")]
    damage_dealt: f64,
    death_type: String,
    name: String,
    #[serde(rename = "playerId")]
    player_id: String,
    #[serde(rename = "rideDistance")]
    ride_distance: f32,
    #[serde(rename = "killPlace")]
    kill_place: i32,
    #[serde(rename = "winPlace")]
    win_place: i32,
}

#[derive(Debug, Deserialize)]
struct AssetAttributes {
    #[serde(rename = "URL")]
    url: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchOverviewRelationships {
    rosters: MatchOverviewRosters,
    assets: MatchOverviewAssets,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchOverviewRosters {
    data: Vec<MatchRosterData>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchRosterData {
    id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchOverviewAssets {
    data: Vec<MatchAssetData>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MatchAssetData {
    id: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct Event {
    #[serde(rename = "attackId")]
    attack_id: Option<i32>,

    #[serde(rename = "_D")]
    event_time: Option<String>,

    #[serde(rename = "_T")]
    action: Option<ActionType>,

    //#[serde(skip_serializing_if = "Option::is_none")] ei voi olla tyhjä kun asetataan myöhemmin vasta tää arvo
    weapon: Option<AttackWeapon>,

    #[serde(rename = "fireWeaponStackCount", skip_serializing_if = "Option::is_none")]
    fire_weapon_stack_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    attacker: Option<Target>,

    #[serde(skip_serializing_if = "Option::is_none")]
    victim: Option<Target>,

    #[serde(rename = "damageReason", skip_serializing_if = "Option::is_none")]
    damage_reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    damage: Option<f32>,

    #[serde(rename = "damageTypeCategory", skip_serializing_if = "Option::is_none")]
    damage_type_category: Option<String>,

    #[serde(rename = "damageCauserName", skip_serializing_if = "Option::is_none")]
    damage_causer_name: Option<String>,

    #[serde(rename = "isSuicide", skip_serializing_if = "Option::is_none")]
    is_suicide: Option<bool>,

    #[serde(rename = "dBNOMaker", skip_serializing_if = "Option::is_none")]
    dbno_maker: Option<Target>,
    #[serde(rename = "dBNODamageInfo", skip_serializing_if = "Option::is_none")]
    dbno_damage_info: Option<AddionalDamageInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    finisher: Option<Target>,
    #[serde(rename = "finishDamageInfo", skip_serializing_if = "Option::is_none")]
    finish_damage_info: Option<AddionalDamageInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    killer: Option<Target>,
    #[serde(rename = "killerDamageInfo", skip_serializing_if = "Option::is_none")]
    killer_damage_info: Option<AddionalDamageInfo>,

    #[serde(rename = "character", skip_serializing_if = "Option::is_none")]
    player: Option<Target>,

    //  #[serde(skip_serializing_if = "Option::is_none")] ei voi olla tyhjä kun asetataan myöhemmin vasta tää arvo
    //#[serde(rename = "item")]
    //broken_armor: Option<BrokenArmor>,

    // #[serde(skip_serializing_if = "Option::is_none")]ei voi olla tyhjä kun asetataan myöhemmin vasta tää arvo
    distance: Option<f32>,

   // #[serde(skip_serializing_if = "Option::is_none")] ei voi olla tyhjä kun asetataan myöhemmin vasta tää arvo
    bullet_speed: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    item: Option<ItemEquipItem>,

    helmet: Option<Armor>,

    vest: Option<Armor>,

    victim_helmet: Option<Armor>,

    victim_vest: Option<Armor>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct Armor {
    condition: bool,
    item: String
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct ItemEquipItem {
    #[serde(rename = "itemId")]
    item_id: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct AddionalDamageInfo {
    #[serde(rename = "damageReason")]
    damage_reason: String,
    #[serde(rename = "damageTypeCategory")]
    damage_type_category: String,
    #[serde(rename = "damageCauserName")]
    weapon: String,
    #[serde(rename = "additionalInfo")]
    attachments: Vec<String>,
    distance: f32
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct BrokenArmor {
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "subCategory")]
    sub_category: String
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct AttackWeapon {
    #[serde(rename = "itemId")]
    weapon: String,
    #[serde(rename = "attachedItems")]
    attachments: Vec<String>
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct Target {
    name: String,
    #[serde(rename = "teamId")]
    team_id: i32,
    health: f32,
    ranking: i32,
    #[serde(rename = "individualRanking")]
    individual_ranking: i32,
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "isInVehicle")]
    is_in_vehicle: bool,
    location: Location
}

#[derive(Serialize, Debug, Clone)]
enum ActionType {
    LogPlayerTakeDamage,
    LogPlayerAttack,
    LogPlayerMakeGroggy, //tosin sanoen knokkaantuu
    LogArmorDestroy,
    LogPlayerKillV2,
    LogMatchDefinition,
    LogPlayerCreate,
    LogItemEquip,
    LogItemPickupFromCarepackage,
    LogItemPickupFromLootbox,
    Unknown
}

impl<'de> Deserialize<'de> for ActionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "LogPlayerTakeDamage" => Ok(ActionType::LogPlayerTakeDamage),
            "LogPlayerAttack" => Ok(ActionType::LogPlayerAttack),
            "LogPlayerMakeGroggy" => Ok(ActionType::LogPlayerMakeGroggy),
            "LogArmorDestroy" => Ok(ActionType::LogArmorDestroy),
            "LogPlayerKillV2" => Ok(ActionType::LogPlayerKillV2),
            "LogMatchDefinition" => Ok(ActionType::LogMatchDefinition),
            "LogPlayerCreate" => Ok(ActionType::LogPlayerCreate),
            "LogItemEquip" => Ok(ActionType::LogItemEquip),
            "LogItemPickupFromCarepackage" => Ok(ActionType::LogItemPickupFromCarepackage),
            "LogItemPickupFromLootbox" => Ok(ActionType::LogItemPickupFromLootbox),
            _ => Ok(ActionType::Unknown)
         }
     }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct Player {
    name: String,
    #[serde(rename = "teamId")]
    team_id: i32,
    health: f64,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
struct Location {
    x: f32,
    y: f32,
    z: f32
}

impl Location {
    fn distance(&self, other: &Location) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt() * 0.01
    }
}

fn format_data_names(data: &mut AddionalDamageInfo) {
    data.weapon = constants::weapons::WEAPONS
        .iter()
        .find(|&&(key, _)| key == data.weapon)
        .map(|&(_, name)| name.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    data.damage_type_category = constants::damage_types::DAMAGE_TYPES
        .iter()
        .find(|&&(key, _)| key == data.damage_type_category)
        .map(|&(_, name)| name.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    for attachment in &mut data.attachments {
        if let Some(&(_, ref new_attachment_name)) = constants::attachments::ATTACHMENTS
            .iter()
            .find(|&&(key, _)| key == *attachment)
        {
             *attachment = new_attachment_name.to_string();
        }
    }
}

fn make_match_summary_with_full_squad(all_squads: &mut HashMap<i32, Vec<String>>, parsed_include_data_for_passing: MatchOverviewInclude, parsed_match_data: MatchOverview, id: &String, parsed_for_all_events: Vec<Event>)
-> Result<bool, Box<dyn std::error::Error>> {

   let mut full_squad:Vec<String> = Vec::new();

   for squad in all_squads.values() {
       for player_name in constants::player_names::PLAYER_NAMES.iter() {
           if squad.contains(&player_name.to_string()) {
               full_squad.extend(squad.clone());
           }
       }
   }

    let mut single_player_performance: Vec<ParticipantAttributes> = Vec::new();

    for item in parsed_include_data_for_passing.included {
            match item {
                MatchOverviewIncluded::Participant {attributes, ..} => {
                    single_player_performance.push(attributes)
                }
                _ => {}
            }
        }

    let date = parsed_match_data.data.attributes.created_at;
    let game_mode = parsed_match_data.data.attributes.game_mode;
    let map_name = parsed_match_data.data.attributes.map_name;
    let actual_map_name = constants::map_names::MAP_NAME
        .iter()
        .find(|&&(key, _)| key == map_name)
        .map(|&(_, name)| name.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut squad: Vec<&mut ParticipantStats> = single_player_performance
        .iter_mut()
        .map(|player| &mut player.stats)
        .filter(|player| full_squad.contains(&player.name))
        .collect();

    for player_stats in &mut squad {
        for event in &parsed_for_all_events {
            match event.action {
                Some(ActionType::LogPlayerKillV2) => {
                    if let Some(kill_event) = &event.finisher {
                        if kill_event.name == player_stats.name {
                            player_stats.kills = player_stats.kills + 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    for player_stats in &mut squad {
        player_stats.kills = 0;
    }

    for event in &parsed_for_all_events {
        if let Some(ActionType::LogPlayerKillV2) = event.action {
            if let (Some(kill_event), Some(id)) = (&event.finisher, &event.attack_id)  {
                if *id != -1 { continue };
                for player_stats in &mut squad {
                    if kill_event.name == player_stats.name {
                        player_stats.kills += 1;
                    }
                }
            }
        }
    }

    let mut squad_match_data: HashMap<String, serde_json::Value> = HashMap::new();
        squad_match_data.insert("id".to_string(), json!(id));
        squad_match_data.insert("date".to_string(), json!(date));
        squad_match_data.insert("game_mode".to_string(), json!(game_mode));
        squad_match_data.insert("map_name".to_string(), json!(actual_map_name));
        squad_match_data.insert("squad".to_string(), json!(squad));

   let save_result = save_to_match(squad_match_data ,id)?;
   if !save_result {
       return Ok(false)
   }
   Ok(true)
}

fn save_to_match(new_match: HashMap<String, Value>, id: &String) -> Result<bool, Box<dyn std::error::Error>> {
    let mut existing_data: Vec<HashMap<String, Value>> = Vec::new();

    if let Ok(mut file) = File::open("data/matches.json") {
          let mut contents = String::new();
          file.read_to_string(&mut contents)?;
          existing_data = from_str(&contents)?;
      }

      let ids: Vec<String> = existing_data.iter()
            .filter_map(|single_match| {
                single_match.get("id").and_then(|id| id.as_str().map(|s| s.to_string()))
            })
            .collect();

    if !ids.is_empty() {
         if ids.contains(&id) {
             return Ok(false) // nein new matches
        }
    }

    existing_data.push(new_match);

    while existing_data.len() > 30 {
        let removed_match = existing_data.remove(0);
        if let Some(first_match) = existing_data.get(0) {
            if let Some(match_id) = first_match.get("id") {
                if let Some(id) = match_id.as_str() {
                    delete_single_match_from_redis(id);
                }
            }
        }
    }

    let json_data = serde_json::to_string_pretty(&existing_data)?;
    let folder_path = "data";
    let file_path = format!("{}/matches.json", folder_path);
    std::fs::create_dir_all(folder_path)?;

    let mut file = File::create(file_path)?;
    file.write_all(json_data.as_bytes())?;
    println!("success");
    let _ = load_matches_summary_data_into_redis()?;
    Ok(true)
}

async fn fetch_telemetry_data (url: String, id: &String, parsed_include_data_for_passing: MatchOverviewInclude, parsed_match_data: MatchOverview) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/vnd.api+json".parse()?);
    headers.insert(header::ACCEPT_ENCODING, "gzip".parse()?);
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let parsed_for_all_events: Vec<Event> = serde_json::from_str(&response).expect("Failed to parse JSON");
    let mut all_attack_and_take_damage_events: Vec<&mut Event> = Vec::new();
    let mut all_armor_equip_events: Vec<&mut Event> = Vec::new();
    let mut match_start_time = String::new();
    let mut all_squads: HashMap<i32, Vec<String>> = HashMap::new();

    for event in &parsed_for_all_events {
        match event.action {
            Some(ActionType::LogPlayerCreate) => {
                if let Some(player) = &event.player {
                    if let (Some(name), Some(team_id)) = (Some(&player.name), Some(&player.team_id)) {
                        all_squads.entry(*team_id)
                            .or_insert_with(Vec::new)
                            .push(name.to_string());
                    }
                }
            }
            Some(ActionType::LogMatchDefinition) => {
                if let Some(time) = &event.event_time {
                    match_start_time = time.to_string();
                }
            }
            _ => {}
        }
    }

    let new_match_found = make_match_summary_with_full_squad(&mut all_squads, parsed_include_data_for_passing, parsed_match_data, id, parsed_for_all_events.clone())?;

    if !new_match_found {
        println!("No new matches found");
        return Ok(());
    }
    println!("Continuing, {:?}", new_match_found);


    let mut filter_parsed_for_all_events_without_unknown_events: Vec<Event> = parsed_for_all_events
        .into_iter()
        .filter(|event| !matches!(event.action, Some(ActionType::Unknown)))
        .collect();

    for event in filter_parsed_for_all_events_without_unknown_events.iter_mut() {
        match event.action {
            Some(ActionType::LogPlayerAttack) | Some(ActionType::LogPlayerTakeDamage) => {
                all_attack_and_take_damage_events.push(event);
            }
            Some(ActionType::LogItemEquip) | Some(ActionType::LogArmorDestroy) | Some(ActionType::LogItemPickupFromCarepackage) | Some(ActionType::LogItemPickupFromLootbox) => {
                if let Some(ref mut armor) = event.item {
                    if let Some((_, new_armor_name)) = constants::armors::ARMORS
                        .iter()
                        .find(|&&(key, _)| key == armor.item_id)
                    {
                        armor.item_id = new_armor_name.to_string();
                        all_armor_equip_events.push(event);
                    }
                }
            }
            _ => {}
        }
    }

    let parsed: Vec<Event> = serde_json::from_str(&response).expect("Failed to parse JSON");

    let mut filtered_events: Vec<Event> = parsed
        .into_iter()
        .filter(|event| !matches!(event.action, Some(ActionType::Unknown)))
        .collect();

    let mut take_damage_events: Vec<&mut Event> = Vec::new();
    let mut attack_events: Vec<&mut Event> = Vec::new();

    for event in filtered_events.iter_mut() {
        match event.action {
            Some(ActionType::LogPlayerTakeDamage) | Some(ActionType::LogArmorDestroy) => {
                if constants::player_names::PLAYER_NAMES.iter().any(|&username| {
                    match (&event.attacker, &event.victim) {
                        (Some(attacker), _) if attacker.name == username => true,
                        (_, Some(victim)) if victim.name == username => true,
                        _ => false,
                    }
                }) {
                    match event.action {
                        Some(ActionType::LogPlayerTakeDamage) => take_damage_events.push(event),
                        Some(ActionType::LogArmorDestroy) => take_damage_events.push(event),
                        _ => {}
                    }
                }
            }
            Some(ActionType::LogPlayerKillV2) => {
                if constants::player_names::PLAYER_NAMES.iter().any(|&username| {
                    if let Some(attacker) = &event.attacker {
                        attacker.name == username
                    } else if let Some(victim) = &event.victim {
                        victim.name == username
                    } else if let Some(dbno_maker) = &event.dbno_maker {
                        dbno_maker.name == username
                    } else if let Some(finisher) = &event.finisher {
                        finisher.name == username
                    } else if let Some(killer) = &event.killer {
                        killer.name == username
                    }
                    else {
                        false
                    }
                }) {
                    match event.action {
                        Some(ActionType::LogPlayerKillV2) => take_damage_events.push(event),
                        _ => {}
                    }
                }
            }
            Some(ActionType::LogPlayerAttack) => {
                attack_events.push(event);
            }
            _ => {}
        }
    }

    let mut attack_ids: HashSet<i32> = attack_events
            .iter()
            .filter_map(|event| event.attack_id)
            .collect();
    attack_ids.insert(-1);

    for event in take_damage_events.iter_mut() {
        event.helmet = Some(Armor {
            condition: true,
            item: "bare".to_string(),
        });

        event.vest = Some(Armor {
            condition: true,
            item: "bare".to_string(),
        });

        event.victim_helmet = Some(Armor {
            condition: true,
            item: "bare".to_string(),
        });

        event.victim_vest = Some(Armor {
            condition: true,
            item: "bare".to_string(),
        });

        event.bullet_speed = Some(0.0);

        if let Some(victim) = &event.victim {
            for action_damage_event in &all_attack_and_take_damage_events {
                if Some(action_damage_event.attack_id) == Some(event.attack_id) {
                    if event.attack_id == Some(-1) {continue}

                    for equip_event in &all_armor_equip_events {

                        if let (Some(attack_time_str), Some(equip_time_str)) = (&event.event_time, &equip_event.event_time) {
                            let attack_time = DateTime::parse_from_rfc3339(attack_time_str).expect("Invalid time");
                            let equip_time = DateTime::parse_from_rfc3339(equip_time_str).expect("Invalid time");

                            if attack_time < equip_time {
                                continue;
                            }

                             if let Some(action) = &equip_event.action {
                                 match action {
                                     ActionType::LogItemEquip |
                                     ActionType::LogItemPickupFromCarepackage |
                                     ActionType::LogItemPickupFromLootbox => {
                                         if let (Some(attacker), Some(equiper)) = (&event.attacker, &equip_event.player) {
                                             if attacker.name == equiper.name {
                                                 if let Some(item) = &equip_event.item {
                                                     if item.item_id.contains("Helmet") {
                                                         if let Some(helmet) = event.helmet.as_mut() {
                                                             helmet.item = item.item_id.clone();
                                                             helmet.condition = true;
                                                         }
                                                   }

                                                     if item.item_id.contains("Vest") {
                                                         if let Some(vest) = event.vest.as_mut() {
                                                             vest.item = item.item_id.clone();
                                                             vest.condition = true;
                                                         }
                                                    }
                                                 }
                                             }

                                             if victim.name == equiper.name {
                                                 if let Some(item) = &equip_event.item {
                                                     if item.item_id.contains("Helmet") {
                                                         if let Some(victim_helmet) = event.victim_helmet.as_mut() {
                                                             victim_helmet.item = item.item_id.clone();
                                                             victim_helmet.condition = true;
                                                        }
                                                   }

                                                    if item.item_id.contains("Vest") {
                                                         if let Some(victim_vest) = event.victim_vest.as_mut() {
                                                             victim_vest.item = item.item_id.clone();
                                                             victim_vest.condition = true;
                                                         }
                                                    }
                                                 }
                                             }
                                         }
                                     },
                                     _ => {}
                                 }
                             }
                        }
                    }

                    if let Some(finisher) = &event.finisher {
                        let start = Location { x: finisher.location.x, y: finisher.location.y, z: finisher.location.z };
                        let end = Location { x: victim.location.x, y: victim.location.y, z: victim.location.z };
                        let distance = start.distance(&end);
                        event.distance = Some(distance);
                    }

                    if let (Some(attack_time), Some(victim_time)) = (&event.event_time, &action_damage_event.event_time) {
                        let attack_time = DateTime::parse_from_rfc3339(attack_time)
                            .expect("Invalid attack time");
                        let victim_time = DateTime::parse_from_rfc3339(victim_time)
                            .expect("Invalid victim time");
                        let duration = victim_time.signed_duration_since(attack_time);
                        let travel_time = (duration.num_milliseconds() as f32 / 1000.0).abs();
                        if travel_time == 0.0 {continue}
                        if let Some(attacker) = &event.attacker {
                            let start = Location { x: attacker.location.x, y: attacker.location.y, z: attacker.location.z };
                            let end = Location { x: victim.location.x, y: victim.location.y, z: victim.location.z };
                            let distance = start.distance(&end);
                            if distance == 0.0 {continue}
                            event.distance = Some(distance);
                            let bullet_speed = distance / travel_time;
                            let current_weapon = &event.damage_causer_name;
                            let formated_current_weapon = constants::weapons::WEAPONS
                                      .iter()
                                      .find(|&&(key, _)| key == current_weapon.clone().expect("Weapon not found").to_string())
                                      .map(|&(_, weapon_name)| weapon_name.to_string())
                                      .unwrap_or_else(|| "Unknown".to_string());

                            let weapon_type = constants::weapon_type::WEAPON_TYPE
                                .iter()
                                .find(|&&(key, _)| key == formated_current_weapon.as_str())
                                .map(|&(_, weapon_type)| weapon_type.to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            let max_bullet_speed = match weapon_type.as_str() {
                                "SMG" | "Pistol" => 500.0,
                                "DMR" | "LMG" | "Assault Rifle" => 1000.0,
                                "HP Sniper" => 1500.0,
                                "Shotgun" => 700.0,
                                _ => f32::MAX,
                            };

                            if bullet_speed > max_bullet_speed {
                                let default_bullet_speed = constants::default_bullet_speed::DEFAULT_BULLET_SPEED
                                    .iter()
                                    .find(|&&(key, _)| key == formated_current_weapon)
                                    .and_then(|&(_, speed_str)| speed_str.parse::<f64>().ok())
                                    .map(|value| value as f32);

                                event.bullet_speed = default_bullet_speed;
                            } else {
                                event.bullet_speed = Some(bullet_speed);
                            }
                        }
                    }
                }
            }
        }
    }

    for event in take_damage_events.iter_mut() {
        if !event.damage_causer_name.is_none() { // tää voi olla että sun setataan kohta muutenkin
            let current_weapon = &event.damage_causer_name;
            let formatted_name = constants::weapons::WEAPONS
                .iter()
                .find(|&&(key, _)| key == &current_weapon.clone().expect("REASON").to_string())
                .map(|&(_, name)| name.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            event.damage_causer_name = Some(formatted_name);
        }

        //Killer nfo aseen nimen vaihto, damage type vaihe ja attachmentit
        if let Some(ref mut killer_damage_info) = event.killer_damage_info {
            format_data_names(killer_damage_info);
        }

        //Finnisher aseen nimen vaihto, damage type vaihe ja attachmentit
        if let Some(ref mut finish_damage_info) = event.finish_damage_info {
            format_data_names(finish_damage_info);
        }

        //DNBO aseen nimen vaihto, damage type vaihe ja attachmentit
        if let Some(ref mut dbno_damage_info) = event.dbno_damage_info {
            format_data_names(dbno_damage_info);
        }

        /*
        if let Some(ref mut armor) = event.broken_armor {
            if let Some((_, new_item_name)) = constants::armors::ARMORS
                .iter()
                .find(|&&(key, _)| key == armor.item_id)
            {
                armor.item_id = new_item_name.to_string();
            }
        }
        */

        if let Some(ref mut damage_type_category) = event.damage_type_category {
            if let Some(&(_, ref new_formated_type)) = constants::damage_types::DAMAGE_TYPES
                .iter()
                .find(|&&(key, _)| key == damage_type_category)
            {
                *damage_type_category = new_formated_type.to_string();
            }
        }

        if let Some(ref mut damage_reason) = event.damage_reason {
            if let Some(&(_, new_damage_reason_name)) = constants::hit_locations::HIT_LOCATIONS
                .iter()
                .find(|&&(key, _)| key == damage_reason)
            {
                *damage_reason = new_damage_reason_name.to_string();
            }
        }
    }

    for event in take_damage_events.iter_mut() {

        if let Some(event_time) = &event.event_time {
            let parsed_event_time = DateTime::parse_from_rfc3339(event_time).expect("Invalid event time");
            let start_time = DateTime::parse_from_rfc3339(&match_start_time).expect("Invalid match start time");
            let time = parsed_event_time.signed_duration_since(start_time);
            let time_formatted = time.num_milliseconds() as f32 / 1000.0;
            event.event_time = Some(time_formatted.to_string());
        }

        if let Some(attack_id) = event.attack_id {
            if attack_ids.contains(&attack_id) {
                if let Some(attack_event) = attack_events.iter_mut().find(|e| e.attack_id == Some(attack_id)) {
                    if event.weapon.is_none() {
                        if let Some(ref mut weapon) = attack_event.weapon {
                            for attachment in &mut weapon.attachments {
                                if let Some(&(_, ref new_name)) = constants::attachments::ATTACHMENTS
                                    .iter()
                                    .find(|&&(key, _)| key == *attachment)
                                {
                                    *attachment = new_name.to_string().clone();
                                } else {
                               //     println!("{:?}", attachment);
                                }
                            }
                        if let Some(&(_, ref new_weapon_name)) = constants::weapons::WEAPONS
                                  .iter()
                                  .find(|&&(key, _)| key == weapon.weapon)
                              {
                                  weapon.weapon = new_weapon_name.to_string();
                              } else {
                                  weapon.weapon = weapon.weapon.to_string();
                              }
                            event.weapon = Some(weapon.clone());
                        }
                    }

                    if event.fire_weapon_stack_count.is_none() {
                        event.fire_weapon_stack_count = attack_event.fire_weapon_stack_count.clone();
                    }
                }
            }
        }
    }

    let filename: &str = &id;
    let _ = save_to_json(&take_damage_events, filename).await;
    Ok(())
}

async fn save_to_json(data: &Vec<& mut Event>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_data = serde_json::to_string_pretty(&data)?;

    let folder_path = "data/matches";
    let file_path = format!("{}/{}.json", folder_path, filename);

    let _ = std::fs::create_dir_all(folder_path)?;

    let mut file = File::create(file_path)?;
    file.write_all(json_data.as_bytes())?;
    let _ = load_single_match_to_redis(filename)?;
    Ok(())
}

async fn fetch_match_data (api_key: String, match_id: &String) -> Result<(), Box<dyn std::error::Error>>  {
    let url = "https://api.pubg.com/shards/steam/matches/".to_owned() + &match_id;
    let client = Client::new();
    let headers = make_headers(&api_key)?;

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let parsed_match_data: MatchOverview = serde_json::from_str(&response).expect("Failed to parse JSON");
    let parsed_include_data: MatchOverviewInclude = serde_json::from_str(&response).expect("Failed to parse JSON");
    let parsed_include_data_for_passing: MatchOverviewInclude = serde_json::from_str(&response).expect("Failed to parse JSON");
    let mut telemetry_url = String::new();
    let mut single_player_performance: Vec<ParticipantAttributes> = Vec::new();
    let mut rosters: Vec<RosterAttributes> = Vec::new(); //rank: 13, team_id: 16 en tiiä mihin näitä tarvis.. telemtry datasta saa vasta squadin.............. teamid kaikille

    for item in parsed_include_data.included {
        match item {
            MatchOverviewIncluded::Asset { attributes, .. } => {
                   telemetry_url = attributes.url;
            }
            MatchOverviewIncluded::Participant {attributes, ..} => {
                single_player_performance.push(attributes)

            }
            MatchOverviewIncluded::Roster {attributes, ..} => {
                rosters.push(attributes);
            }
            //_ => {}
        }
    }
    let id = &parsed_match_data.data.id.clone();

    let _ = fetch_telemetry_data(telemetry_url, id, parsed_include_data_for_passing, parsed_match_data).await?;

    Ok(())
}

async fn fetch_player_match_ids(api_key: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

    let match_ids = extract_user_id(&response).await;

    Ok(match_ids)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = env::var("api_key").expect("Api key not found...");
    let mut init_done = false;

      loop {
          match fetch_player_match_ids(api_key.clone()).await {
              Ok(match_ids) => {
                  if init_done {
                      println!("Checking for new matches!");
                      let _ = fetch_match_data(api_key.clone(), &match_ids[0]).await;
                  } else {
                      println!("Initial run");
                      for match_id in &match_ids {
                          let _ = fetch_match_data(api_key.clone(), match_id).await;
                      }
                      init_done = true;
                  }
              },
              Err(e) => {
                  eprintln!("Failed to fetch match IDs: {}", e);
              }
          }
          sleep(Duration::from_secs(30)).await;
      }
}
