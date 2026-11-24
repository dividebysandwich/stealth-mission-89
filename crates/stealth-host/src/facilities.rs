//! The campaign's strategic facilities in the simulation (`stealth_campaign::facilities`): each
//! stands as a building unit there, its destruction reported back to the campaign.

use stealth_campaign::{Avoid, CampaignEvent, Facility, FacilityKind, Side, WorldView};
use stealth_sim::Sim;
use stealth_sim::aaa::{GroundKind, M_PER_MAP};

use crate::Host;

/// The simulation's building of a kind of facility.
pub fn facility_kind(kind: FacilityKind) -> GroundKind {
    match kind {
        FacilityKind::DroneFactory => GroundKind::DroneFactory,
        FacilityKind::TankFactory => GroundKind::TankFactory,
        FacilityKind::Headquarters => GroundKind::Headquarters,
        FacilityKind::AmmoStorage => GroundKind::AmmoDepot,
        FacilityKind::PowerPlant => GroundKind::PowerPlant,
        FacilityKind::FuelDepot => GroundKind::FuelDepot,
    }
}

/// What the facilities keep clear of: the runways (metres off their sides and ends), and the
/// units on the ground.
fn avoid(sim: &Sim) -> Vec<Avoid> {
    const RUNWAY_CLEAR_M: f64 = 250.0;
    let map = |e: f64, n: f64| ((e / M_PER_MAP) as i32, 0x8000 - (n / M_PER_MAP) as i32);
    let mut out = Vec::new();
    for r in sim.runways() {
        let steps = (r.half_len / 200.0).ceil() as i32;
        for k in -steps..=steps {
            let s = r.half_len * k as f64 / steps.max(1) as f64;
            let (x, y) = map(r.c.0 + r.axis.0 * s, r.c.1 + r.axis.1 * s);
            out.push((x, y, ((r.half_width + RUNWAY_CLEAR_M) / M_PER_MAP) as i32));
        }
    }
    out.extend(sim.units().iter().filter(|u| !u.gone).map(|u| (u.x, u.y, 4)));
    out
}

/// The way a facility's building faces (rad from north): its own, from its id.
fn heading(id: u32) -> f32 {
    (id.wrapping_mul(0x9E37_79B1) >> 16) as f32 / 65536.0 * std::f32::consts::TAU
}

impl Host {
    /// The facilities stand in the simulation (placed first, for a new campaign or an older
    /// save without them).
    pub(crate) fn place_facilities(&mut self, sim: &mut Sim) {
        let avoid = avoid(sim);
        self.state.ensure_facilities(&avoid);
        for f in self.state.facilities.clone() {
            self.spawn_facility(sim, &f);
        }
    }

    fn spawn_facility(&mut self, sim: &mut Sim, f: &Facility) {
        let blue = self.state.facility_side(f) == Side::Blue;
        let kind = facility_kind(f.kind);
        if let Some(k) = sim.place_structure(kind, f.x, f.y, blue, heading(f.id)) {
            sim.set_structure_destroyed(k, !f.alive());
            self.facility_units.retain(|u| u.0 != f.id);
            self.facility_units.push((f.id, k));
        }
    }

    /// Facility `id`'s unit in the simulation.
    pub fn facility_unit(&self, id: u32) -> Option<usize> {
        self.facility_units.iter().find(|u| u.0 == id).map(|u| u.1)
    }

    /// The facility a ground unit is.
    pub fn unit_facility(&self, k: usize) -> Option<u32> {
        self.facility_units.iter().find(|u| u.1 == k).map(|u| u.0)
    }

    /// The campaign is told which of the units are facilities.
    pub(crate) fn tag_facilities(&self, view: &mut WorldView) {
        for &(id, k) in &self.facility_units {
            if let Some(u) = view.units.get_mut(k) {
                u.facility = Some(id);
            }
        }
    }

    /// A campaign event about the facilities, applied to the simulation.
    pub(crate) fn facility_event(&mut self, sim: &mut Sim, e: &CampaignEvent) {
        match *e {
            CampaignEvent::FacilityRestored { id } => {
                if let Some(k) = self.facility_unit(id) {
                    sim.set_structure_destroyed(k, false);
                }
            }
            CampaignEvent::FacilityDestroyed { id } => {
                if let Some(k) = self.facility_unit(id) {
                    sim.set_structure_destroyed(k, true);
                }
            }
            CampaignEvent::FacilityMoved { id } => {
                let Some(f) = self.state.facility(id).cloned() else { return };
                match self.facility_unit(id) {
                    Some(k) => sim.move_structure(k, f.x, f.y),
                    None => self.spawn_facility(sim, &f),
                }
            }
            // The facilities of a place taken change sides with it.
            CampaignEvent::PlaceCaptured { place, side, .. } => {
                for f in self.state.facilities.iter().filter(|f| f.place == Some(place)) {
                    if f.owner.is_none()
                        && let Some(k) = self.facility_unit(f.id)
                    {
                        sim.set_unit_side(k, side == Side::Blue);
                    }
                }
            }
            _ => {}
        }
    }
}
