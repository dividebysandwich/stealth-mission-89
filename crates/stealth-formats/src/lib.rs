pub mod font;
pub mod terrain;
pub mod wld;

/// The classic theaters' ids, also their asset file names.
pub const THEATERS: [&str; 9] = [
    "libya",
    "persian_gulf",
    "north_cape",
    "central_europe",
    "vietnam",
    "middle_east",
    "desert_storm",
    "cuba",
    "korea",
];

/// All theater ids: the nine classic theaters, then the newer ones.
pub const ALL_THEATERS: [&str; 12] = [
    "libya",
    "persian_gulf",
    "north_cape",
    "central_europe",
    "vietnam",
    "middle_east",
    "desert_storm",
    "cuba",
    "korea",
    "ukraine",
    "gulf_of_finland",
    "suwalki",
];

/// The theaters whose enemy launches attack drones (Geran) from launch sites: the drone
/// defense missions and the campaign's drone waves.
pub const DRONE_THEATERS: [&str; 3] = ["ukraine", "gulf_of_finland", "suwalki"];

/// Whether theater `code` has attack drones (see [`DRONE_THEATERS`]).
pub fn drone_theater(code: &str) -> bool {
    DRONE_THEATERS.iter().any(|t| t.eq_ignore_ascii_case(code))
}
