use crate::math::{abs16, approx, cdiv, clamp_v, s16};
use crate::types::*;
use crate::{PlayerView, Sim, ballistics};

/// Frames a nearest-target request waits for the HUD to switch to the weapon's mode.
const NEAREST_WAIT: u8 = 8;
/// Seconds the RAH-66's doors stay open after the last launch they opened for.
const BAY_AUTO_CLOSE: i32 = 4;
/// Seconds a missile dropped off its pylon falls clear before its motor lights (the Maverick,
/// the Harpoon, the Penguin; the others light theirs on the rail), and one ejected from the
/// F-117A's bays (all of them, lest they fly through the fuselage).
const DROP_SECS: f64 = 0.5;
const BAY_DROP_SECS: f64 = 0.6;
/// Speed (wu/s) the bay's ejectors push a store down at.
const EJECT_SPEED: f64 = 10.0;

impl Sim {
    /// Key handling for this frame's commands.
    pub(crate) fn handle_keys(&mut self, ps: &PlayerState) {
        let c = &ps.cmd;
        // HUD mode changes (F2): entering AIR clears both locks, entering GND the ground lock.
        // Entering AIR also lets go of a held lock (a ground lock of the pod or the HSD), so
        // the HUD picks the aircraft ahead by itself again.
        if self.pl.hud_mode != self.pl.prev_hud_mode {
            match self.pl.hud_mode {
                HudMode::Air => {
                    self.pl.air_lock = -1;
                    self.pl.gnd_lock = -1;
                    self.pl.pilot_lock = None;
                    self.pl.target_hold = false;
                }
                HudMode::Gnd => self.pl.gnd_lock = -1,
                HudMode::Nav => {}
            }
            // Realistic avionics: the new mode's own weapon (none fits: the selection stays,
            // and does not fire).
            if ps.mode_weapons && !self.selection_in_mode(ps) {
                self.next_weapon(ps);
            }
        }
        // After ejecting, the flare key fires the pilot's signal flares (csar.rs).
        if c.flare && self.pl.eject == 0 {
            self.release_cm(1);
        }
        if c.chaff && self.pl.eject == 0 {
            self.release_cm(2);
        }
        if c.decoy && self.pl.eject == 0 {
            self.release_cm(3);
        }
        if c.toggle_irj {
            self.pl.pflags ^= pflags::IRJ;
        }
        if c.toggle_ecm {
            self.pl.pflags ^= pflags::ECM;
        }
        if c.toggle_bay && self.pl.damage & dmg::BD == 0 && !self.pl.external_stores {
            self.pl.bay_req = !self.pl.bay_req;
            // Set by hand, the RAH-66's doors stay as they are.
            self.pl.bay_close = 0;
            self.pl.bay_pending = false;
            self.sound(0x1F, 1);
        }
        if c.next_bay {
            let before = (self.pl.gun_selected, self.pl.cur_bay);
            self.next_weapon(ps);
            if ps.mode_weapons
                && before == (self.pl.gun_selected, self.pl.cur_bay)
                && !self.selection_in_mode(ps)
            {
                self.mode_msg();
            }
        }
        if c.designate {
            self.pl.target_hold = false;
            self.pl.pflags ^= pflags::DESIGNATE;
            if self.pl.camera_mfd {
                self.camera_key(0);
            }
        }
        if c.next_target {
            self.pl.target_hold = false;
            self.pl.targeting.target_killed = false;
            self.pl.gnd_lock |= 0x80;
            self.pl.air_lock |= 0x80;
            self.pl.pilot_lock = None;
        }
        if let Some(d) = c.camera_dir {
            self.camera_key(d as i32);
        }
        if c.nearest_target {
            self.pl.target_hold = false;
            self.camera_key(0);
            self.pl.nearest_req = NEAREST_WAIT;
        }
        if c.clear_target {
            self.pl.air_lock = -1;
            self.pl.gnd_lock = -1;
            self.pl.pilot_lock = None;
            self.pl.nearest_req = 0;
            self.pl.pflags &= !pflags::DESIGNATE;
            self.pl.target_hold = true;
        }
        if c.toggle_training {
            self.pl.pflags ^= pflags::TRAINING;
        }
        if c.resupply && self.training() {
            self.resupply();
        }
        if c.eject {
            self.do_eject();
        }
        // Display locks, after the HUD mode change above has cleared the old ones.
        if let Some(o) = c.lock_object.map(usize::from) {
            if self.objects.get(o).is_some_and(|ob| ob.flags & oflags::DESTROYED == 0) {
                self.pl.gnd_lock = o as i32;
                self.pl.target_hold = true;
                self.pl.nearest_req = 0;
                self.pl.targeting.target_killed = false;
                self.pl.pflags &= !pflags::DESIGNATE;
            } else {
                self.msg("No target");
            }
        }
        if let Some(k) = c.designate_unit
            && let Some(u) = self.units.get(k as usize).filter(|u| !u.destroyed)
        {
            let (ux, uy) = u.map_xy();
            let (x, y) = (ux.round() as i32, uy.round() as i32);
            if !self.player_sees(x, y, 0) {
                self.msg("Target obscured");
            } else {
                self.ground_point(x, y);
                self.pl.gnd_lock = 0;
                self.pl.spi = Some((x, y));
                self.pl.spi_unit = Some(k);
                self.pl.target_hold = true;
                self.pl.nearest_req = 0;
                self.pl.targeting.target_killed = false;
                self.pl.pflags &= !pflags::DESIGNATE;
            }
        }
        if let Some((x, y)) = c.designate_at {
            self.pl.spi_unit = None;
            // The object there (a world object, else a scenery tile), else the plain ground
            // (scratch object 0).
            let world = (3..self.objects.len())
                .filter(|&i| self.objects[i].flags & oflags::DESTROYED == 0)
                .map(|i| (i as i32, approx(self.objects[i].x - x, self.objects[i].y - y)))
                .filter(|&(_, d)| d <= DESIGNATE_RADIUS)
                .min_by_key(|&(_, d)| d)
                .map(|(i, _)| i);
            let near = world.or_else(|| {
                let o = self.find_object_at(x, y);
                let ob = self.obj(o);
                (o >= 0 && approx(ob.x - x, ob.y - y) <= DESIGNATE_RADIUS).then_some(o)
            });
            let (o, spi) = match near {
                Some(o) => (o, None),
                None => (0, Some((x, y))),
            };
            if spi.is_some() {
                self.ground_point(x, y);
            }
            if !self.player_sees(self.obj(o).x, self.obj(o).y, 0) {
                self.msg("Target obscured");
            } else {
                self.pl.gnd_lock = o;
                self.pl.spi = spi;
                self.pl.target_hold = true;
                self.pl.nearest_req = 0;
                self.pl.targeting.target_killed = false;
                self.pl.pflags &= !pflags::DESIGNATE;
                if self.smoke_src == 0 {
                    self.smoke_src = -1;
                }
            }
        }
        if let Some(u) = c.lock_unit.map(usize::from) {
            if self.flights.get(u).is_some_and(|f| {
                f.flags & fflags::AIRBORNE != 0 && f.flags & fflags::DESTROYED == 0
            }) {
                self.pl.air_lock = u as i32;
                self.pl.pilot_lock = None;
                self.pl.target_hold = true;
                self.pl.nearest_req = 0;
            } else {
                self.msg("No target");
            }
        }
        if let Some(k) = c.lock_pilot {
            if self.pilot_airborne(k) {
                self.pl.pilot_lock = Some(k);
                self.pl.air_lock = -1;
                self.pl.target_hold = true;
                self.pl.nearest_req = 0;
            } else {
                self.msg("No target");
            }
        }
    }

    /// Space: the next weapon carried, then the cannon (stations with the selected weapon are
    /// skipped; classic mode steps through the 2 or 4 bays, the same with a different weapon in
    /// each).
    /// The HUD mode store `w` (weapon type) is used in with realistic avionics: None for the
    /// camera and the tanks, used in any mode.
    fn store_mode(&self, w: i32) -> Option<HudMode> {
        let spec = self.weapon_types.get(usize::try_from(w).ok()?)?.spec as i32;
        if spec <= 0 {
            return None;
        }
        Some(if self.missile(spec).class == 7 { HudMode::Air } else { HudMode::Gnd })
    }

    /// The cannon (`None`) or the store at station `i` may be used in the HUD's mode.
    fn usable_in_mode(&self, ps: &PlayerState, i: Option<usize>) -> bool {
        if !ps.mode_weapons {
            return true;
        }
        let mode = self.pl.hud_mode;
        match i {
            None => mode != HudMode::Nav,
            Some(i) => {
                self.pl.bays.get(i).is_none_or(|b| self.store_mode(b.0).is_none_or(|m| m == mode))
            }
        }
    }

    /// The selected weapon may be used in the HUD's mode.
    fn selection_in_mode(&self, ps: &PlayerState) -> bool {
        let i = (!self.pl.gun_selected).then_some(self.pl.cur_bay);
        self.usable_in_mode(ps, i)
    }

    /// Realistic avionics: the selected weapon belongs to another HUD mode.
    fn mode_msg(&mut self) {
        let want = if self.pl.gun_selected {
            None
        } else {
            self.pl.bays.get(self.pl.cur_bay).and_then(|b| self.store_mode(b.0))
        };
        self.msg(match want {
            Some(HudMode::Air) => "Select A-A mode",
            Some(_) => "Select A-G mode",
            None => "Select A-A or A-G mode",
        });
    }

    /// Space: the next weapon (with realistic avionics, the next one of the HUD's mode).
    fn next_weapon(&mut self, ps: &PlayerState) {
        // A gunship's cannon is two entries: slaved to the helmet, then to the pod.
        if self.pl.gun_selected
            && self.pl.gunship
            && !self.pl.gun_pod
            && self.usable_in_mode(ps, None)
        {
            self.pl.gun_pod = true;
            return;
        }
        // Entries: the first station of each weapon, then the cannon (index n).
        let n = self.pl.bays.len();
        // Gun pods are the cannon's entry, not stations of their own.
        let first_of = |b: &[(i32, i32)], i: usize| {
            b[i].0 >= 0 && !self.gun_pod(b[i].0) && !b[..i].iter().any(|o| o.0 == b[i].0)
        };
        let cur = if self.pl.gun_selected { n } else { self.pl.cur_bay };
        let cur_w =
            if self.pl.gun_selected { -2 } else { self.pl.bays.get(cur).map_or(-1, |b| b.0) };
        let gun = self.has_gun();
        let next = (1..=n + 1).map(|k| (cur + k) % (n + 1)).find(|&i| {
            if i == n {
                gun && self.usable_in_mode(ps, None)
            } else {
                first_of(&self.pl.bays, i)
                    && self.pl.bays[i].0 != cur_w
                    && self.usable_in_mode(ps, Some(i))
            }
        });
        match next {
            Some(i) if i == n => {
                self.pl.gun_selected = true;
                self.pl.gun_pod = false;
            }
            Some(i) => {
                self.pl.cur_bay = i;
                self.pl.gun_selected = false;
            }
            None => {}
        }
    }

    /// The selected weapon (-1 none or the cannon) and its rounds on all stations.
    pub(crate) fn selected_weapon(&self) -> (i32, i32) {
        if self.pl.gun_selected {
            return (-1, 0);
        }
        let w = self.pl.bays.get(self.pl.cur_bay).map_or(-1, |b| b.0);
        if w < 0 {
            return (-1, 0);
        }
        (w, self.pl.bays.iter().filter(|b| b.0 == w).map(|b| b.1).sum())
    }

    /// Missile table index of the selected weapon (0: none or guns).
    pub(crate) fn selected_spec(&self) -> i32 {
        match self.selected_weapon().0 {
            w if w < 0 => 0,
            w => self.weapon_types[(w as usize).min(self.weapon_types.len() - 1)].spec as i32,
        }
    }

    /// Camera keys.
    fn camera_key(&mut self, dir: i32) {
        if self.pl.pflags & pflags::CAMERA_VIEW != 0 {
            return;
        }
        self.pl.targeting.target_killed = false;
        self.pl.target_hold = false;
        self.pl.maverick_view = false;
        self.pl.cam_dir = s16(dir);
        self.pl.air_lock = -1;
        self.pl.gnd_lock = -1;
    }

    /// Alt-R: resupply (repair, reload bays and gun).
    fn resupply(&mut self) {
        self.pl.damage = 0;
        self.pl.hits = 0;
        self.pl.control_fault = ControlFault::default();
        self.pl.fire_frames = 0;
        self.pl.breakup_frames = 0;
        for b in self.pl.bays.iter_mut() {
            b.1 = if b.0 < 16 { 9 } else { 1 };
        }
        self.pl.gun_ammo = 1000;
        self.emit(SimEvent::Resupplied);
    }

    /// Per-frame systems: JAM damage, IR jammer heat, bay doors.
    pub(crate) fn update_systems(&mut self) {
        if self.pl.damage & dmg::JAM != 0 {
            self.pl.pflags &= !(pflags::ECM | pflags::IRJ);
        }
        if self.irj_on() {
            self.pl.irj_heat += 1;
            if self.pl.irj_heat > 20 * self.fs {
                self.pl.pflags &= !pflags::IRJ;
            }
        } else if self.pl.irj_heat > 0 {
            self.pl.irj_heat -= 1;
        }
        if self.pl.damage & dmg::BD != 0 {
            self.pl.bay_req = true;
            self.pl.door_anim = 2;
            self.pl.bay_close = 0;
        }
        // The RAH-66's doors shut again a while after they opened for a launch.
        if self.pl.bay_close > 0 && !self.pl.bay_pending {
            self.pl.bay_close -= 1;
            if self.pl.bay_close == 0 {
                self.pl.bay_req = false;
                self.sound(0x1F, 1);
            }
        }
        if self.pl.bay_req {
            if self.pl.door_anim == 2 {
                self.pl.pflags |= pflags::BAY_OPEN;
            } else {
                self.pl.door_anim += 1;
            }
        } else if self.pl.door_anim == 0 {
            self.pl.pflags &= !(pflags::BAY_OPEN | pflags::CAMERA_VIEW);
        } else {
            self.pl.door_anim -= 1;
        }
    }

    /// Fire handling at the end of the key handling.
    pub(crate) fn handle_fire(&mut self, ps: &PlayerState) {
        if self.pl.fire_cooldown > 0 {
            self.pl.fire_cooldown -= 1;
        }
        let bomb = ballistics::drag(self.selected_spec(), self.opts.fix_classic_bugs).is_some();
        let ccrp = ps.ccrp && self.pl.hud_mode == HudMode::Gnd && bomb;
        // The RAH-66's missile waiting for its doors goes as soon as they are open.
        if self.pl.bay_pending && self.pl.pflags & pflags::BAY_OPEN != 0 {
            self.pl.bay_pending = false;
            if self.pl.eject == 0 {
                self.fire_missile();
            }
        }
        // The rest of a ripple at its spacing while the button stays held; let go, it stops.
        if let Some(mut r) = self.pl.ripple {
            if !ps.cmd.trigger || !bomb || self.pl.eject != 0 {
                self.pl.ripple = None;
            } else if r.left > 0 {
                r.run_ft += self.ground_speed() / self.fs as f64;
                if r.run_ft >= r.spacing_ft {
                    r.run_ft -= r.spacing_ft;
                    r.left -= 1;
                    self.fire_missile();
                }
                self.pl.ripple = Some(r);
            }
        }
        let rippling = self.pl.ripple.is_some();
        let in_mode = self.selection_in_mode(ps);
        if ps.cmd.fire && self.pl.fire_cooldown == 0 && self.pl.eject == 0 && !rippling {
            if !in_mode {
                self.mode_msg();
                self.pl.fire_cooldown = 4;
            } else if !ccrp {
                if bomb {
                    self.release_bombs(ps);
                } else {
                    self.fire_missile();
                }
                self.pl.fire_cooldown = 4;
            } else if self.ccrp_solution(ps).is_none() {
                self.msg("No target designated");
            }
        }
        // CCRP: with the button held, the (first) bomb goes when the impact point reaches the
        // release point, wherever the aircraft points: lined up badly, it misses to the side.
        let along = if ccrp { self.ccrp_solution(ps).map(|s| s.0) } else { None };
        if let (Some(last), Some(now)) = (self.pl.ccrp_last, along)
            && last > 0.0
            && now <= 0.0
            && ps.cmd.trigger
            && self.pl.eject == 0
            && !rippling
            && in_mode
        {
            self.release_bombs(ps);
        }
        self.pl.ccrp_last = along;
    }

    /// Bombs a press releases (at least 1, no more than are left) and the ground distance
    /// between them (ft).
    pub(crate) fn ripple(&self, ps: &PlayerState) -> (i32, f64) {
        let n = (ps.ripple as i32).clamp(1, self.selected_weapon().1.max(1));
        if n == 1 { (1, 0.0) } else { (n, ps.ripple_ft.max(1) as f64) }
    }

    /// The first bomb of a release; once it is away the rest of the ripple follow.
    fn release_bombs(&mut self, ps: &PlayerState) {
        let (n, spacing_ft) = self.ripple(ps);
        let before = self.selected_weapon().1;
        self.fire_missile();
        if n > 1 && self.selected_weapon().1 < before {
            self.pl.ripple = Some(crate::pilot::RippleRun { left: n - 1, spacing_ft, run_ft: 0.0 });
        }
    }

    /// Ground speed (ft/s).
    fn ground_speed(&self) -> f64 {
        let v =
            ballistics::release_velocity(self.pl.p.heading, self.pl.p.velocity, self.pl.p.climb);
        v[0].hypot(v[1])
    }

    /// Where a CCRP release aims (map units): the targeting pod's ground point, else the ground
    /// lock (the lock when the pod rests on it).
    pub(crate) fn ccrp_aim_of(&self, ps: &PlayerState) -> Option<(f64, f64)> {
        let lock = usize::try_from(self.pl.gnd_lock)
            .ok()
            .and_then(|g| self.objects.get(g))
            .map(|o| (o.x, o.y));
        let pod = ps.pod.and_then(|p| p.ground);
        let (x, y) = match (pod, lock) {
            (Some(p), Some(o)) if approx(o.0 - p.0, o.1 - p.1) <= DESIGNATE_RADIUS => o,
            (Some(p), _) => p,
            (None, Some(o)) => o,
            (None, None) => return None,
        };
        Some((x as f64 + 0.5, y as f64 + 0.5))
    }

    /// CCRP release solution: the ground distance (ft) the impact point of a bomb released
    /// now still has to go to the release point along the flight path (<= 0 at and past it)
    /// and the seconds to it. A ripple's release point is half its stick short of the aim
    /// point, so the stick straddles it.
    pub(crate) fn ccrp_solution(&self, ps: &PlayerState) -> Option<(f64, f64)> {
        let (tx, ty) = self.ccrp_aim_of(ps)?;
        let (ix, iy) = self.bomb_impact()?;
        let v =
            ballistics::release_velocity(self.pl.p.heading, self.pl.p.velocity, self.pl.p.climb);
        let vh = v[0].hypot(v[1]);
        if vh < 1.0 {
            return None;
        }
        // Map units are 32 ft (y south, like the velocity).
        let (dx, dy) = ((tx - ix) * 32.0, (ty - iy) * 32.0);
        let half = {
            let (n, spacing) = self.ripple(ps);
            (n - 1) as f64 * spacing / 2.0
        };
        let along = (dx * v[0] + dy * v[1]) / vh - half;
        Some((along, along / vh))
    }

    /// Release the selected store.
    fn fire_missile(&mut self) {
        if abs16(self.pl.p.roll) > 0x3000 {
            return;
        }
        let (w, count) = self.selected_weapon();
        if count <= 0 {
            return;
        }
        let bay = self.firing_station(w);
        let wt = self.weapon_types[(w.max(0) as usize).min(self.weapon_types.len() - 1)].clone();
        let spec = wt.spec as i32;
        if spec == 0 || spec == -1 {
            return;
        }
        if self.pl.pflags & pflags::BAY_OPEN == 0 && spec != -2 {
            // The RAH-66 opens its doors and fires as they are open.
            if self.pl.comanche {
                if !self.pl.bay_req {
                    self.pl.bay_req = true;
                    self.pl.bay_close = BAY_AUTO_CLOSE * self.fs;
                    self.sound(0x1F, 1);
                }
                self.pl.bay_pending = true;
                return;
            }
            self.sound(0x20, 1);
            self.msg("Bay closed");
            return;
        }
        if self.pl.comanche && self.pl.bay_close > 0 {
            self.pl.bay_close = BAY_AUTO_CLOSE * self.fs;
        }
        if self.pl.p.alt == 0
            || (self.obj(self.pl.nearest_base).flags & oflags::CARRIER != 0
                && self.pl.p.alt == 0x80)
        {
            self.sound(0x20, 1);
            self.msg("Insufficient Altitude");
            return;
        }
        self.pl.bays[bay].1 -= 1;
        if spec == -2 {
            self.emit(SimEvent::FuelTransfer { lbs: 0x76C });
            self.msg("Fuel transfer");
            return;
        }
        self.log_event(4, bay as i32);
        self.emit(SimEvent::CancelTimeAccel);
        let Some(slot) = (8..12).rev().find(|&i| self.proj[i].ttl == 0) else { return };
        let mt = self.missile(spec).clone();
        let mut pr = self.launch_projectile(w, spec);
        let point = self.station_point(bay);
        self.drop_clear(&mut pr, bay);
        if spec == 27 {
            self.pl.maverick_view = true;
        }
        let gl = self.pl.gnd_lock;
        if spec == HELLFIRE_SPEC {
            // At the targeting pod's spot, whatever is locked: its seeker sees the spot on the
            // rail, so it pitches onto it as it leaves (with realistic guidance it flies on
            // straight from the rail first, so it can be lobbed).
            self.proj[slot] = pr;
            self.follow_spot(slot);
            pr = self.proj[slot];
            if pr.target >= 0 && pr.tgt_x >= 0 && !self.guided(spec) {
                let o = self.obj(pr.target);
                let (pos, _) = self.launch_motion();
                let to =
                    [o.x as f64 * 32.0 + 16.0 - pos[0], o.y as f64 * 32.0 + 16.0 - pos[1], -pos[2]];
                (pr.heading, pr.pitch) = crate::guidance::heading_pitch(to);
            }
        } else if gl >= 0 && (gl as usize) < self.objects.len() {
            if self.pl.targeting.lock_valid {
                let c = mt.class as i32;
                if c == 6
                    || (c == 4 && self.objects[gl as usize].unit_type != 0)
                    || (c == 5 && self.is_ship(gl))
                {
                    pr.target = gl;
                    pr.tgt_x = self.objects[gl as usize].x;
                    pr.tgt_y = self.objects[gl as usize].y;
                }
            } else {
                pr.target = -1;
            }
        }
        if self.fall_drag(spec).is_some() {
            // Free-fall and retarded bombs (and, fixed, supply pods; under realistic guidance
            // the laser-guided ones) fall ballistically. Classic flew free-fall and guided bombs
            // at half speed, turning down to a floor set by the release altitude, and dropped
            // retarded ones straight down at speed 1.
            let f = ROUND_FINE as f64;
            let v = ballistics::release_velocity(
                self.pl.p.heading,
                self.pl.p.velocity,
                self.pl.p.climb,
            );
            let mut fall = BombFall {
                x: self.pl.p.wx * ROUND_FINE,
                y: (0x100000 - self.pl.p.wy) * ROUND_FINE,
                alt: ((self.pl.p.alt as f64 - ballistics::RELEASE_DROP) * f) as i32,
                vx: (v[0] * f).round() as i32,
                vy: (v[1] * f).round() as i32,
                vz: (v[2] * f).round() as i32,
            };
            if let Some(p) = point {
                fall.x = (p[0] * f).round() as i32;
                fall.y = (p[1] * f).round() as i32;
                fall.alt = (p[2] * f).round() as i32;
            }
            pr.fall = Some(fall);
            // Falls for as long as it takes (from high up, retarded bombs take minutes).
            pr.ttl = 0x7FFF;
        }
        self.proj[slot] = pr;
        // An air-to-air missile fired with another pilot locked goes for him.
        if mt.class == 7 {
            self.proj[slot].target_pilot = self.pl.pilot_lock.filter(|&k| self.pilot_airborne(k));
        }
        if self.guided(spec) {
            // An air-to-air missile goes for the radar's target.
            let lock = self.pl.air_lock;
            if mt.class == 7 && lock >= 0 && (lock as usize) < self.flights.len() {
                self.proj[slot].target = lock;
            }
            let (pos, vel) = self.store_motion(bay);
            self.start_flight(slot, pos, vel);
            let mut m = self.proj[slot];
            self.eject(&mut m);
            self.proj[slot] = m;
            let tgt = match m.target_pilot {
                Some(k) => Some(self.pilot_target(k)),
                None => {
                    (mt.class == 7 && m.target >= 0).then(|| self.unit_target(m.target as usize))
                }
            };
            if let Some(tg) = tgt {
                let mut m = self.proj[slot];
                self.loft_launch(&mut m, tg.p);
                self.proj[slot] = m;
            }
        }
        self.pl.last_missile_slot = slot as i32;
        self.msg(format!("{} fired", wt.long_name));
        self.sound(if mt.lock_range == 0 { 0x10 } else { 0x0D }, 2);
    }

    /// Store `w` (missile table entry `spec`) leaving the aircraft now, before its target is set.
    pub(crate) fn launch_projectile(&self, w: i32, spec: i32) -> Projectile {
        let mt = self.missile(spec);
        let fs = self.fs;
        let mut ttl =
            s16(cdiv((mt.lock_range as i32 * fs) << 5, ((mt.max_speed as i32) >> 6) + 1)) + 6;
        if ttl <= 6 {
            ttl = 999;
        }
        let mut pr = Projectile {
            x: self.pl.p.mx,
            y: self.pl.p.my,
            alt: self.pl.p.view_z - 20,
            speed: self.pl.p.velocity >> 10,
            heading: self.pl.p.heading,
            pitch: self.pl.p.pitch,
            roll: self.pl.p.roll,
            ttl,
            spec,
            weapon: w,
            target: -1,
            launcher: 0,
            tgt_x: -1,
            tgt_y: -1,
            locked: false,
            fall: None,
            flight: None,
            target_pilot: None,
            station: None,
            drop: 0,
        };
        if matches!(spec, 28 | 29 | 30 | 38) {
            pr.speed >>= 1;
        }
        pr
    }

    /// The station store `w` leaves from next: the one with the most left (pairs of pylons
    /// release in turn).
    pub(crate) fn firing_station(&self, w: i32) -> usize {
        let mut bay = self.pl.cur_bay;
        for (i, b) in self.pl.bays.iter().enumerate() {
            if b.0 == w && b.1 > self.pl.bays.get(bay).map_or(-1, |c| c.1) {
                bay = i;
            }
        }
        bay
    }

    /// The aircraft keeps its stores in internal bays (the F-117A): they all drop clear.
    fn internal_bays(&self) -> bool {
        !self.pl.external_stores && !self.pl.helicopter
    }

    /// Missile `pr` leaving station `bay`: dropped clear first off a pylon (the Maverick, the
    /// Harpoon, the Penguin) or out of the bays (all of them), its motor lighting after.
    pub(crate) fn drop_clear(&self, pr: &mut Projectile, bay: usize) {
        pr.station = Some(bay as u8);
        let drop = if self.internal_bays() {
            BAY_DROP_SECS
        } else if matches!(pr.weapon, 3..=5) {
            DROP_SECS
        } else {
            0.0
        };
        if trails_smoke(pr.spec) {
            pr.drop = (drop * self.fs as f64).round() as u8;
        }
    }

    /// [`Sim::launch_motion`] of a missile off station `bay`: from where it hangs.
    pub(crate) fn store_motion(&self, bay: usize) -> ([f64; 3], [f64; 3]) {
        let (mut pos, vel) = self.launch_motion();
        if let Some(p) = self.station_point(bay) {
            pos = p;
        }
        (pos, vel)
    }

    /// Missile `pm` under realistic guidance dropping clear: pushed down by the bay's ejectors.
    pub(crate) fn eject(&self, pm: &mut Projectile) {
        if let Some(f) = pm.flight.as_mut().filter(|_| pm.drop > 0 && self.internal_bays()) {
            f.sink = (EJECT_SPEED * ROUND_FINE as f64).round() as i32;
        }
    }

    /// Where station `bay`'s stores hang now (wu: east, south, true altitude), if known.
    fn station_point(&self, bay: usize) -> Option<[f64; 3]> {
        let o = self.pl.hardpoints.get(bay)?.map(|m| m as f64 / WU_METRES);
        let p = &self.pl.p;
        let d = body_to_world(p.heading, p.pitch, p.roll, o);
        Some([p.wx as f64 + d[0], (0x100000 - p.wy) as f64 + d[1], p.alt as f64 + d[2]])
    }

    /// Where a missile under realistic guidance starts (wu: east, south, true altitude) and its
    /// velocity: off the aircraft along its nose with its speed (pull up to lob a missile).
    pub(crate) fn launch_motion(&self) -> ([f64; 3], [f64; 3]) {
        let pos =
            [self.pl.p.wx as f64, (0x100000 - self.pl.p.wy) as f64, self.pl.p.alt as f64 - 20.0];
        let vel = crate::guidance::direction(self.pl.p.heading, self.pl.p.pitch)
            .map(|d| d * self.pl.p.velocity as f64 / 16.0);
        (pos, vel)
    }

    /// Release a countermeasure `kind`: 1 flare, 2 chaff, 3 decoy.
    pub(crate) fn release_cm(&mut self, kind: usize) {
        let count = self.pl.stores[kind];
        self.pl.stores[kind] -= 1;
        if count <= 0 {
            self.pl.stores[kind] = 0;
            self.msg("Stores exhausted");
            return;
        }
        let slot = if kind == 3 {
            (self.pl.cms[0].ttl == 0 && self.pl.damage & dmg::JAM == 0).then_some(0)
        } else {
            (1..4).rev().find(|&k| self.pl.cms[k].ttl == 0)
        };
        if let Some(k) = slot {
            let mut ttl = (9 - 2 * self.diff) * self.fs;
            let name = match kind {
                1 => {
                    self.pl.flare_lamp = 3 * self.fs;
                    "Flare"
                }
                2 => {
                    self.pl.chaff_lamp = 3 * self.fs;
                    "Chaff"
                }
                _ => {
                    ttl <<= 3;
                    "Decoy"
                }
            };
            self.pl.cms[k] =
                Countermeasure { x: self.pl.p.mx, y: self.pl.p.my, kind: kind as i32, ttl };
            self.msg(format!("{name} released"));
        }
        self.sound(15, 2);
    }

    /// Velocity of a player round leaving the cannon: the muzzle speed plus the aircraft's
    /// speed along the nose (fine units per frame: east, south, up).
    pub fn gun_round_velocity(heading: i32, pitch: i32, knots: i32, fs: i32) -> (f64, f64, f64) {
        let bam = std::f64::consts::TAU / 65536.0;
        let (h, p) = ((heading as u16) as f64 * bam, (pitch as i16) as f64 * bam);
        let v = (GUN_MUZZLE_SPEED + knots as f64 * 0.514_444) / WU_METRES * ROUND_FINE as f64
            / fs as f64;
        (v * h.sin() * p.cos(), -v * h.cos() * p.cos(), v * p.sin())
    }

    /// Gravity on a player round (fine units per frame, each frame).
    pub fn gun_round_gravity(fs: i32) -> i32 {
        (GUN_GRAVITY / WU_METRES * ROUND_FINE as f64 / (fs * fs) as f64).round() as i32
    }

    /// A round leaving the cannon of an aircraft at `p`, one frame out: along the nose, or
    /// along the turret's `aim` (BAM heading, pitch) with the aircraft's own speed added.
    pub fn gun_round(p: &PlayerView, aim: Option<(i32, i32)>, fs: i32) -> Bullet {
        let f = ROUND_FINE;
        let (vx, vy, vz) = match aim {
            None => Self::gun_round_velocity(p.heading, p.pitch, p.knots, fs),
            Some((h, pitch)) => {
                let (mx, my, mz) = Self::gun_round_velocity(h, pitch, 0, fs);
                let (ax, ay, az) = Self::gun_round_velocity(p.heading, p.pitch, p.knots, fs);
                let (bx, by, bz) = Self::gun_round_velocity(p.heading, p.pitch, 0, fs);
                (mx + ax - bx, my + ay - by, mz + az - bz)
            }
        };
        let (vx, vy, vz) = (vx.round() as i32, vy.round() as i32, vz.round() as i32);
        let (x, y) = (p.wx * f, (0x100000 - p.wy) * f);
        Bullet { x: x + vx, y: y + vy, alt: (p.view_z - 2) * f + vz, vx, vy, vz }
    }

    /// Where the active pilot's turret points: an attack helicopter's follows the helmet sight
    /// or the targeting pod (`gun_pod`) as far as it turns (None: a fixed cannon, or no sight,
    /// along the nose).
    pub(crate) fn turret_aim(&self, ps: &PlayerState) -> Option<(i32, i32)> {
        let sight = if self.pl.gun_pod { ps.pod } else { ps.helmet.or(ps.pod) };
        let pod = sight.filter(|_| self.pl.gunship)?;
        let p = &self.pl.p;
        let az = s16(pod.heading as i32 - p.heading).clamp(-TURRET_AZIMUTH, TURRET_AZIMUTH);
        let el = s16(pod.pitch as i32 - p.pitch).clamp(-TURRET_DEPRESSION, TURRET_ELEVATION);
        Some((s16(p.heading + az), s16(p.pitch + el)))
    }

    /// The active pilot's cannon and gun rounds: move, then the hit tests (his kills).
    pub(crate) fn update_own_rounds(&mut self, ps: &PlayerState) {
        let fs = self.fs;
        let nb = self.n_bullets;
        // A round every frame while the trigger is held (classic mode fired every other frame);
        // each slot is reused after `nb` frames, which ends the round there.
        if self.has_gun() {
            let slot = (self.frame as usize) % nb;
            let trigger = ps.cmd.gun || (self.pl.gun_selected && ps.cmd.trigger);
            // Realistic avionics: the cannon fires in AIR and GND only.
            let in_mode = self.usable_in_mode(ps, None);
            if trigger && !in_mode && !self.pl.gun_blocked && self.pl.eject == 0 {
                self.msg("Select A-A or A-G mode");
            }
            self.pl.gun_blocked = trigger && !in_mode;
            if trigger && in_mode && self.pl.gun_ammo > 0 && self.pl.eject == 0 {
                self.pl.gun_ammo = clamp_v(self.pl.gun_ammo + cdiv(-20, fs), 0, 1000);
                self.sound(6, 2);
                let aim = self.turret_aim(ps);
                self.bullets[slot] = Self::gun_round(&self.pl.p, aim, fs);
                self.pl.gun_aim = aim;
                self.pl.gun_firing = true;
                self.pl.fired_step = self.steps;
            } else {
                self.bullets[slot].x = 0;
                self.pl.gun_firing = false;
            }
        }
        let g = Self::gun_round_gravity(fs);
        for b in self.bullets[..nb].iter_mut() {
            if b.x != 0 {
                b.x += b.vx;
                b.y += b.vy;
                b.alt += b.vz;
                b.vz -= g;
            }
        }
        // A coop client's rounds hit on his side (he reports the hits): here they only fly.
        if self.pl.client_rounds {
            let hills = self.hills();
            for b in self.bullets[..nb].iter_mut().filter(|b| b.alt < b.ground(&hills)) {
                b.x = 0;
            }
            return;
        }
        // Hit tests (all rounds, not only those that project on screen). A player round
        // hits what passes within the hit radius of its path over this frame.
        self.credit = Some(self.pl.id);
        for i in 0..nb {
            let b = self.bullets[i];
            if b.x == 0 {
                continue;
            }
            for j in self.round_hits(&b) {
                self.gun_kill(j);
                self.bullets[i].x = 0;
            }
            if self.bullets[i].x != 0
                && let Some(k) = self.round_hits_pilots(&b).first().copied()
            {
                self.gun_hit_pilot(k, Self::round_point(&self.pilot_ref(k as usize).p));
                self.bullets[i].x = 0;
            }
            self.round_on_ground(i);
        }
        self.credit = None;
    }

    /// Gun hit radius (world units).
    pub(crate) fn gun_hit_radius(&self) -> f64 {
        GUN_HIT_RADIUS[self.diff.clamp(0, 3) as usize] / WU_METRES
    }

    /// The airborne flight units a round's path over this frame passes within the hit radius
    /// of.
    pub(crate) fn round_hits(&self, b: &Bullet) -> Vec<usize> {
        (0..self.flights.len())
            .filter(|&j| {
                let u = self.flights[j];
                // A falling wreck is not shot down again.
                u.flags & fflags::AIRBORNE != 0
                    && u.flags & fflags::DESTROYED == 0
                    && self.round_passes(b, (u.wx, u.wy, u.alt))
            })
            .collect()
    }

    /// The other pilots in the air a round's path over this frame passes within the hit
    /// radius of.
    pub(crate) fn round_hits_pilots(&self, b: &Bullet) -> Vec<u8> {
        self.other_pilots()
            .into_iter()
            .filter(|(_, p)| self.round_passes(b, Self::round_point(p)))
            .map(|(k, _)| k)
            .collect()
    }

    /// A pilot's aircraft in the rounds' frame (world units, y growing south, viewZ).
    pub(crate) fn round_point(p: &PlayerView) -> (i32, i32, i32) {
        (p.wx, 0x100000 - p.wy, p.view_z)
    }

    /// A round's path over this frame passes within the hit radius of point `at` (world
    /// units).
    fn round_passes(&self, b: &Bullet, at: (i32, i32, i32)) -> bool {
        let r = self.gun_hit_radius();
        let fine = ROUND_FINE as f64;
        let end = [b.x as f64 / fine, b.y as f64 / fine, b.alt as f64 / fine];
        let step = [b.vx as f64 / fine, b.vy as f64 / fine, b.vz as f64 / fine];
        let len2 = step.iter().map(|s| s * s).sum::<f64>().max(1e-9);
        let to = [at.0 as f64 - end[0], at.1 as f64 - end[1], at.2 as f64 - end[2]];
        // Closest point of the segment end - step .. end.
        let k = (-(to[0] * step[0] + to[1] * step[1] + to[2] * step[2]) / len2).clamp(0.0, 1.0);
        let d2: f64 = (0..3).map(|a| (to[a] + k * step[a]).powi(2)).sum();
        d2 < r * r
    }

    /// The active pilot's gunfire hits pilot `k` (at `at`, world units as the rounds'): a
    /// spark for everybody, and (with friendly fire, at most every quarter second) damage on
    /// his turn.
    pub(crate) fn gun_hit_pilot(&mut self, k: u8, at: (i32, i32, i32)) {
        let (x, y) = (at.0 / 32, at.1 / 32);
        self.emit(SimEvent::Explosion { x, y, alt: at.2, kind: -1 });
        let (me, now) = (self.pl.id, self.steps);
        let every = (self.fs / 4).max(1) as u32;
        self.gun_damage_steps.retain(|&(_, _, s)| now.wrapping_sub(s) < every);
        if !self.gun_damage_steps.iter().any(|&(a, b, _)| (a, b) == (me, k)) {
            self.gun_damage_steps.push((me, k, now));
            self.pilot_hits.push((k, None));
        }
    }

    /// Flight unit `j` shot down by the active pilot's cannon.
    pub(crate) fn gun_kill(&mut self, j: usize) {
        let u = self.flights[j];
        self.flights[j].flags |= fflags::HIT;
        self.destroy_aircraft(j);
        let n = self.ac_type(u.spec).name.clone();
        self.msg(format!("{n} destroyed by gunfire"));
        self.flash = Flash { x: u.x, y: u.y, alt: u.alt, timer: 0x10 };
        self.emit(SimEvent::Explosion { x: u.x, y: u.y, alt: u.alt, kind: 0x10 });
    }

    /// The enemy aircraft's gun rounds: move, then the hit tests against every pilot.
    pub(crate) fn update_enemy_rounds(&mut self, present: &[usize]) {
        let nb = self.n_bullets;
        let f = ROUND_FINE;
        for b in self.bullets[nb..].iter_mut() {
            if b.x != 0 {
                b.x += b.vx;
                b.y += b.vy;
                b.alt += b.vz;
            }
        }
        for i in nb..self.bullets.len() {
            if self.bullets[i].x == 0 {
                continue;
            }
            let mut hit = false;
            for &k in present {
                if present.len() > 1 {
                    self.activate(k);
                }
                // The rounds fly on past a pilot who ejected.
                if self.pl.eject != 0 {
                    continue;
                }
                let b = self.bullets[i];
                let (bx, by) = b.map_xy();
                let d = (s16(bx - self.pl.p.mx).abs()
                    + s16(by - self.pl.p.my).abs()
                    + (s16(b.alt / f - self.pl.p.view_z).abs() >> 5))
                    .abs();
                if d < 0x20 {
                    self.msg("Hit by gunfire");
                    if d < 0x20 / (4 - self.diff) {
                        self.player_hit(false);
                    }
                    let (bx, by) = b.map_xy();
                    self.emit(SimEvent::Explosion { x: bx, y: by, alt: b.alt / f, kind: -1 });
                    self.bullets[i].x = 0;
                    hit = true;
                    break;
                }
            }
            if !hit {
                self.round_on_ground(i);
            }
        }
    }

    /// Gun round `i` below the ground: its impact, and what it destroys there.
    fn round_on_ground(&mut self, i: usize) {
        let b = self.bullets[i];
        if let Some((x, y)) = b.ground_crossing(&self.hills()) {
            self.bullets[i].x = 0;
            self.round_impact(x, y, b.map_xy());
        }
    }

    /// A round of the active pilot's hit the ground at (x, y) (fine units), at map point `at`
    /// below it: dust, and what stands there may go.
    pub(crate) fn round_impact(&mut self, x: i32, y: i32, (bx, by): (i32, i32)) {
        {
            let heavy = self.pl.a10 || self.pl.tornado || self.pl.gunship;
            self.emit(SimEvent::RoundImpact { x, y, heavy });
            let o = self.find_object_at(bx, by);
            if o == -1 || self.obj(o).flags & oflags::DESTROYED != 0 {
                return;
            }
            let t = self.pl.last_tile;
            if !self.tile_is_intact(&t) {
                return;
            }
            let (tx, ty) = t.map_or((self.obj(o).x, self.obj(o).y), |t| t.map_pos());
            let d = approx(tx - bx, ty - by);
            if self.compat(18, o) * 6 / (self.diff + 2) > d {
                self.destroy_ground(o);
                let n = self.place_name(o as usize);
                self.msg(format!("{n} destroyed by gunfire"));
                self.emit(SimEvent::Explosion { x: tx, y: ty, alt: 0, kind: 0x10 });
            }
        }
    }
}

/// Missiles light a motor (bombs, pods and tanks only fall).
fn trails_smoke(spec: i32) -> bool {
    !matches!(spec, 28 | 29 | 30 | 38)
}

/// Offset `o` (right, forward, up) of an aircraft at `heading`, `pitch`, `roll` (BAM; roll
/// right wing down) in the world (east, south, up).
pub(crate) fn body_to_world(heading: i32, pitch: i32, roll: i32, o: [f64; 3]) -> [f64; 3] {
    let bam = 65536.0 / std::f64::consts::TAU;
    let (h, p, r) =
        ((heading as u16) as f64 / bam, s16(pitch) as f64 / bam, s16(roll) as f64 / bam);
    let fwd = crate::guidance::direction(heading, pitch);
    let right0 = [h.cos(), h.sin(), 0.0];
    let up0 = [-p.sin() * h.sin(), p.sin() * h.cos(), p.cos()];
    let (c, sn) = (r.cos(), r.sin());
    let right: [f64; 3] = std::array::from_fn(|i| c * right0[i] - sn * up0[i]);
    let up: [f64; 3] = std::array::from_fn(|i| c * up0[i] + sn * right0[i]);
    std::array::from_fn(|i| right[i] * o[0] + fwd[i] * o[1] + up[i] * o[2])
}
