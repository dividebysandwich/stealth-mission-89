use crate::ballistics;
use crate::guidance::{POPUP_CLIMB, POPUP_PUSHOVER, POPUP_TOP};
use crate::math::{abs16, approx, bearing, cdiv, clamp_q, hi8s, s16, sign, view_z_f};
use crate::types::*;
use crate::weather::optical_seeker;
use crate::{ENEMY_SLOTS, Sim};
use stealth_i18n::{Text, text};

/// The most a laser-guided bomb's fins steer it across its path (g).
const LGB_PULL: f64 = 1.5;
/// Speed (viewZ units/s) a classic missile sinks at while it drops clear.
const CLASSIC_DROP_SPEED: i32 = 20;
/// Radians to BAM.
const BAM: f64 = 65536.0 / std::f64::consts::TAU;
/// Flare / chaff seduction: a missile rolls once per cartridge it sees, at `CM_FAR_PCT` %
/// from `CM_FAR` map units (32 ft) out, rising to `CM_NEAR_PCT` % at `CM_NEAR` and closer
/// (less `CM_DIFF_PCT` % per difficulty level).
pub(crate) const CM_FAR: i32 = 0x240;
const CM_NEAR: i32 = 0x60;
const CM_FAR_PCT: i32 = 15;
const CM_NEAR_PCT: i32 = 80;
const CM_DIFF_PCT: i32 = 5;

/// Chance (%) a flare or chaff cartridge `r` map units from the missile fools it.
pub(crate) fn cm_fool_pct(r: i32, diff: i32) -> i32 {
    let t = (CM_FAR - r.clamp(CM_NEAR, CM_FAR)) * 100 / (CM_FAR - CM_NEAR);
    (CM_FAR_PCT + (CM_NEAR_PCT - CM_FAR_PCT) * t / 100 - CM_DIFF_PCT * diff).max(5)
}

impl Sim {
    /// Seeker acquisition of a target at (tx, ty, talt) for class `cls`. Sets `acq_aim` always and `acq_range` on
    /// success. Optical seekers also need a line of sight through the clouds (`talt` viewZ).
    pub(crate) fn acquire(&mut self, s: usize, tx: i32, ty: i32, talt: i32, cls: i32) -> bool {
        let m = self.proj[s];
        let (r, aim, off) = seeker_geometry(&m, tx, ty);
        self.acq_aim = aim;
        let overshot = off > 0x6000 && s < 8 && m.speed < r;
        // Fixed: class 3 (track-via-missile) sees all around, and classic kept it circling a
        // target it had overshot; now it is spent like the others.
        if off > 0x1000 && (cls != 3 || (overshot && self.opts.fix_classic_bugs)) {
            if overshot {
                self.proj[s].ttl = 0;
            }
            return false;
        }
        if cls == 0 && abs16(s16(m.heading - self.pl.p.heading)) > 0x2000 {
            return false;
        }
        if optical_seeker(m.spec, cls) && !self.sees((m.x, m.y, m.alt), (tx, ty, talt)) {
            return false;
        }
        self.acq_range = r;
        true
    }

    /// The Hellfire in slot `s` takes the targeting pod's spot as its target: the object at it
    /// (within `HELLFIRE_SPOT_RADIUS`), else the plain ground there (scratch object 0). Kept
    /// while the pod sees no ground or the pilot has no line of sight to it.
    pub(crate) fn follow_spot(&mut self, s: usize) {
        let Some((x, y)) = self.pl.last_input.pod.and_then(|p| p.ground) else { return };
        if !self.player_sees(x, y, 0) {
            return;
        }
        let m = self.proj[s];
        if (m.tgt_x, m.tgt_y) == (x, y) && m.target > 0 {
            return;
        }
        let near = (1..self.objects.len())
            .filter(|&i| self.objects[i].flags & oflags::DESTROYED == 0)
            .map(|i| (i, approx(self.objects[i].x - x, self.objects[i].y - y)))
            .filter(|&(_, d)| d <= HELLFIRE_SPOT_RADIUS)
            .min_by_key(|&(_, d)| d);
        let t = match near {
            Some((i, _)) => i as i32,
            None => {
                self.ground_point(x, y);
                0
            }
        };
        let m = &mut self.proj[s];
        m.target = t;
        m.tgt_x = x;
        m.tgt_y = y;
    }

    /// Scratch object 0 as the plain ground at (x, y) (map units): no shape, nothing to destroy.
    pub(crate) fn ground_point(&mut self, x: i32, y: i32) {
        if let Some(o) = self.objects.get_mut(0) {
            *o = GroundObject { x, y, ..Default::default() };
        }
    }

    /// The designated ground point stays object 0 while it is the held ground lock (rounds and
    /// bombs falling on tiles reuse object 0); another lock ends it.
    pub(crate) fn hold_spi(&mut self) {
        let Some((mut x, mut y)) = self.pl.spi else {
            self.pl.spi_unit = None;
            return;
        };
        // A vehicle's point moves with it (until it is destroyed).
        if let Some(k) = self.pl.spi_unit {
            match self.units.get(k as usize).filter(|u| !u.destroyed) {
                Some(u) => {
                    let (ux, uy) = u.map_xy();
                    (x, y) = (ux.round() as i32, uy.round() as i32);
                    self.pl.spi = Some((x, y));
                }
                None => self.pl.spi_unit = None,
            }
        }
        if self.pl.gnd_lock == 0 && self.pl.target_hold {
            self.ground_point(x, y);
        } else {
            self.pl.spi = None;
            self.pl.spi_unit = None;
        }
    }

    pub(crate) fn update_missiles(&mut self, present: &[usize]) {
        self.unit_threats.clear();
        let mut saved0 = Vec::new();
        for &k in present {
            self.activate(k);
            saved0.push((k, self.objects.first().copied()));
            self.pl.lights.rad = false;
            self.pl.lights.ir = false;
            self.pl.lights.klaxon = false;
            self.pl.lights.rad_threat = false;
            self.pl.lights.ir_threat = false;
        }
        // Enemy missiles, each against the pilot it was fired at.
        for s in 0..ENEMY_SLOTS {
            if self.proj[s].ttl == 0 {
                continue;
            }
            let k = self.proj_pilot[s] as usize;
            if present.len() > 1 && present.contains(&k) {
                self.activate(k);
            }
            self.missile_frame(s);
        }
        // Each pilot's own weapons: their kills are his.
        for &k in present {
            self.activate(k);
            self.credit = Some(self.pl.id);
            for s in Self::own_slots() {
                if self.proj[s].ttl != 0 {
                    self.missile_frame(s);
                }
            }
            self.credit = None;
        }
        // The other pilots' missiles homing on a pilot light his warnings.
        for (k, close) in std::mem::take(&mut self.pilot_threats) {
            self.activate(k as usize);
            if self.pl.id != k || self.pl.damage & dmg::MW != 0 {
                continue;
            }
            self.pl.lights.rad_threat = true;
            if self.frame & 2 == 0 {
                self.pl.lights.rad = true;
            }
            if close && self.frame & 3 == 0 {
                self.pl.lights.klaxon = true;
                self.emit(SimEvent::Rwr(RwrTone::AirLaunch));
            }
        }
        self.apply_pilot_hits();
        for (k, o) in saved0 {
            self.activate(k);
            if let (Some(first), Some(o)) = (self.objects.first_mut(), o) {
                *first = o;
            }
        }
    }

    /// One frame of the projectile in slot `s`.
    /// The pilots' weapons' hits on other pilots (`pilot_hits`), each on his turn: damage
    /// with friendly fire. A pilot's gunfire damages another at most every quarter second.
    pub(crate) fn apply_pilot_hits(&mut self) {
        let hits = std::mem::take(&mut self.pilot_hits);
        if !self.opts.friendly_fire {
            return;
        }
        let back = self.pl.id as usize;
        for (k, spec) in hits {
            self.activate(k as usize);
            if self.pl.id != k || self.pl.eject != 0 {
                continue;
            }
            match spec {
                Some(spec) => {
                    let name = self.missile(spec).name.clone();
                    self.msg(text!("sim-hit-by", weapon = name.to_string()));
                    self.player_hit(true);
                }
                None => self.player_hit(false),
            }
        }
        self.activate(back);
    }

    fn missile_frame(&mut self, s: usize) {
        let (tgx, tgy) = if self.decoy_active() {
            (self.pl.cms[0].x, self.pl.cms[0].y)
        } else {
            (self.pl.p.mx, self.pl.p.my)
        };
        let fs = self.fs;
        if self.proj[s].ttl == 0 {
            return;
        }
        if self.proj[s].fall.is_some() {
            self.fall_bomb(s);
            return;
        }
        if self.proj[s].slam.is_some() {
            self.slam_frame(s);
            return;
        }
        if self.proj[s].glide.is_some() {
            self.glide_frame(s);
            return;
        }
        // A sea-skimmer flies with the fleet's war (`naval`).
        if self.proj[s].sea.is_some() {
            return;
        }
        if s < ENEMY_SLOTS && self.proj[s].intercept.is_some() {
            self.intercept_frame(s);
            return;
        }
        let spec = self.proj[s].spec;
        let mt = self.missile(spec).clone();
        let cls = mt.class as i32;
        // Realistic guidance flies this one (`guidance`).
        let guided = self.proj[s].flight.is_some();
        let mut locked = false;
        let mut aim = 0;
        let mut best;
        let mut alt0 = 0;
        let mut hit: i32 = -1;
        // Another pilot this one homes on (and whether on his decoy).
        let mut pilot_hit: Option<(u8, bool)> = None;
        // A Phoenix flying on to where its target was last seen (wu: east, south, altitude).
        let mut point: Option<(i32, i32, i32)> = None;
        if s < 8 {
            // ---- enemy missile ----
            alt0 = self.pl.p.view_z;
            locked = self.acquire(s, tgx, tgy, self.pl.p.view_z, cls);
            best = self.acq_range;
            aim = self.acq_aim;
            // Its radar jammed by a Growler: as against the aircraft's own jammer.
            let launcher = self.proj[s].launcher;
            let jammed = cls > 0 && launcher > 2 && self.site_jammed(launcher as usize);
            let jam = (self.ecm_on() && cls > 0)
                || jammed
                || (self.irj_on() && cls <= 0)
                || (self.alarm_recent() > self.alarm_timer && spec != 0x21);
            if jam && ((self.diff << 8) + 0x180 < best || cls == 1 || cls == -1) {
                aim = self.proj[s].heading;
                locked = false;
            }
            // A pilot who ejected is no target: the missile flies on past his chute.
            if self.pl.eject != 0 {
                locked = false;
            }
            for k in 1..4 {
                let c = self.pl.cms[k];
                if c.ttl <= 0 {
                    continue;
                }
                let attract = (c.kind == 1 && cls <= 0)
                    || (c.kind == 2
                        && (cls == 1
                            || cls == 2
                            || (cls == 3
                                && (hi8s(s16(aim - self.pl.p.heading)).abs() - 0x40).abs()
                                    < 0x40 - 12 * self.diff)));
                if !attract || !self.acquire(s, c.x, c.y, self.pl.p.view_z, cls) {
                    continue;
                }
                let bit = 1u8 << k;
                if self.proj[s].cm_rolled & bit == 0 {
                    self.proj[s].cm_rolled |= bit;
                    if self.rng.random(100) < cm_fool_pct(self.acq_range, self.diff) {
                        self.proj[s].cm_fooled |= bit;
                    }
                }
                if self.proj[s].cm_fooled & bit != 0 {
                    aim = 1;
                    locked = false;
                }
            }
            if best > 0x200 {
                let l = self.proj[s].launcher;
                if l > 2 && self.obj(l).flags & oflags::TRACKING == 0 {
                    locked = false;
                }
                if l <= 0
                    && self
                        .flights
                        .get((-l) as usize)
                        .is_none_or(|u| u.flags & fflags::CONTACT == 0)
                {
                    locked = false;
                }
            }
            if !guided && self.proj[s].speed < (mt.max_speed as i32) >> 6 && self.frame & 1 != 0 {
                self.proj[s].speed += 1;
            }
        } else {
            // ---- player weapon ----
            best = 0x7FFF;
            // The Phoenix and the Sparrow: guided by the radar's track of their own target.
            let radar_guided = cls == 7 && matches!(spec, PHOENIX_SPEC | SPARROW_SPEC);
            if radar_guided {
                let h = self.radar_homing(s);
                if let Some((x, y, z)) = h.at {
                    aim = h.aim;
                    best = h.range;
                    alt0 = z;
                    locked = true;
                    hit = h.unit.map_or(-1, |j| j as i32);
                    pilot_hit = h.pilot.map(|k| (k, false));
                    point = h.point.then_some((x, y, z));
                    if hit >= 0 && best < 0x180 {
                        self.flights[hit as usize].flags |= fflags::HIT;
                    }
                }
                if let Some(k) = pilot_hit {
                    let close = (self.proj[s].speed << 4) > best;
                    self.pilot_threats.push((k.0, close));
                }
            } else if cls == 7 {
                for j in 0..self.flights.len() {
                    let u = self.flights[j];
                    if u.flags & fflags::AIRBORNE == 0 || u.speed == 0 {
                        continue;
                    }
                    // Fixed: a falling wreck is no target (classic homed on it until it hit
                    // the ground).
                    if self.opts.fix_classic_bugs && u.flags & fflags::DESTROYED != 0 {
                        continue;
                    }
                    // The seeker leaves the campaign's friendly aircraft alone unless the
                    // pilot locked one up himself.
                    let friend = self.campaign_flight(j).is_some_and(|f| f.blue);
                    if friend && self.proj[s].target != j as i32 {
                        continue;
                    }
                    if self.acquire(s, u.x, u.y, u.alt, cls) && self.acq_range < best {
                        aim = self.acq_aim;
                        best = self.acq_range;
                        hit = j as i32;
                        alt0 = u.alt;
                        locked = true;
                        if best < 0x180 {
                            self.flights[j].flags |= fflags::HIT;
                        }
                    }
                }
                // Guided: the target locked at launch while the seeker sees it.
                let d = self.proj[s].target;
                if guided && d >= 0 && d != hit && (d as usize) < self.flights.len() {
                    let u = self.flights[d as usize];
                    if u.flags & fflags::AIRBORNE != 0
                        && u.flags & fflags::DESTROYED == 0
                        && u.speed != 0
                        && self.acquire(s, u.x, u.y, u.alt, cls)
                    {
                        aim = self.acq_aim;
                        best = self.acq_range;
                        hit = d;
                        alt0 = u.alt;
                        locked = true;
                    }
                }
                // Realistic AI flight: the aircraft's flares or chaff may fool it (for good),
                // else its pilot may see it coming.
                if hit >= 0 && self.opts.realistic_ai_flight {
                    let fooled = self.proj[s].cm_fooled & crate::ai_flight::DECOYED != 0
                        || self.unit_homed(s, hit as usize, cls);
                    if fooled {
                        hit = -1;
                        locked = false;
                        aim = self.proj[s].heading;
                    }
                }
                // Fired at another pilot: him while the seeker sees him (or his decoy).
                if let Some(k) = self.proj[s].target_pilot.filter(|&k| self.pilot_airborne(k)) {
                    let p = self.pilot_ref(k as usize);
                    let decoy = p.cms[0].ttl > 0;
                    let (x, y) = if decoy { (p.cms[0].x, p.cms[0].y) } else { (p.p.mx, p.p.my) };
                    let z = p.p.view_z;
                    if self.acquire(s, x, y, z, cls) {
                        aim = self.acq_aim;
                        best = self.acq_range;
                        hit = -1;
                        alt0 = z;
                        locked = true;
                        pilot_hit = Some((k, decoy));
                        let close = (self.proj[s].speed << 4) > best;
                        self.pilot_threats.push((k, close));
                    }
                }
            }
            if !guided
                && self.proj[s].drop == 0
                && self.proj[s].speed < (mt.max_speed as i32) >> 6
                && self.frame & 1 != 0
            {
                self.proj[s].speed += 1;
                aim = self.proj[s].heading;
            }
            let t = self.proj[s].target;
            if spec == HELLFIRE_SPEC {
                // The Hellfire rides the laser spot: wherever the targeting pod looks now, while
                // the pilot sees it (else on to where it last was).
                // Its seeker sees the spot wider than the Maverick's sees its target.
                self.follow_spot(s);
                let t = self.proj[s].target;
                if t >= 0 && self.proj[s].tgt_x >= 0 {
                    let o = self.obj(t);
                    let (r, a, off) = seeker_geometry(&self.proj[s], o.x, o.y);
                    self.acq_aim = a;
                    if off <= HELLFIRE_SEEKER {
                        self.acq_range = r;
                        aim = a;
                        best = r;
                        hit = t;
                        alt0 = self.hill_alt(o.x, o.y);
                        locked = true;
                    }
                }
            } else if matches!(cls, 4..=6) && t != -1 {
                if t == 0 {
                    let (gx, gy) = (self.proj[s].tgt_x, self.proj[s].tgt_y);
                    if self.objects.first().is_some_and(|o| (o.x, o.y) != (gx, gy)) {
                        if s >= 8 && self.pl.spi == Some((gx, gy)) {
                            self.ground_point(gx, gy);
                        } else {
                            self.find_object_at(gx, gy);
                            self.objects[0].flags = 0;
                        }
                    }
                }
                let o = self.obj(t);
                if self.acquire(s, o.x, o.y, 0, cls) {
                    aim = self.acq_aim;
                    best = self.acq_range;
                    hit = t;
                    alt0 = self.hill_alt(o.x, o.y);
                    locked = true;
                }
            }
            let gl = self.pl.gnd_lock;
            if spec == 28 && gl >= 0 && (gl as usize) < self.objects.len() {
                let o = self.obj(gl);
                // The laser spot needs the designator's line of sight too.
                if self.player_sees(o.x, o.y, 0) && self.acquire(s, o.x, o.y, 0, 28) {
                    aim = self.acq_aim;
                    best = self.acq_range;
                    hit = gl;
                    alt0 = self.hill_alt(o.x, o.y);
                    locked = true;
                }
            }
        }
        if self.proj[s].ttl == 0 {
            // Overshot in acquire.
            return;
        }
        // Warning lights / klaxon.
        if locked
            && s < 8
            && abs16(s16(self.acq_aim - self.proj[s].heading)) < 0x1000
            && self.pl.damage & dmg::MW == 0
            && !self.decoy_active()
        {
            self.pl.lights.rad_threat |= cls != 0;
            self.pl.lights.ir_threat |= cls <= 0;
            if cls <= 0 && self.frame & 2 != 0 {
                self.pl.lights.ir = true;
            }
            if cls != 0 && self.frame & 2 == 0 {
                self.pl.lights.rad = true;
            }
            if self.frame & 3 == 0 && (self.proj[s].speed << 4) > best {
                self.pl.lights.klaxon = true;
                // Site launchers are object indices > 2, aircraft are -flight index.
                let tone =
                    if self.proj[s].launcher > 2 { RwrTone::SamLaunch } else { RwrTone::AirLaunch };
                self.emit(SimEvent::Rwr(tone));
            }
        }
        // Steering.
        let fused = if guided {
            let target = if s < 8 {
                Some(if self.decoy_active() {
                    let c = self.pl.cms[0];
                    let p =
                        [c.x as f64 * 32.0 + 16.0, c.y as f64 * 32.0 + 16.0, self.pl.p.alt as f64];
                    crate::guidance::Target { p, v: [0.0; 3] }
                } else {
                    self.player_target()
                })
            } else if let Some((k, decoy)) = pilot_hit {
                Some(if decoy {
                    let p = self.pilot_ref(k as usize);
                    let c = p.cms[0];
                    let p = [c.x as f64 * 32.0 + 16.0, c.y as f64 * 32.0 + 16.0, p.p.alt as f64];
                    crate::guidance::Target { p, v: [0.0; 3] }
                } else {
                    self.pilot_target(k)
                })
            } else if let Some((x, y, z)) = point {
                Some(crate::guidance::Target { p: [x as f64, y as f64, z as f64], v: [0.0; 3] })
            } else if hit < 0 {
                None
            } else if cls == 7 {
                Some(self.unit_target(hit as usize))
            } else {
                Some(self.object_target(hit))
            };
            // The fuse goes off near the player whatever the seeker sees, and at the decoy it
            // steers for (harmlessly; a missile that cannot reach it would circle it).
            let fuse = if s < 8 && self.pl.eject != 0 {
                None
            } else if s < 8 && !self.decoy_active() {
                Some(self.player_target())
            } else if point.is_some() {
                // Nothing to burst at where the target was.
                None
            } else {
                target
            };
            let fused = self.fly_guided(s, target.filter(|_| locked && aim != 0), fuse);
            if self.proj[s].ttl == 0 {
                return;
            }
            fused
        } else {
            let mut m = self.proj[s];
            let fused = if m.drop > 0 {
                self.classic_drop(&mut m);
                false
            } else {
                self.classic_step(&mut m, s < 8, locked && point.is_none(), aim, best, alt0)
            };
            self.proj[s] = m;
            // Terrain contact: the flat world and the rolling hills, objects are not tested.
            if m.alt < self.hill_alt(m.x, m.y) {
                self.ground_impact(s);
            }
            fused
        };
        // Proximity fuse.
        let m = self.proj[s];
        if fused {
            if s < 8 {
                if m.ttl > 4 * fs {
                    self.flash = Flash { x: m.x, y: m.y, alt: m.alt, timer: 0x10 };
                    self.emit(SimEvent::Explosion { x: m.x, y: m.y, alt: m.alt, kind: 0x10 });
                    self.proj[s].ttl = if guided { 0 } else { 2 * fs };
                    if !self.decoy_active() {
                        self.msg(text!("sim-hit-by", weapon = mt.name.clone()));
                        self.player_hit(true);
                        if !self.training() {
                            self.log_event(5, spec);
                        }
                    }
                }
            } else if let Some((k, decoy)) = pilot_hit {
                // At another pilot: the burst is everybody's to see, the damage his (with
                // friendly fire), applied when his turn comes.
                let p = self.pilot_ref(k as usize).p;
                let (x, y, alt) = if decoy { (m.x, m.y, m.alt) } else { (p.mx, p.my, p.view_z) };
                self.flash = Flash { x, y, alt, timer: 0x10 };
                self.emit(SimEvent::Explosion { x, y, alt, kind: 0x10 });
                self.proj[s].ttl = 0;
                if !decoy {
                    self.pilot_hits.push((k, Some(spec)));
                }
            } else if cls == 7 {
                if hit >= 0 {
                    let u = self.flights[hit as usize];
                    let pm = &mut self.proj[s];
                    pm.x = u.x;
                    pm.y = u.y;
                    pm.alt = u.alt;
                    pm.ttl = 2 * fs;
                    self.flash = Flash { x: u.x, y: u.y, alt: u.alt, timer: 0x10 };
                    self.emit(SimEvent::Explosion { x: u.x, y: u.y, alt: u.alt, kind: 0x10 });
                    self.destroy_aircraft(hit as usize);
                    let n = self.ac_type(u.spec).name.clone();
                    self.msg(text!(
                        "sim-target-hit-by",
                        target = n.to_string(),
                        weapon = mt.name.clone()
                    ));
                    if self.opts.fix_classic_bugs {
                        // Classic: the missile sits on the wreck for 2 s, still locked, and
                        // repeats the hit every frame. Fixed: it is spent.
                        self.proj[s].ttl = 0;
                    }
                }
            } else if hit == 0 && self.objects.first().is_some_and(|o| o.shape == 0) {
                // On the plain ground at the spot.
                let o = self.obj(0);
                self.flash = Flash { x: o.x, y: o.y, alt: 0, timer: 0x10 };
                self.emit(SimEvent::Explosion { x: o.x, y: o.y, alt: 0, kind: 0x10 });
                self.proj[s].ttl = 0;
                self.msg(text!("sim-impact", weapon = mt.name.clone()));
            } else if hit >= 0 {
                let o = self.obj(hit);
                self.flash = Flash { x: o.x, y: o.y, alt: 0, timer: 0x10 };
                self.emit(SimEvent::Explosion { x: o.x, y: o.y, alt: 0, kind: 0x10 });
                self.proj[s].ttl = 0;
                let r = self.rng.random(4);
                let target = if self.compat(m.weapon, hit) > r {
                    self.destroy_ground(hit);
                    Text::raw(self.place_name(hit as usize))
                } else {
                    text!("sim-ineffective")
                };
                self.alarm_timer = self.alarm_init();
                self.ref_x = o.x;
                self.ref_y = o.y;
                self.ref_alt = 3000;
                if self.present().len() > 1 {
                    self.pl.seen = Some((o.x, o.y, 3000));
                }
                self.msg(text!("sim-target-hit-by", target = target, weapon = mt.name.clone()));
            }
        }
    }

    /// A frame of classic missile `m` dropping clear of the aircraft: on unpowered and
    /// unsteered, sinking, until its motor lights.
    pub(crate) fn classic_drop(&self, m: &mut Projectile) {
        self.classic_step(m, false, false, 0, 0, 0);
        m.alt -= (CLASSIC_DROP_SPEED / self.fs).max(1);
        m.drop -= 1;
    }

    /// One frame of the classic flight of missile `m` (an enemy missile when `enemy`): turns
    /// toward `aim` when `locked` (the target `best` map units away at viewZ `alt0`) and moves.
    /// True when the fuse goes off.
    pub(crate) fn classic_step(
        &self,
        m: &mut Projectile,
        enemy: bool,
        locked: bool,
        aim: i32,
        best: i32,
        alt0: i32,
    ) -> bool {
        let (diff, fs) = (self.diff, self.fs);
        let turn = self.missile(m.spec).turn as i32;
        m.locked = locked;
        if aim != 0 && locked {
            let mut d = s16(aim - m.heading);
            if enemy {
                d = clamp_q(d, -(diff + 1) << 8, (diff + 1) << 8);
            }
            d = clamp_q(d, -turn << 7, turn << 7);
            m.heading = s16(m.heading + cdiv(d * 4, fs));
            m.roll = s16(d * 2);
            let ab = best.abs();
            let mut pa = if enemy && best < 0x400 {
                bearing(s16(alt0 - m.alt) >> 4, ab)
            } else {
                bearing((s16(alt0 - m.alt) >> 5) + if ab > 0x140 { ab >> 3 } else { 0 }, ab)
            };
            // A Sidearm's pop-up: it climbs until turning down to the radar at its turn rate
            // would take a good part of the time left to it, or it is high enough above it,
            // then steers for it as any other.
            if m.popup != Popup::Off {
                let above = m.alt - alt0;
                let los = bearing(-above >> 5, ab);
                let rate = ((turn << 8) * 4) as f64 / BAM;
                // At the speed it is getting up to.
                let top = (self.missile(m.spec).max_speed as i32 >> 6).max(m.speed).max(1);
                let to_go = ((above >> 5) as f64).hypot(ab as f64) / (top * 4) as f64;
                let down = s16(m.pitch - los) as f64 / BAM;
                if down.max(0.0) / rate > POPUP_PUSHOVER * to_go || above as f64 >= POPUP_TOP {
                    m.popup = Popup::Off;
                } else {
                    pa = (POPUP_CLIMB * BAM) as i32;
                }
            }
            let mut dp = s16(pa - m.pitch);
            if enemy {
                dp = clamp_q(dp, -(diff + 1) << 9, (diff + 1) << 9);
            }
            dp = clamp_q(dp, -turn << 8, turn << 8);
            m.pitch = s16(m.pitch + cdiv(dp * 4, fs));
        } else if m.pitch > 0 && m.spec != 30 {
            m.pitch = s16(m.pitch - cdiv(sign(m.pitch) * 0x800, fs));
        }
        if m.spec == 28 {
            m.pitch = m.pitch.min(-0x800);
        }
        if m.spec == 30 || m.alt == 1 {
            m.pitch = s16(m.pitch - cdiv(0x800, fs)).max(s16(m.launcher));
        }
        let step = cdiv(self.trig.cos_mul(m.pitch, m.speed) * 4, fs);
        m.x = (m.x + self.trig.sin_mul(m.heading, step)) & 0xFFFF;
        m.y = (m.y - self.trig.cos_mul(m.heading, step)) & 0xFFFF;
        m.alt += self.trig.sin_mul(m.pitch, cdiv(m.speed << 7, fs));
        m.ttl -= 1;
        locked && (s16(alt0 - m.alt).abs() >> 5) + best < cdiv(m.speed << 3, fs)
    }

    /// A ballistic bomb's frame: it moves along its path ([`ballistics`]) and lands where the
    /// path meets the ground.
    fn fall_bomb(&mut self, s: usize) {
        let m = self.proj[s];
        let Some(f) = m.fall else { return };
        let Some(k) = self.fall_drag(m.spec) else { return };
        let fine = ROUND_FINE as f64;
        let before = [f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine];
        let (mut p, mut v) = (before, [f.vx as f64 / fine, f.vy as f64 / fine, f.vz as f64 / fine]);
        let dt = 1.0 / self.fs as f64;
        let sfw = m.spec == CBU105_SPEC && s >= ENEMY_SLOTS;
        if m.spec == 28 && s >= ENEMY_SLOTS {
            self.steer_lgb(&p, &mut v, dt);
        } else if sfw {
            self.steer_sfw(&m, &p, &mut v, dt);
        }
        ballistics::step(&mut p, &mut v, k, dt);
        // The CBU-105 opens at its burst altitude.
        let opens = sfw
            && p[2] - self.hill_alt((p[0] / 32.0) as i32, (p[1] / 32.0) as i32) as f64
                <= crate::sfw::burst_of(&m) as f64;
        // The ground (the rolling hills) where the bomb is now.
        let ground = self.hill_alt((p[0] / 32.0) as i32, (p[1] / 32.0) as i32) as f64;
        let landed = p[2] <= ground;
        if landed {
            // Where the step crossed the ground.
            let (b, a) = (before[2] - ground, p[2] - ground);
            let t = if b > 0.0 { b / (b - a) } else { 0.0 };
            p = [before[0] + (p[0] - before[0]) * t, before[1] + (p[1] - before[1]) * t, ground];
        }
        let q = |a: f64| (a * fine).round() as i32;
        let nf = BombFall {
            x: q(p[0]),
            y: q(p[1]),
            alt: q(p[2]),
            vx: q(v[0]),
            vy: q(v[1]),
            vz: q(v[2]),
        };
        let bam = 65536.0 / std::f64::consts::TAU;
        let pm = &mut self.proj[s];
        pm.fall = Some(nf);
        pm.x = (nf.x / (32 * ROUND_FINE)) & 0xFFFF;
        pm.y = (nf.y / (32 * ROUND_FINE)) & 0xFFFF;
        pm.alt = if landed { -1 } else { view_z_f(p[2]) as i32 };
        pm.heading = s16((v[0].atan2(-v[1]) * bam) as i32);
        pm.pitch = s16((v[2].atan2(v[0].hypot(v[1])) * bam) as i32);
        pm.ttl -= 1;
        if opens && !landed {
            self.sfw_dispense(s, p, v);
        } else if landed {
            self.ground_impact(s);
        }
    }

    /// How a store falls ballistically ([`ballistics::drag`]), if it does: under realistic
    /// guidance the laser-guided bombs too, like free-fall ones (steered by [`Self::steer_lgb`]).
    pub(crate) fn fall_drag(&self, spec: i32) -> Option<f64> {
        ballistics::drag(spec, self.opts.fix_classic_bugs).or_else(|| {
            // The CBU-105 is new: it falls like this with either guidance.
            (spec == 28 && self.opts.realistic_guidance || spec == CBU105_SPEC)
                .then(|| ballistics::drag(30, self.opts.fix_classic_bugs))
                .flatten()
        })
    }

    /// A falling laser-guided bomb at `p` (ft: east, south, up) steering its velocity `v` for
    /// `dt` s toward the laser spot on the ground lock while the designator sees it: across
    /// its path, for where it would land, with its fins' limited pull.
    fn steer_lgb(&self, p: &[f64; 3], v: &mut [f64; 3], dt: f64) {
        let gl = self.pl.gnd_lock;
        if gl < 0 || gl as usize >= self.objects.len() {
            return;
        }
        let o = self.obj(gl);
        if !self.player_sees(o.x, o.y, 0) {
            return;
        }
        let (tx, ty) = ((o.x as f64 + 0.5) * 32.0, (o.y as f64 + 0.5) * 32.0);
        let h = p[2] - self.hill_alt(o.x, o.y) as f64;
        if h <= 0.0 {
            return;
        }
        // Seconds to the ground at the target's height.
        let g = ballistics::GRAVITY;
        let t_go = (v[2] + (v[2] * v[2] + 2.0 * g * h).sqrt()) / g;
        if t_go < 0.1 {
            return;
        }
        let want = [(tx - p[0]) / t_go, (ty - p[1]) / t_go];
        let dv = [want[0] - v[0], want[1] - v[1]];
        let n = dv[0].hypot(dv[1]);
        let most = LGB_PULL * g * dt;
        let k = if n > most { most / n } else { 1.0 };
        v[0] += dv[0] * k;
        v[1] += dv[1] * k;
    }

    /// A falling CBU-105 `m` at `p` (ft: east, south, up) steering its velocity `v` for `dt` s
    /// toward the point marked at its release, on its own: for opening at its burst altitude
    /// short of the point by as far as the submunitions drift on, so they settle over it.
    pub(crate) fn steer_sfw(&self, m: &Projectile, p: &[f64; 3], v: &mut [f64; 3], dt: f64) {
        let Some((ax, ay)) = m.aim else { return };
        let (ax, ay) = (ax as f64, ay as f64);
        let open = self.hill_alt((ax / 32.0) as i32, (ay / 32.0) as i32) as f64
            + crate::sfw::burst_of(m) as f64;
        let h = p[2] - open;
        if h <= 0.0 {
            return;
        }
        // Seconds to the burst altitude.
        let g = ballistics::GRAVITY;
        let t_go = (v[2] + (v[2] * v[2] + 2.0 * g * h).sqrt()) / g;
        if t_go < 0.1 {
            return;
        }
        // The submunitions go on at the dispenser's speed for their drift time.
        let vz = -(v[2] * v[2] + 2.0 * g * h).sqrt();
        let t = t_go + crate::sfw::drift_secs(crate::sfw::burst_of(m) as f64, vz, dt);
        let want = [(ax - p[0]) / t, (ay - p[1]) / t];
        let dv = [want[0] - v[0], want[1] - v[1]];
        let n = dv[0].hypot(dv[1]);
        let most = crate::sfw::SFW_PULL_G * g * dt;
        let k = if n > most { most / n } else { 1.0 };
        v[0] += dv[0] * k;
        v[1] += dv[1] * k;
    }

    /// Ground contact of a projectile.
    pub(crate) fn ground_impact(&mut self, s: usize) {
        let m = self.proj[s];
        // A Hellfire into the sea over a submarine at periscope depth.
        if m.spec == HELLFIRE_SPEC && s >= ENEMY_SLOTS && !self.asw.subs.is_empty() {
            let (x, y, _) = m.map_pos();
            let at = [x * crate::aaa::M_PER_MAP, (0x8000 as f64 - y) * crate::aaa::M_PER_MAP];
            self.hellfire_on_sea(at);
        }
        let (ex, ey) = (m.x, m.y);
        let mut ealt = m.alt;
        self.proj[s].ttl = 0;
        self.flash = Flash { x: ex, y: ey, alt: ealt, timer: -6 };
        let long = if s >= 8 {
            self.weapon_types[(m.weapon.max(0) as usize).min(self.weapon_types.len() - 1)]
                .long_name
                .clone()
        } else {
            String::new()
        };
        let mut kind = -6;
        if self.opts.fix_classic_bugs && s >= 8 && m.spec == 38 {
            // Intended airdrop rule, unreachable in classic mode.
            let mut delivered = false;
            for k in 0..2 {
                let t = self.pl.targets[k];
                let o = self.obj(t.object);
                if approx(ex - o.x, ey - o.y) < 0x100
                    && t.mission_type == 3
                    && self.mission_tick < self.pl.deadline
                {
                    self.msg(text!("sim-equipment-delivered"));
                    self.mark_target_reached(k);
                    delivered = true;
                }
            }
            if !delivered {
                self.msg(text!("sim-bad-drop"));
            }
        } else if matches!(m.spec, 28..=30 | CBU105_SPEC) {
            self.sound(5, 2);
            let mut text = text!("sim-misses", weapon = long.to_string());
            let o = self.find_object_at(ex, ey);
            if o != -1 && self.obj(o).flags & oflags::DESTROYED == 0 {
                let (tx, ty) =
                    self.pl.last_tile.map_or((self.obj(o).x, self.obj(o).y), |t| t.map_pos());
                let dist = approx(tx - ex, ty - ey);
                let lim = (self.compat(m.weapon, o) << 6) / (self.diff + 2);
                if (lim as u16) > (dist as u16) {
                    self.destroy_ground(o);
                    let place = self.place_name(o as usize);
                    text = text!("sim-destroyed-by", target = place, weapon = long.to_string());
                    kind = 0x10;
                    ealt = 0;
                }
            }
            self.msg(text);
            // The heavy bombs' blast takes the vehicles farther off too.
            if s >= 8
                && let Some(scale) = crate::weapons::blast_scale(m.weapon)
            {
                self.heavy_blast(ex, ey, scale);
            }
            let d = approx(self.pl.p.mx - ex, self.pl.p.my - ey) + (self.pl.p.view_z >> 5);
            if self.diff * 32 > d {
                // Classic builds a "Blast damage" message after the miss text but never shows it.
                self.player_hit(false);
            }
        } else if s >= 8 {
            self.msg(text!("sim-ground-impact", weapon = long.to_string()));
        }
        self.emit(SimEvent::Explosion { x: ex, y: ey, alt: ealt, kind });
    }

    /// Countermeasure timers: only the first active slot counts down.
    pub(crate) fn update_cm_timers(&mut self) {
        if let Some(c) = self.pl.cms.iter_mut().find(|c| c.ttl > 0) {
            c.ttl -= 1;
        }
    }
}

/// Range (map units), bearing and angle off the heading of (tx, ty) from missile `m`.
/// What a radar-guided missile (the Phoenix, the Sparrow) homes on this frame
/// ([`Sim::radar_homing`]).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RadarHoming {
    /// Where it steers (wu: east, south, the altitude its target is drawn at), None: nowhere
    /// (it flies on unguided).
    pub at: Option<(i32, i32, i32)>,
    /// A flight unit or another pilot at `at`, or only the point (`point`: a Phoenix whose
    /// track was lost flying on to where its target was).
    pub unit: Option<usize>,
    pub pilot: Option<u8>,
    pub point: bool,
    /// The bearing (BAM) and range (map units) to it.
    pub aim: i32,
    pub range: i32,
}

impl Sim {
    /// The radar-guided missile in slot `s` (the player's Phoenix or Sparrow): what it homes on.
    ///
    /// - The Sparrow (semi-active) homes on its target while the radar's single target track
    ///   holds it (the air lock, or the locked pilot) and its seeker sees it; else it flies on.
    /// - The Phoenix in midcourse homes on its target while the radar tracks it
    ///   (`PlayerState::support`, the air lock too), remembering where it is; with the track
    ///   lost it flies on to that point. Within [`PHOENIX_ACTIVE`] of it its own seeker comes
    ///   on, and from then it homes on whatever aircraft that seeker sees within that range
    ///   ahead (its target first), as an AMRAAM does.
    pub(crate) fn radar_homing(&mut self, s: usize) -> RadarHoming {
        let m = self.proj[s];
        let t = m.target;
        let alive = |sim: &Sim, j: i32| {
            j >= 0
                && (j as usize) < sim.flights.len()
                && sim.flights[j as usize].flags & fflags::AIRBORNE != 0
                && sim.flights[j as usize].flags & fflags::DESTROYED == 0
                && sim.flights[j as usize].speed != 0
        };
        let unit_at = |sim: &Sim, j: usize| {
            let tg = sim.unit_target(j);
            (tg.p[0] as i32, tg.p[1] as i32, tg.p[2] as i32)
        };
        let pilot = m.target_pilot.filter(|&k| self.pilot_airborne(k));
        let pilot_at = |sim: &Sim, k: u8| {
            let tg = sim.pilot_target(k);
            (tg.p[0] as i32, tg.p[1] as i32, tg.p[2] as i32)
        };
        let map = |(x, y, _): (i32, i32, i32)| (x >> 5, y >> 5);
        let geometry = |at: (i32, i32, i32)| {
            let (x, y) = map(at);
            let (r, aim, off) = seeker_geometry(&m, x, y);
            (r, aim, off)
        };
        let home = |at, unit, pilot, point| {
            let (range, aim, _) = geometry(at);
            RadarHoming { at: Some(at), unit, pilot, point, aim, range }
        };
        let input = &self.pl.last_input;
        let tracked =
            alive(self, t) && (self.pl.air_lock == t || input.support.contains(&(t as u16)));
        let pilot_tracked =
            pilot.filter(|&k| self.pl.pilot_lock == Some(k) || input.support_pilots.contains(&k));
        if m.spec == SPARROW_SPEC {
            // Lit by the radar's single target track only.
            let stt = if let Some(k) = pilot.filter(|&k| self.pl.pilot_lock == Some(k)) {
                Some((pilot_at(self, k), None, Some(k)))
            } else if alive(self, t) && self.pl.air_lock == t {
                Some((unit_at(self, t as usize), Some(t as usize), None))
            } else {
                None
            };
            let Some((at, unit, pk)) = stt else { return RadarHoming::default() };
            let (x, y) = map(at);
            if !self.acquire(s, x, y, at.2, 7) {
                return RadarHoming::default();
            }
            return home(at, unit, pk, false);
        }
        // The Phoenix.
        if let Some(mut mc) = m.midcourse {
            let seen = if let Some(k) = pilot_tracked {
                Some((pilot_at(self, k), None, Some(k)))
            } else if tracked {
                Some((unit_at(self, t as usize), Some(t as usize), None))
            } else {
                None
            };
            let (at, unit, pk, point) = match seen {
                Some((at, unit, pk)) => {
                    mc.at = at;
                    (at, unit, pk, false)
                }
                None => (mc.at, None, None, true),
            };
            let (range, ..) = geometry(at);
            if range > PHOENIX_ACTIVE {
                self.proj[s].midcourse = Some(mc);
                return home(at, unit, pk, point);
            }
            // Close enough: its own seeker comes on.
            self.proj[s].midcourse = None;
        }
        // Active: its target while its seeker sees it within its range, else the nearest it
        // sees (not the campaign's friends).
        if let Some(k) = pilot {
            let at = pilot_at(self, k);
            let (x, y) = map(at);
            if self.acquire(s, x, y, at.2, 7) && self.acq_range <= PHOENIX_ACTIVE {
                return home(at, None, Some(k), false);
            }
        }
        let mut best: Option<(i32, usize)> = None;
        for j in 0..self.flights.len() {
            if !alive(self, j as i32) {
                continue;
            }
            let friend = self.campaign_flight(j).is_some_and(|f| f.blue);
            if friend && t != j as i32 {
                continue;
            }
            let u = self.flights[j];
            if !self.acquire(s, u.x, u.y, u.alt, 7) || self.acq_range > PHOENIX_ACTIVE {
                continue;
            }
            // Its own target wins while it sees it.
            let r = if j as i32 == t { -1 } else { self.acq_range };
            if best.is_none_or(|(b, _)| r < b) {
                best = Some((r, j));
            }
        }
        match best {
            Some((_, j)) if !(self.opts.realistic_ai_flight && self.unit_homed(s, j, 7)) => {
                home(unit_at(self, j), Some(j), None, false)
            }
            _ => RadarHoming::default(),
        }
    }
}

pub(crate) fn seeker_geometry(m: &Projectile, tx: i32, ty: i32) -> (i32, i32, i32) {
    let dx = s16(tx - m.x);
    let dy = s16(ty - m.y);
    let aim = bearing(dx, s16(-dy));
    (approx(dx, dy), aim, abs16(s16(aim - m.heading)))
}
