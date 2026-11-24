//! The sides' strategic facilities: each side's drone factory, tank factory and headquarters,
//! an ammo storage at each airfield, fuel depots outside its cities and the power plants of its
//! regions. They are buildings on the ground (the host stands each up as a unit of the
//! simulation, so a targeting pod or a FLIR picks them out like a vehicle). One destroyed is out
//! of action for some in-game days ([`FacilityKind::days`]), then rebuilt where it stood:
//!
//! * the drone factory: no drones are made for the side meanwhile;
//! * the tank factory: no tanks and infantry fighting vehicles join its armour reserve;
//! * the headquarters: the side's war slows down (its operations come less often, its fighters
//!   scramble later, its materiel comes in at half the rate);
//! * an ammo storage: its airfield's munitions are gone, and none are kept there meanwhile;
//! * a power plant: the towns of its region go dark, and repairs and launches there are slower;
//! * a fuel depot: the vehicles of its area stand still.
//!
//! An air defence unit (not a drone command center) runs on its side's nearest power plant and
//! fuel depot: with both destroyed it is offline.
//!
//! They are placed once, by the host (which knows the terrain and the runways), with their own
//! random stream: an older save gets them when it is loaded.

use serde::{Deserialize, Serialize};
use stealth_i18n::text;

use crate::rng::Rng;
use crate::tuning::*;
use crate::{
    CampaignEvent, CampaignState, ObjectId, PlaceId, PlayerId, Side, SiteKind, WorldView, dist,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FacilityKind {
    DroneFactory,
    TankFactory,
    Headquarters,
    AmmoStorage,
    PowerPlant,
    FuelDepot,
}

impl FacilityKind {
    pub const ALL: [FacilityKind; 6] = [
        FacilityKind::DroneFactory,
        FacilityKind::TankFactory,
        FacilityKind::Headquarters,
        FacilityKind::AmmoStorage,
        FacilityKind::PowerPlant,
        FacilityKind::FuelDepot,
    ];

    /// In-game days it is out of action once destroyed.
    pub fn days(self) -> f64 {
        match self {
            FacilityKind::DroneFactory | FacilityKind::TankFactory => FACTORY_DAYS,
            FacilityKind::Headquarters => HEADQUARTERS_DAYS,
            FacilityKind::PowerPlant => POWER_PLANT_DAYS,
            FacilityKind::FuelDepot => FUEL_DEPOT_DAYS,
            FacilityKind::AmmoStorage => AMMO_STORAGE_DAYS,
        }
    }

    /// Its ground's radius (map units): what it keeps clear of.
    pub fn footprint(self) -> i32 {
        match self {
            FacilityKind::DroneFactory | FacilityKind::TankFactory => 8,
            FacilityKind::PowerPlant => 8,
            FacilityKind::Headquarters | FacilityKind::AmmoStorage => 5,
            FacilityKind::FuelDepot => 5,
        }
    }

    /// One a side to start with (the factories, the headquarters), built in its rear.
    pub fn unique(self) -> bool {
        matches!(
            self,
            FacilityKind::DroneFactory | FacilityKind::TankFactory | FacilityKind::Headquarters
        )
    }

    /// Message id of its name.
    /// What it is worth to strike (the most first on the boards).
    pub fn value(self) -> i32 {
        match self {
            FacilityKind::Headquarters => 6,
            FacilityKind::DroneFactory | FacilityKind::TankFactory => 5,
            FacilityKind::PowerPlant => 4,
            FacilityKind::AmmoStorage | FacilityKind::FuelDepot => 3,
        }
    }

    pub fn name_id(self) -> &'static str {
        match self {
            FacilityKind::DroneFactory => "fac-drone-factory",
            FacilityKind::TankFactory => "fac-tank-factory",
            FacilityKind::Headquarters => "fac-headquarters",
            FacilityKind::AmmoStorage => "fac-ammo-storage",
            FacilityKind::PowerPlant => "fac-power-plant",
            FacilityKind::FuelDepot => "fac-fuel-depot",
        }
    }
}

/// One strategic facility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Facility {
    pub id: u32,
    pub kind: FacilityKind,
    /// Map units (x east, y south).
    pub x: i32,
    pub y: i32,
    /// The place it was built at (named after it).
    pub place: Option<PlaceId>,
    /// An ammo storage's airfield.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub airfield: Option<ObjectId>,
    /// The side it was built by: the headquarters' for good (it never changes hands, it moves
    /// back from the front), the others' until the front passes them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Side>,
    /// Campaign time it was destroyed, and when it is back in action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destroyed_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_at: Option<f64>,
    /// When it was last rebuilt (the AI leaves it alone a while after, `FACILITY_GRACE`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restored_at: Option<f64>,
}

impl Facility {
    pub fn alive(&self) -> bool {
        self.destroyed_at.is_none()
    }
}

/// A circle (x, y, radius; map units) a facility keeps out of: a runway's, a unit's.
pub type Avoid = (i32, i32, i32);

impl CampaignState {
    /// Campaign seconds of `days` in-game days (the sky's days, its clock ratio held within
    /// `FACILITY_SKY_MIN` .. `FACILITY_SKY_MAX`).
    pub fn game_days(&self, days: f64) -> f64 {
        let ratio = (self.setup.sky_ratio as f64).clamp(FACILITY_SKY_MIN, FACILITY_SKY_MAX);
        days * 86_400.0 / ratio
    }

    pub fn facility(&self, id: u32) -> Option<&Facility> {
        self.facilities.iter().find(|f| f.id == id)
    }

    /// Who holds a facility now.
    /// The headquarters its builder's; any other the side whose territory it stands in (the
    /// owner of the place nearest to it).
    pub fn facility_side(&self, f: &Facility) -> Side {
        if f.kind == FacilityKind::Headquarters
            && let Some(side) = f.owner
        {
            return side;
        }
        self.territory(f.x, f.y)
            .or(f.owner)
            .or_else(|| f.place.and_then(|p| self.place(p)).map(|p| p.owner))
            .unwrap_or(Side::Neutral)
    }

    /// Whose territory (x, y) is: the owner of the place nearest to it.
    pub fn territory(&self, x: i32, y: i32) -> Option<Side> {
        self.places.iter().min_by_key(|p| dist(p.x - x, p.y - y)).map(|p| p.owner)
    }

    /// A side's factory of `kind` works: one it holds stands (a side that never had one, as
    /// one with only carriers at the start, makes what it makes without).
    pub fn factory_works(&self, side: Side, kind: FacilityKind) -> bool {
        self.facilities_of(side, kind).any(Facility::alive)
            || !self.facilities.iter().any(|f| f.kind == kind && f.owner == Some(side))
    }

    /// A side's facilities of a kind.
    pub fn facilities_of(&self, side: Side, kind: FacilityKind) -> impl Iterator<Item = &Facility> {
        self.facilities.iter().filter(move |f| f.kind == kind && self.facility_side(f) == side)
    }

    /// A side's unique facility of `kind` (factories, headquarters) is destroyed.
    pub fn facility_down(&self, side: Side, kind: FacilityKind) -> bool {
        self.facilities_of(side, kind).any(|f| !f.alive())
    }

    /// A side's facility of `kind` nearest to (x, y).
    pub fn nearest_facility(
        &self,
        side: Side,
        kind: FacilityKind,
        x: i32,
        y: i32,
    ) -> Option<&Facility> {
        self.facilities_of(side, kind).min_by_key(|f| (dist(f.x - x, f.y - y), f.id))
    }

    /// The facilities placed, if they are not yet: clear of the host's circles (runways,
    /// units).
    pub fn ensure_facilities(&mut self, avoid: &[Avoid]) {
        if !self.facilities.is_empty() {
            return;
        }
        let mut rng = Rng::new(self.setup.seed ^ 0xFAC1_1715);
        for side in [Side::Blue, Side::Red] {
            self.place_side(side, avoid, &mut rng);
        }
    }

    fn place_side(&mut self, side: Side, avoid: &[Avoid], rng: &mut Rng) {
        let places: Vec<PlaceId> = (0..self.places.len() as PlaceId)
            .filter(|&p| self.places[p as usize].owner == side)
            .collect();
        if places.is_empty() {
            return;
        }
        // The factories and the headquarters: in the rear (the friendly side has no
        // headquarters in the theater when it starts with no airfield, or far outnumbered).
        let hq = side == Side::Red || !self.no_friendly_hq();
        let unique =
            [FacilityKind::DroneFactory, FacilityKind::TankFactory, FacilityKind::Headquarters];
        if let Some(rear) = self.rear_place(side) {
            for kind in unique.into_iter().filter(|&k| hq || k != FacilityKind::Headquarters) {
                let (x, y) = self.place_spot(kind, side, rear, FACTORY_FROM, avoid, rng);
                self.add_facility(kind, x, y, Some(rear), None, Some(side));
            }
        }
        // An ammo storage at each airfield.
        let fields: Vec<(ObjectId, Option<PlaceId>)> = self
            .sites
            .iter()
            .filter(|s| s.kind == SiteKind::Airfield && !s.removed)
            .filter(|s| self.owner(s.object) == Some(side))
            .map(|s| (s.object, s.place))
            .collect();
        for (object, place) in fields {
            let Some(place) = place else { continue };
            let s = self.site(object).expect("airfield");
            let (ax, ay) = (s.x, s.y);
            let (x, y) = self.spot_near(
                FacilityKind::AmmoStorage,
                side,
                (ax, ay),
                AMMO_STORAGE_FROM,
                Some(place),
                avoid,
                rng,
            );
            self.add_facility(
                FacilityKind::AmmoStorage,
                x,
                y,
                Some(place),
                Some(object),
                Some(side),
            );
        }
        // Fuel depots outside the cities.
        let cities: Vec<PlaceId> =
            places.iter().copied().filter(|&p| self.city_of(p).is_some()).collect();
        for p in spread(&cities, FUEL_DEPOTS_MAX, |p| self.place_xy(p)) {
            let at = self.city_of(p).expect("city");
            let (x, y) = self.spot_near(
                FacilityKind::FuelDepot,
                side,
                at,
                FUEL_DEPOT_FROM,
                Some(p),
                avoid,
                rng,
            );
            self.add_facility(FacilityKind::FuelDepot, x, y, Some(p), None, Some(side));
        }
        // The power plants, spread over the side's places.
        let n = places
            .len()
            .div_ceil(PLACES_PER_POWER_PLANT)
            .clamp(POWER_PLANTS.0, POWER_PLANTS.1)
            .min(places.len());
        for p in spread(&places, n, |p| self.place_xy(p)) {
            let (x, y) =
                self.place_spot(FacilityKind::PowerPlant, side, p, POWER_PLANT_FROM, avoid, rng);
            self.add_facility(FacilityKind::PowerPlant, x, y, Some(p), None, Some(side));
        }
    }

    fn place_xy(&self, p: PlaceId) -> (i32, i32) {
        let p = &self.places[p as usize];
        (p.x, p.y)
    }

    /// A place's city (where it is), if it has one.
    fn city_of(&self, p: PlaceId) -> Option<(i32, i32)> {
        self.places[p as usize]
            .members
            .iter()
            .filter_map(|&o| self.site(o))
            .find(|s| s.kind == SiteKind::City)
            .map(|s| (s.x, s.y))
    }

    /// A side's place furthest from the front: its production base's, else the one furthest
    /// from the other side's places.
    pub(crate) fn rear_place(&self, side: Side) -> Option<PlaceId> {
        if let Some(p) = self.rearmost(side).and_then(|b| self.site(b)).and_then(|s| s.place)
            && self.places[p as usize].owner == side
        {
            return Some(p);
        }
        let enemy: Vec<(i32, i32)> =
            self.places.iter().filter(|p| p.owner == side.enemy()).map(|p| (p.x, p.y)).collect();
        (0..self.places.len() as PlaceId)
            .filter(|&p| self.places[p as usize].owner == side)
            .max_by_key(|&p| {
                let (x, y) = self.place_xy(p);
                let d = enemy.iter().map(|&(ex, ey)| dist(x - ex, y - ey)).min();
                (d.unwrap_or(0), std::cmp::Reverse(p))
            })
    }

    fn place_spot(
        &self,
        kind: FacilityKind,
        side: Side,
        place: PlaceId,
        from: (i32, i32),
        avoid: &[Avoid],
        rng: &mut Rng,
    ) -> (i32, i32) {
        let at = self.place_xy(place);
        self.spot_near(kind, side, at, from, None, avoid, rng)
    }

    /// A spot for a facility `from.0` .. `from.1` from `at`: on dry, level land the side holds,
    /// clear of the towns, the airfields and the sites, of the host's circles and of the other
    /// facilities (its own place's town not counted for `own`). The rules relax one by one
    /// where nothing fits; at worst it stands at the inner radius.
    fn spot_near(
        &self,
        kind: FacilityKind,
        side: Side,
        at: (i32, i32),
        from: (i32, i32),
        own: Option<PlaceId>,
        avoid: &[Avoid],
        rng: &mut Rng,
    ) -> (i32, i32) {
        const TRIES: usize = 48;
        let mut first = None;
        for strict in (0..4).rev() {
            for _ in 0..TRIES {
                let t = rng.below(0x8000) as f64 / 32768.0 * std::f64::consts::TAU;
                let r = from.0 + rng.below(from.1 - from.0 + 1);
                let x = at.0 + (t.sin() * r as f64) as i32;
                let y = at.1 - (t.cos() * r as f64) as i32;
                first.get_or_insert((x, y));
                if self.spot_ok(kind, side, x, y, own, avoid, strict) {
                    return (x, y);
                }
            }
        }
        first.unwrap_or(at)
    }

    /// `strict` 3: every rule; 2: steep ground allowed; 1: half the clearances; 0: dry land on
    /// the map is enough.
    fn spot_ok(
        &self,
        kind: FacilityKind,
        side: Side,
        x: i32,
        y: i32,
        own: Option<PlaceId>,
        avoid: &[Avoid],
        strict: u8,
    ) -> bool {
        if !(0x200..0x7E00).contains(&x) || !(0x200..0x7E00).contains(&y) {
            return false;
        }
        let foot = kind.footprint();
        let wet = |x: i32, y: i32| self.terrain.is_water(x, y) == Some(true);
        let ring = (0..8).map(|k| {
            let t = k as f64 * std::f64::consts::FRAC_PI_4;
            (x + (t.cos() * foot as f64) as i32, y + (t.sin() * foot as f64) as i32)
        });
        if wet(x, y) || ring.into_iter().any(|(px, py)| wet(px, py)) {
            return false;
        }
        if strict == 0 {
            return true;
        }
        if strict >= 3 && self.terrain.is_steep(x, y, foot) {
            return false;
        }
        let scale = if strict >= 2 { 1 } else { 2 };
        let near = |px: i32, py: i32, d: i32| dist(px - x, py - y) < d / scale;
        let nearest = self.places.iter().min_by_key(|p| dist(p.x - x, p.y - y));
        if nearest.is_none_or(|p| p.owner != side) {
            return false;
        }
        let own_xy = own.map(|p| self.place_xy(p));
        if self
            .places
            .iter()
            .filter(|p| Some((p.x, p.y)) != own_xy)
            .any(|p| near(p.x, p.y, DEPLOY_PLACE_CLEAR + foot))
        {
            return false;
        }
        if self.sites.iter().any(|s| !s.removed && near(s.x, s.y, DEPLOY_SITE_CLEAR + foot)) {
            return false;
        }
        if avoid.iter().any(|&(ax, ay, r)| near(ax, ay, r + foot)) {
            return false;
        }
        if self.facilities.iter().any(|f| near(f.x, f.y, f.kind.footprint() + foot + 0x10)) {
            return false;
        }
        !self.deployments.iter().any(|d| near(d.x, d.y, crate::footprint(d.kind) + foot))
    }

    fn add_facility(
        &mut self,
        kind: FacilityKind,
        x: i32,
        y: i32,
        place: Option<PlaceId>,
        airfield: Option<ObjectId>,
        owner: Option<Side>,
    ) -> u32 {
        let id = self.id();
        self.facilities.push(Facility {
            id,
            kind,
            x,
            y,
            place,
            airfield,
            owner,
            destroyed_at: None,
            back_at: None,
            restored_at: None,
        });
        id
    }

    /// The friendly side goes without a headquarters: it starts with no airfield of its own
    /// (a carrier's war), or outnumbered `HQ_OMIT_RATIO` times over.
    pub(crate) fn no_friendly_hq(&self) -> bool {
        let airfield = self
            .sites
            .iter()
            .any(|s| s.kind == SiteKind::Airfield && self.owner(s.object) == Some(Side::Blue));
        let [blue, red] = self.start.strength;
        !airfield || blue <= 0.0 || red / blue >= HQ_OMIT_RATIO
    }

    /// Facility `id`'s unit was destroyed (reported once per pilot: the first counts).
    pub(crate) fn facility_destroyed(
        &mut self,
        id: u32,
        by: Option<PlayerId>,
        ev: &mut Vec<CampaignEvent>,
    ) {
        let Some(k) = self.facilities.iter().position(|f| f.id == id && f.alive()) else {
            return;
        };
        let now = self.clock;
        let back = now + self.game_days(self.facilities[k].kind.days());
        let f = &mut self.facilities[k];
        f.destroyed_at = Some(now);
        f.back_at = Some(back);
        let f = f.clone();
        let side = self.facility_side(&f);
        self.facility_lost(&f, side);
        let what = text!(f.kind.name_id());
        let place = f.place.and_then(|p| self.place(p)).map(|p| p.name.clone()).unwrap_or_default();
        let msg = if side == Side::Blue {
            text!("cl-facility-destroyed", what = what, place = place.as_str())
        } else {
            text!("cl-enemy-facility-destroyed", what = what, place = place.as_str())
        };
        self.note(msg.clone());
        ev.push(CampaignEvent::Message(msg));
        ev.push(CampaignEvent::FacilityDestroyed { id });
        // What the player's strike did, for his debrief.
        if let Some(p) = by.filter(|_| side == Side::Red) {
            let hours = (back - now) / 3600.0;
            let hours = format!("{:.1}", hours).trim_end_matches(".0").to_string();
            let id = match f.kind {
                FacilityKind::DroneFactory => "cl-fx-drone-factory",
                FacilityKind::TankFactory => "cl-fx-tank-factory",
                FacilityKind::Headquarters => "cl-fx-headquarters",
                FacilityKind::AmmoStorage => "cl-fx-ammo-storage",
                FacilityKind::PowerPlant => "cl-fx-power-plant",
                FacilityKind::FuelDepot => "cl-fx-fuel-depot",
            };
            self.effect(p, text!(id, place = place.as_str(), hours = hours));
        }
        self.board_at = f64::MIN;
    }

    /// What losing facility `f` of `side` costs at once: an ammo storage's munitions go up
    /// with it.
    fn facility_lost(&mut self, f: &Facility, _side: Side) {
        if f.kind == FacilityKind::AmmoStorage
            && let Some(s) = f.airfield.and_then(|b| self.stocks.get_mut(&b))
        {
            for c in s.counts.values_mut() {
                *c = 0.0;
            }
        }
    }

    /// Airfield `base`'s ammo storage is destroyed: it keeps no munitions until it is rebuilt.
    pub fn store_down(&self, base: ObjectId) -> bool {
        self.facilities
            .iter()
            .any(|f| f.kind == FacilityKind::AmmoStorage && f.airfield == Some(base) && !f.alive())
    }

    /// The power plant of a side's region at (x, y): its nearest.
    fn plant_for(&self, side: Side, x: i32, y: i32) -> Option<&Facility> {
        self.nearest_facility(side, FacilityKind::PowerPlant, x, y)
    }

    /// The fuel depot of a side's area at (x, y): its nearest.
    fn depot_for(&self, side: Side, x: i32, y: i32) -> Option<&Facility> {
        self.nearest_facility(side, FacilityKind::FuelDepot, x, y)
    }

    /// A side's region at (x, y) has no power (its power plant is destroyed).
    pub fn dark(&self, side: Side, x: i32, y: i32) -> bool {
        self.plant_for(side, x, y).is_some_and(|f| !f.alive())
    }

    /// A side's vehicles at (x, y) have no fuel (their area's fuel depot is destroyed).
    pub fn dry(&self, side: Side, x: i32, y: i32) -> bool {
        self.depot_for(side, x, y).is_some_and(|f| !f.alive())
    }

    /// How much slower a side's repairs and launches at (x, y) are: `POWER_SLOW` without
    /// power.
    pub fn power_slow(&self, side: Side, x: i32, y: i32) -> f64 {
        if self.dark(side, x, y) { POWER_SLOW } else { 1.0 }
    }

    /// The same at a site (its owner's).
    pub(crate) fn power_slow_at(&self, object: ObjectId) -> f64 {
        match (self.site(object), self.owner(object)) {
            (Some(s), Some(side)) => self.power_slow(side, s.x, s.y),
            _ => 1.0,
        }
    }

    /// What an air defence unit of a side at (x, y) is cut off from: bit 1 its power, bit 2 its
    /// fuel (both: it is offline).
    pub fn ad_cut(&self, side: Side, x: i32, y: i32) -> u8 {
        (self.dark(side, x, y) as u8) | (self.dry(side, x, y) as u8) << 1
    }

    /// The areas without fuel: where a side's destroyed fuel depots stand and the places they
    /// supply (map units: side, centre, radius).
    pub fn dry_areas(&self) -> Vec<(Side, (i32, i32), i32)> {
        let mut out = Vec::new();
        for f in self.facilities.iter().filter(|f| f.kind == FacilityKind::FuelDepot && !f.alive())
        {
            let side = self.facility_side(f);
            out.push((side, (f.x, f.y), FUEL_AREA));
            for p in self.places.iter().filter(|p| p.owner == side) {
                if self.depot_for(side, p.x, p.y).is_some_and(|d| d.id == f.id) {
                    out.push((side, (p.x, p.y), FUEL_AREA));
                }
            }
        }
        out
    }

    /// The cities of the regions without power (map units): their lights are out.
    pub fn dark_cities(&self) -> Vec<(i32, i32)> {
        self.sites
            .iter()
            .filter(|s| s.kind == SiteKind::City && !s.removed)
            .filter(|s| self.owner(s.object).is_some_and(|side| self.dark(side, s.x, s.y)))
            .map(|s| (s.x, s.y))
            .collect()
    }

    /// The delay (s) a side's fighters at `base` take to scramble: longer without power.
    pub(crate) fn scramble_delay(&self, base: ObjectId) -> f64 {
        let power = if self.power_slow_at(base) > 1.0 { POWER_SCRAMBLE_DELAY } else { 0.0 };
        let hq = match self.owner(base) {
            Some(side) if self.hq_slow(side) > 1.0 => HQ_SCRAMBLE_DELAY,
            _ => 0.0,
        };
        power + hq
    }

    /// A threat seen from `base` now: its fighters may go (they have waited their delay since
    /// it was first seen).
    pub(crate) fn scramble_ready(&mut self, base: ObjectId) -> bool {
        let delay = self.scramble_delay(base);
        let now = self.clock;
        // Threats not seen lately are forgotten.
        self.scramble_alert.retain(|a| now - a.2 <= 30.0);
        if delay <= 0.0 {
            return true;
        }
        let k = match self.scramble_alert.iter().position(|a| a.0 == base) {
            Some(k) => k,
            None => {
                self.scramble_alert.push((base, now, now));
                self.scramble_alert.len() - 1
            }
        };
        self.scramble_alert[k].2 = now;
        now - self.scramble_alert[k].1 >= delay
    }

    /// Offers for `side`'s board on the other side's facilities: the most valuable standing
    /// ones in reach of its bases (not those just rebuilt), up to `FACILITY_OFFERS` and one
    /// more for every `FACILITY_OFFERS_PER` players beyond the first (title, objective, where,
    /// threat).
    pub(crate) fn facility_offers(
        &self,
        side: Side,
        view: &WorldView,
    ) -> Vec<(stealth_i18n::Text, crate::Objective, i32, i32, i32)> {
        let now = self.clock;
        let mut found: Vec<(&Facility, u32, i32)> = self
            .facilities
            .iter()
            .filter(|f| f.alive() && self.facility_side(f) == side.enemy())
            .filter(|f| f.restored_at.is_none_or(|t| now - t >= FACILITY_GRACE))
            .filter_map(|f| {
                let k = view.units.iter().position(|u| u.facility == Some(f.id) && u.alive)?;
                let base = self.nearest_base(side, f.x, f.y)?;
                let b = self.site(base)?;
                let d = dist(b.x - f.x, b.y - f.y);
                (d <= DEEP_RANGE).then_some((f, k as u32, d))
            })
            .collect();
        found.sort_by_key(|&(f, _, d)| (std::cmp::Reverse(f.kind.value()), d, f.id));
        let players = self.players.iter().filter(|p| p.sortie.is_some()).count().max(1);
        let n = FACILITY_OFFERS + (players - 1) / FACILITY_OFFERS_PER;
        found
            .into_iter()
            .take(n)
            .map(|(f, k, _)| {
                // The SAM sites about it.
                let sams = self
                    .sites
                    .iter()
                    .filter(|s| s.kind == SiteKind::Sam && s.alive())
                    .filter(|s| self.owner(s.object) == Some(side.enemy()))
                    .filter(|s| dist(s.x - f.x, s.y - f.y) <= SITE_ASSOCIATION)
                    .count() as i32;
                let place = f.place.and_then(|p| self.place(p)).map_or("", |p| p.name.as_str());
                let title = text!("cb-facility", what = text!(f.kind.name_id()), place = place);
                let objective = crate::Objective::Facility { facility: f.id, aaa: vec![k] };
                (title, objective, f.x, f.y, sams * SAM_THREAT)
            })
            .collect()
    }

    /// Every tick: the facilities whose time is up are back in action.
    pub(crate) fn run_facilities(&mut self, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        let due: Vec<usize> = (0..self.facilities.len())
            .filter(|&k| self.facilities[k].back_at.is_some_and(|t| t <= now))
            .collect();
        for k in due {
            let f = &mut self.facilities[k];
            f.destroyed_at = None;
            f.back_at = None;
            f.restored_at = Some(now);
            let f = f.clone();
            let side = self.facility_side(&f);
            ev.push(CampaignEvent::FacilityRestored { id: f.id });
            if side == Side::Blue {
                let place =
                    f.place.and_then(|p| self.place(p)).map(|p| p.name.clone()).unwrap_or_default();
                let msg =
                    text!("cl-facility-restored", what = text!(f.kind.name_id()), place = place);
                self.note(msg.clone());
                ev.push(CampaignEvent::Message(msg));
            }
        }
    }

    /// The facilities' sides before a place changes hands (for `facilities_captured`).
    pub(crate) fn facility_sides(&self) -> Vec<(u32, Side)> {
        self.facilities.iter().map(|f| (f.id, self.facility_side(f))).collect()
    }

    /// A place changed hands: the facilities the front passed change sides with their ground
    /// (the players hear of it); a headquarters overrun is built again in its side's rear
    /// (destroyed meanwhile).
    pub(crate) fn facilities_captured(
        &mut self,
        before: &[(u32, Side)],
        ev: &mut Vec<CampaignEvent>,
    ) {
        for &(id, was) in before {
            let Some(f) = self.facility(id).cloned() else { continue };
            let now = self.facility_side(&f);
            if now == was {
                continue;
            }
            let place =
                f.place.and_then(|p| self.place(p)).map_or(String::new(), |p| p.name.clone());
            let what = text!(f.kind.name_id());
            let msg = match (was, now) {
                (_, Side::Blue) => Some(text!("cl-facility-captured", what = what, place = place)),
                (Side::Blue, _) => Some(text!("cl-facility-lost", what = what, place = place)),
                _ => None,
            };
            if let Some(t) = msg {
                self.note(t.clone());
                ev.push(CampaignEvent::Message(t));
            }
            ev.push(CampaignEvent::FacilityChanged { id });
        }
        // A headquarters in the other side's territory now.
        let overrun: Vec<u32> = self
            .facilities
            .iter()
            .filter(|f| f.kind == FacilityKind::Headquarters)
            .filter(|f| f.owner.is_some_and(|s| self.territory(f.x, f.y) != Some(s)))
            .map(|f| f.id)
            .collect();
        for id in overrun {
            self.move_headquarters(id, true, ev);
        }
        // The front moved: those it came near move back.
        self.keep_headquarters_back(ev);
    }

    /// Headquarters `id` moves to its side's rear: `overrun` (destroyed meanwhile, for its
    /// days) or ahead of the front (standing at once). False when there is nowhere further.
    fn move_headquarters(&mut self, id: u32, overrun: bool, ev: &mut Vec<CampaignEvent>) -> bool {
        let Some(k) = self.facilities.iter().position(|f| f.id == id) else { return false };
        let Some(side) = self.facilities[k].owner else { return false };
        let Some(rear) = self.rear_place(side) else { return false };
        let kind = self.facilities[k].kind;
        let mut rng = Rng::new(self.setup.seed ^ 0xFAC1_1715 ^ self.next_id ^ id);
        let (ox, oy) = (self.facilities[k].x, self.facilities[k].y);
        // Out of the way of its own old spot while choosing the new one.
        self.facilities[k].x = -0x10000;
        // (Far from the airfields' runways: those of the rear place are well inside its
        // clearance.)
        let (x, y) = self.place_spot(kind, side, rear, FACTORY_FROM, &[], &mut rng);
        // Not worth it ahead of the front unless the new spot is well further back.
        if !overrun && self.front_distance(side, x, y) < self.front_distance(side, ox, oy) + 0x100 {
            self.facilities[k].x = ox;
            return false;
        }
        let now = self.clock;
        let back = now + self.game_days(kind.days());
        let was_alive = self.facilities[k].alive();
        let f = &mut self.facilities[k];
        f.x = x;
        f.y = y;
        f.place = Some(rear);
        if overrun {
            f.destroyed_at = Some(now);
            f.back_at = Some(back);
        }
        let f = f.clone();
        if overrun && was_alive {
            self.facility_lost(&f, side);
        }
        if side == Side::Blue {
            let place = self.places[rear as usize].name.clone();
            let t = text!("cl-hq-moved", place = place);
            self.note(t.clone());
            ev.push(CampaignEvent::Message(t));
        }
        ev.push(CampaignEvent::FacilityMoved { id });
        true
    }

    /// How far (x, y) is from the other side's nearest place (map units).
    fn front_distance(&self, side: Side, x: i32, y: i32) -> i32 {
        self.places
            .iter()
            .filter(|p| p.owner == side.enemy())
            .map(|p| dist(p.x - x, p.y - y))
            .min()
            .unwrap_or(i32::MAX)
    }

    /// A headquarters the front came near moves back (`HQ_FRONT_CLEAR`).
    pub(crate) fn keep_headquarters_back(&mut self, ev: &mut Vec<CampaignEvent>) {
        let near: Vec<u32> = self
            .facilities
            .iter()
            .filter(|f| f.kind == FacilityKind::Headquarters && f.alive())
            .filter(|f| f.owner.is_some_and(|s| self.front_distance(s, f.x, f.y) < HQ_FRONT_CLEAR))
            .map(|f| f.id)
            .collect();
        for id in near {
            self.move_headquarters(id, false, ev);
        }
    }
}

/// Up to `n` of `items`, spread out: the first, then each time the one furthest from those
/// picked (all of them when there are no more than `n`).
fn spread<T: Copy + PartialEq>(items: &[T], n: usize, at: impl Fn(T) -> (i32, i32)) -> Vec<T> {
    if items.len() <= n {
        return items.to_vec();
    }
    let mut out: Vec<T> = Vec::new();
    while out.len() < n {
        let next = items
            .iter()
            .copied()
            .filter(|i| !out.contains(i))
            .max_by_key(|&i| {
                let (x, y) = at(i);
                out.iter().map(|&o| at(o)).map(|(ox, oy)| dist(ox - x, oy - y)).min().unwrap_or(0)
            })
            .expect("more items than picked");
        out.push(next);
    }
    out
}
