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
pub const ALL_THEATERS: [&str; 10] = [
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
];
