use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

pub fn get_player_coords(uuid: &str) -> Result<Vec<f64>, Box<dyn std::error::Error>> {

    let file = File::open(format!("/Users/brianbarry/Library/Application Support/minecraft/saves/New World/playerdata/{}.dat", uuid))?;
    let mut decoder = GzDecoder::new(file);
    let mut nbt_data = Vec::new();
    decoder.read_to_end(&mut nbt_data)?;

    // Deserialize NBT data into a HashMap
    let nbt: HashMap<String, fastnbt::Value> = from_bytes(&nbt_data)?;

    if let Some(fastnbt::Value::List(pos_list)) = nbt.get("Pos") {
        let coords: Vec<f64> = pos_list.iter().filter_map(|v| {
            if let fastnbt::Value::Double(d) = v {
                Some(*d) //dereference
            } else {
                None
            }
        }).collect();

        if coords.len() == 3 {
            Ok(coords)
        } else {
            Err("Pos tag exists but is malformed (expected 3 doubles).".into())
        }
    } else {
        Err("Pos tag not found in NBT data.".into())
    }

}
