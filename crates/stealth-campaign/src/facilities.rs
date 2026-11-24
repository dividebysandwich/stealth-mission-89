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
use crate::{CampaignEvent, CampaignState, ObjectId, PlaceId, PlayerId, Side, SiteKind, dist};

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

    /// One a side (the factories, the headquarters), built in its rear and moved back there
    /// when its place falls; the others belong to their place and change hands with it.
    pub fn unique(self) -> bool {
        matches!(
            self,
            FacilityKind::DroneFactory | FacilityKind::TankFactory | FacilityKind::Headquarters
        )
    }

    /// Message id of its name.
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
    /// The place it stands at (its owner holds it; a unique one's place is where it was
    /// built).
    pub place: Option<PlaceId>,
    /// An ammo storage's airfield.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub airfield: Option<ObjectId>,
    /// A unique facility's side (it never changes hands).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Side>,
    /// Campaign time it was destroyed, and when it is back in action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destroyed_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_at: Option<f64>,
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
    pub fn facility_side(&self, f: &Facility) -> Side {
        f.owner
            .or_else(|| f.place.and_then(|p| self.place(p)).map(|p| p.owner))
            .unwrap_or(Side::Neutral)
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
        // The factories and the headquarters: in the rear.
        if let Some(rear) = self.rear_place(side) {
            for kind in
                [FacilityKind::DroneFactory, FacilityKind::TankFactory, FacilityKind::Headquarters]
            {
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
            self.add_facility(FacilityKind::AmmoStorage, x, y, Some(place), Some(object), None);
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
            self.add_facility(FacilityKind::FuelDepot, x, y, Some(p), None, None);
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
            self.add_facility(FacilityKind::PowerPlant, x, y, Some(p), None, None);
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
        });
        id
    }

    /// Facility `id`'s unit was destroyed (reported once per pilot: the first counts).
    pub(crate) fn facility_destroyed(
        &mut self,
        id: u32,
        _by: Option<PlayerId>,
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
            text!("cl-facility-destroyed", what = what, place = place)
        } else {
            text!("cl-enemy-facility-destroyed", what = what, place = place)
        };
        self.note(msg.clone());
        ev.push(CampaignEvent::Message(msg));
        ev.push(CampaignEvent::FacilityDestroyed { id });
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

    /// A place changed hands: a side's unique facility there is overrun, and is built again in
    /// the side's rear (destroyed meanwhile).
    pub(crate) fn facilities_captured(&mut self, place: PlaceId, ev: &mut Vec<CampaignEvent>) {
        let overrun: Vec<usize> = (0..self.facilities.len())
            .filter(|&k| {
                let f = &self.facilities[k];
                f.place == Some(place)
                    && f.owner.is_some_and(|s| s != self.places[place as usize].owner)
            })
            .collect();
        let mut rng = Rng::new(self.setup.seed ^ 0xFAC1_1715 ^ self.next_id);
        for k in overrun {
            let side = self.facilities[k].owner.expect("unique");
            let Some(rear) = self.rear_place(side) else { continue };
            let kind = self.facilities[k].kind;
            let id = self.facilities[k].id;
            // Out of the way of its own old spot while choosing the new one.
            self.facilities[k].x = -0x10000;
            // (Far from the airfields' runways: those of the rear place are well inside its
            // clearance.)
            let (x, y) = self.place_spot(kind, side, rear, FACTORY_FROM, &[], &mut rng);
            let now = self.clock;
            let back = now + self.game_days(kind.days());
            let was_alive = self.facilities[k].alive();
            let f = &mut self.facilities[k];
            f.x = x;
            f.y = y;
            f.place = Some(rear);
            f.destroyed_at = Some(now);
            f.back_at = Some(back);
            let f = f.clone();
            if was_alive {
                self.facility_lost(&f, side);
            }
            ev.push(CampaignEvent::FacilityMoved { id });
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
