//! Theater data the simulation needs, without the renderer.

use anyhow::{Context, Result};
use stealth_mission::GameData;
use stealth_sim::{SimData, TheaterTiles};

/// Tile data of a remake theater (`assets/theaters/<code>.ron`).
pub fn load_tiles(code: &str) -> Result<TheaterTiles> {
    let rel = format!("theaters/{}.ron", code.to_ascii_lowercase());
    let mut tiles = TheaterTiles::read_ron(&crate::assets::read(&rel)?).context(rel)?;
    tiles.relief = relief(code);
    Ok(tiles)
}

/// A theater's relief (`assets/theaters/<code>.relief`; flat without one).
pub fn relief(code: &str) -> stealth_sim::relief::Relief {
    TheaterTiles::read_relief(|name| crate::assets::read(name.trim_start_matches("../")), code)
}

/// The simulation's tables, with the tiles of theater `index` (`code`) if they are not among
/// the sim data's.
pub fn sim_data(index: usize, code: &str) -> Result<SimData> {
    let d = sim_tables()?;
    if d.theaters.get(index).is_some() {
        return Ok(d);
    }
    let tiles = load_tiles(code)?;
    Ok(d.with_theater(index, || tiles))
}

/// The mission generator's tables.
pub fn game_data() -> Result<GameData> {
    GameData::load_with(|name| match name.strip_prefix("../") {
        Some(rel) => crate::assets::read(rel),
        None => crate::assets::read(&format!("mission/{name}")),
    })
}

/// The simulation's tables and the classic theaters' tiles (`assets/sim`).
pub fn sim_tables() -> Result<SimData> {
    SimData::load_with(|name| match name.strip_prefix("../") {
        Some(rel) => crate::assets::read(rel),
        None => crate::assets::read(&format!("sim/{name}")),
    })
}

/// Index of a theater code ("central_europe").
pub fn theater_index(code: &str) -> Option<usize> {
    stealth_formats::ALL_THEATERS.iter().position(|t| t.eq_ignore_ascii_case(code))
}

/// The theater codes, lower case.
pub fn codes() -> Vec<String> {
    stealth_formats::ALL_THEATERS.iter().map(|t| t.to_ascii_lowercase()).collect()
}

/// The classic theaters first, then the remake's own (file prefix, display name).
pub const THEATERS: [(&str, &str); 10] = [
    ("libya", "Libya (1986)"),
    ("persian_gulf", "Persian Gulf (1984)"),
    ("north_cape", "North Cape"),
    ("central_europe", "Central Europe"),
    ("vietnam", "Vietnam"),
    ("middle_east", "Middle East"),
    ("desert_storm", "Desert Storm"),
    ("cuba", "Cuba"),
    ("korea", "Korea"),
    ("ukraine", "Ukraine (2026)"),
];

/// A theater's display name ("central_europe": "Central Europe"; the code itself if unknown).
pub fn theater_name(code: &str) -> String {
    THEATERS
        .iter()
        .find(|t| t.0.eq_ignore_ascii_case(code))
        .map_or_else(|| code.to_ascii_uppercase(), |t| t.1.to_string())
}
