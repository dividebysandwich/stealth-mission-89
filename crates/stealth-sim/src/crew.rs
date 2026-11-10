//! Campaign: aircraft with more than one seat. The pilot whose sortie it is (the *owner*)
//! flies it with his flight model, from the pilot's seat or, the AI flying for him, from the
//! gunner's seat. Another player can sit in a free gunner's seat ([`CrewCmd::Join`]): he is
//! out of the world like a player at an air defence console, his pod and trigger go to the
//! owner's aircraft ([`Sim::crew_input`]): the AC-130's side guns; the AH-64's targeting pod
//! (the TADS), chin gun and missiles, the pilot still firing them too.
//!
//! The seats are in the campaign's [`DefenceState`](crate::station::DefenceState) (`crews`),
//! so every client knows who sits where.

use serde::{Deserialize, Serialize};
use stealth_i18n::{Text, text};

use crate::Sim;
use crate::gunship::SideGuns;
use crate::types::*;

/// A seat in an aircraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Seat {
    #[default]
    Pilot,
    Gunner,
}

/// Who sits where in pilot `owner`'s aircraft (only kept while it is not just him flying).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Crew {
    pub owner: u8,
    /// The owner's seat (the gunner's: the AI flies).
    pub seat: Seat,
    /// Another player at the gunner's console.
    pub gunner: Option<u8>,
}

/// A gunner's seat: where the pod looks, the trigger held, the gun selected (the AC-130's:
/// 0 40 mm, 1 105 mm); the AH-64's: the station whose missiles are selected (None: the chin
/// gun, which the trigger fires).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GunnerAim {
    pub pod: Option<PodAim>,
    pub trigger: bool,
    pub gun: u8,
    #[serde(default)]
    pub station: Option<u8>,
}

/// What a player does about a seat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrewCmd {
    /// Sit at the free gunner's console of pilot `owner`'s aircraft.
    Join { owner: u8 },
    /// Leave the gunner's console (back to the operations room).
    Leave,
    /// The owner moves to another seat of his aircraft.
    Seat(Seat),
    /// The gunner's console now.
    Aim(GunnerAim),
    /// The gunner designates a ground point or a vehicle with the pod.
    Designate { at: Option<(i32, i32)>, unit: Option<u16> },
    /// The AH-64's gunner fires a missile from this station.
    Fire { station: u8 },
}

/// What a crew member's client is told of the aircraft he is in (with his own state): the
/// owner's side guns and designation for a gunner, the gunner's pod for the owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CrewView {
    pub owner: u8,
    pub seat: Seat,
    pub gunner: Option<u8>,
    pub side_guns: SideGuns,
    pub spi: Option<(i32, i32)>,
    pub spi_unit: Option<u16>,
    /// The pod's line of sight the side guns are laid by.
    pub pod: Option<PodAim>,
    /// The AH-64's: its gun rounds, its stores by station (weapon type, rounds; -1 none), its
    /// chin gun firing.
    #[serde(default)]
    pub gun_ammo: i16,
    #[serde(default)]
    pub stores: [(i8, i8); 6],
    #[serde(default)]
    pub gun_firing: bool,
}

/// The aircraft has a gunner's seat: the AC-130's console, the AH-64's front cockpit.
pub fn has_gunner_seat(spectre: bool, apache: bool) -> bool {
    spectre || apache
}

impl Sim {
    /// The seats taken (a campaign's).
    pub fn crews(&self) -> &[Crew] {
        self.defence().map_or(&[], |d| &d.crews[..])
    }

    /// The aircraft pilot `player` sits in and his seat (as owner or as gunner).
    pub fn crew_of(&self, player: u8) -> Option<(u8, Seat)> {
        self.crews().iter().find_map(|c| {
            if c.owner == player {
                Some((c.owner, c.seat))
            } else if c.gunner == Some(player) {
                Some((c.owner, Seat::Gunner))
            } else {
                None
            }
        })
    }

    /// The other player at pilot `owner`'s gunner's console.
    pub fn gunner_of(&self, owner: u8) -> Option<u8> {
        self.crews().iter().find(|c| c.owner == owner).and_then(|c| c.gunner)
    }

    /// The owner's seat in his own aircraft.
    pub fn owner_seat(&self, owner: u8) -> Seat {
        self.crews().iter().find(|c| c.owner == owner).map_or(Seat::Pilot, |c| c.seat)
    }

    /// Pilot `owner`'s aircraft has a gunner's console nobody sits at.
    pub fn gunner_seat_free(&self, owner: u8) -> bool {
        // A mirror knows the others' aircraft as everyone sees them.
        let spectre = if self.mirror && owner != self.pl.id {
            self.remote
                .iter()
                .any(|p| p.id == owner && has_gunner_seat(p.spectre, p.apache) && p.chute.is_none())
        } else {
            let p = self.pilot_ref(owner as usize);
            p.id == owner && !p.absent && p.eject == 0 && has_gunner_seat(p.spectre, p.apache)
        };
        spectre && self.owner_seat(owner) == Seat::Pilot && self.gunner_of(owner).is_none()
    }

    /// Pilot `id`'s aircraft in the world: whether it is in the air (None: not in the world);
    /// a mirror's from the others' aircraft as everyone sees them.
    pub fn pilot_flying(&self, id: u8) -> Option<bool> {
        if self.mirror && id != self.pl.id {
            return self.remote.iter().find(|p| p.id == id).map(|p| !p.on_ground);
        }
        (!self.pilot_absent(id)).then(|| self.pilot_in_the_air(id))
    }

    fn crews_mut(&mut self) -> Option<&mut Vec<Crew>> {
        self.campaign.as_mut().map(|c| &mut c.defence.crews)
    }

    /// Sets the owner's seat (the host, for a player who takes an AI aircraft as gunner).
    pub fn set_owner_seat(&mut self, owner: u8, seat: Seat) {
        let Some(crews) = self.crews_mut() else { return };
        match crews.iter_mut().find(|c| c.owner == owner) {
            Some(c) => c.seat = seat,
            None => crews.push(Crew { owner, seat, gunner: None }),
        }
        self.tidy_crews();
    }

    /// A seat command of player `player`.
    pub fn crew_command(&mut self, player: u8, cmd: CrewCmd) -> Result<(), Text> {
        if self.campaign.is_none() {
            return Err(text!("st-no-campaign"));
        }
        match cmd {
            CrewCmd::Join { owner } => {
                if owner == player {
                    return Err(text!("crew-own-aircraft"));
                }
                if !self.pilot_absent(player) {
                    return Err(text!("ce-finish-sortie"));
                }
                if self.station_of(player).is_some() {
                    return Err(text!("crew-at-a-console"));
                }
                if !self.gunner_seat_free(owner) {
                    return Err(text!("crew-seat-taken"));
                }
                self.leave_crew(player);
                let crews = self.crews_mut().expect("campaign");
                match crews.iter_mut().find(|c| c.owner == owner) {
                    Some(c) => c.gunner = Some(player),
                    None => crews.push(Crew { owner, seat: Seat::Pilot, gunner: Some(player) }),
                }
                let back = self.pl.id as usize;
                self.activate(player as usize);
                if self.pl.id == player {
                    self.pl.crew_aim = GunnerAim::default();
                    self.pl.crew_cmd = Commands::default();
                    self.pl.crew_fire = None;
                }
                self.activate(back);
                Ok(())
            }
            CrewCmd::Leave => {
                if self.crew_of(player).is_none_or(|(o, _)| o == player) {
                    return Err(text!("crew-not-a-gunner"));
                }
                self.leave_crew(player);
                Ok(())
            }
            CrewCmd::Seat(seat) => {
                if self.pilot_absent(player) {
                    return Err(text!("ce-not-flying"));
                }
                if seat == Seat::Gunner {
                    let p = self.pilot_ref(player as usize);
                    if !has_gunner_seat(p.spectre, p.apache) {
                        return Err(text!("crew-no-such-seat"));
                    }
                    if self.gunner_of(player).is_some() {
                        return Err(text!("crew-seat-taken"));
                    }
                }
                self.set_owner_seat(player, seat);
                Ok(())
            }
            CrewCmd::Aim(aim) => {
                self.gunner_seat_of(player)?;
                self.with_pilot(player, |sim| sim.pl.crew_aim = aim);
                Ok(())
            }
            CrewCmd::Designate { at, unit } => {
                self.gunner_seat_of(player)?;
                self.with_pilot(player, |sim| {
                    sim.pl.crew_cmd.designate_at = at;
                    sim.pl.crew_cmd.designate_unit = unit;
                });
                Ok(())
            }
            CrewCmd::Fire { station } => {
                self.gunner_seat_of(player)?;
                self.with_pilot(player, |sim| sim.pl.crew_fire = Some(station));
                Ok(())
            }
        }
    }

    fn gunner_seat_of(&self, player: u8) -> Result<u8, Text> {
        self.crews()
            .iter()
            .find(|c| c.gunner == Some(player))
            .map(|c| c.owner)
            .ok_or_else(|| text!("crew-not-a-gunner"))
    }

    fn with_pilot(&mut self, id: u8, f: impl FnOnce(&mut Self)) {
        let back = self.pl.id as usize;
        self.activate(id as usize);
        if self.pl.id == id {
            f(self);
        }
        self.activate(back);
    }

    /// Player `player` leaves the seat he has in another's aircraft (a disconnect, a sortie).
    pub fn leave_crew(&mut self, player: u8) {
        let Some(crews) = self.crews_mut() else { return };
        for c in crews.iter_mut().filter(|c| c.gunner == Some(player)) {
            c.gunner = None;
        }
        self.tidy_crews();
    }

    /// Seats of aircraft no longer in the world are gone (its gunner back in the operations
    /// room), as are the records of an owner alone in the pilot's seat.
    pub(crate) fn tidy_crews(&mut self) {
        let Some(c) = self.campaign.as_ref() else { return };
        if c.defence.crews.is_empty() {
            return;
        }
        let gone: Vec<u8> = c
            .defence
            .crews
            .iter()
            .map(|c| c.owner)
            .filter(|&o| {
                let p = self.pilot_ref(o as usize);
                p.id != o || p.absent || p.eject != 0
            })
            .collect();
        let crews = self.crews_mut().expect("campaign");
        crews.retain(|c| !gone.contains(&c.owner) && (c.seat != Seat::Pilot || c.gunner.is_some()));
    }

    /// Pilot `k`'s report this frame with his gunner's seat in it: the gunner lays the pod and
    /// designates. The AC-130's gunner fires the side guns (the owner's own trigger does not
    /// then); the AH-64's fires the chin gun along his sight and the missiles of the station he
    /// selected, the pilot still firing with his own.
    pub(crate) fn crew_input(&mut self, k: usize, ps: &mut PlayerState) {
        if self.pl.id as usize == k {
            self.pl.crew_gun = false;
        }
        let Some(g) = self.gunner_of(k as u8) else { return };
        let g = g as usize;
        if g >= self.parked.len() || g == self.pl.id as usize || self.parked[g].id as usize != g {
            return;
        }
        let aim = self.parked[g].crew_aim;
        let cmd = std::mem::take(&mut self.parked[g].crew_cmd);
        let fire = self.parked[g].crew_fire.take();
        ps.pod = aim.pod;
        if self.pl.id as usize == k && self.pl.apache {
            self.pl.crew_gun = aim.trigger && aim.station.is_none();
            if fire.is_some() {
                self.pl.crew_launch = fire;
            }
        } else {
            ps.cmd.trigger = aim.trigger;
            ps.cmd.gun = false;
            ps.cmd.fire = false;
        }
        if cmd.designate_at.is_some() || cmd.designate_unit.is_some() {
            ps.cmd.designate_at = cmd.designate_at;
            ps.cmd.designate_unit = cmd.designate_unit;
        }
        if self.pl.id as usize == k && self.pl.spectre && (aim.gun as usize) < crate::gunship::GUNS
        {
            self.pl.side_guns.gun = aim.gun;
        }
    }

    /// The crew view for player `id`'s client (None: he is in nobody's crew).
    pub fn crew_view(&self, id: u8) -> Option<CrewView> {
        if self.mirror {
            return self.crew_view;
        }
        let c = *self.crews().iter().find(|c| c.owner == id || c.gunner == Some(id))?;
        let owner = self.pilot_ref(c.owner as usize);
        let pod = match c.gunner {
            Some(g) if c.owner == id => self.pilot_ref(g as usize).crew_aim.pod,
            _ => owner.last_input.pod,
        };
        Some(CrewView {
            owner: c.owner,
            seat: c.seat,
            gunner: c.gunner,
            side_guns: owner.side_guns,
            spi: owner.spi,
            spi_unit: owner.spi_unit,
            pod,
            gun_ammo: owner.gun_ammo.clamp(0, i16::MAX as i32) as i16,
            stores: std::array::from_fn(|i| {
                owner
                    .bays
                    .get(i)
                    .map_or((-1, 0), |&(w, n)| (w.clamp(-1, 127) as i8, n.clamp(0, 127) as i8))
            }),
            gun_firing: owner.gun_firing,
        })
    }

    /// Campaign: the aircraft of pilot `from` (in the air) goes on with pilot `to` (out of the
    /// world: his gunner) as everything it was (stores, damage, designation); `from`'s sortie
    /// ends as handed over. `to` sits at the gunner's console, the AI flying. Returns where
    /// `to`'s flight model starts.
    pub fn transfer_pilot(&mut self, from: u8, to: u8) -> Option<PlayerState> {
        if from == to || !self.pilot_in_the_air(from) || !self.pilot_absent(to) {
            return None;
        }
        let (fk, tk) = (from as usize, to as usize);
        if tk >= self.parked.len() || self.pilot_ref(tk).id != to {
            return None;
        }
        let back = self.pl.id as usize;
        // His aircraft as `activate` keeps a parked pilot's (rounds, weapons in flight).
        self.activate(fk);
        let mut p = self.pl.clone();
        let nb = self.n_bullets.min(self.bullets.len());
        p.rounds = self.bullets[..nb].to_vec();
        p.missiles.copy_from_slice(&self.proj[crate::ENEMY_SLOTS..]);
        if let Some(o) = self.objects.first() {
            p.scratch0 = *o;
        }
        // His are `to`'s now: none left with him.
        self.pl.rounds.clear();
        self.bullets[..nb].fill(Bullet::default());
        for s in Self::own_slots() {
            self.proj[s] = Projectile::default();
        }
        self.activate(tk);
        let log = std::mem::take(&mut self.pl.log);
        p.id = to;
        p.absent = false;
        p.log = log;
        p.crew_aim = GunnerAim::default();
        p.crew_cmd = Commands::default();
        let start = p.last_input.clone();
        self.pl = p;
        self.pl.rounds.resize(nb, Bullet::default());
        self.bullets[..nb].copy_from_slice(&self.pl.rounds);
        self.proj[crate::ENEMY_SLOTS..].copy_from_slice(&self.pl.missiles);
        if let Some(o) = self.objects.first_mut() {
            *o = self.pl.scratch0;
        }
        for s in self.side_shells.iter_mut().filter(|s| s.pilot == from) {
            s.pilot = to;
        }
        // The seats: `to` owns it, from the gunner's console.
        if let Some(crews) = self.crews_mut() {
            crews.retain(|c| c.owner != from && c.owner != to);
            crews.push(Crew { owner: to, seat: Seat::Gunner, gunner: None });
        }
        self.activate(back);
        self.end_sortie_pilot(from, OUTCOME_HANDED_OVER);
        Some(start)
    }
}
