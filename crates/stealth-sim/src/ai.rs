use crate::Sim;
use crate::ai_flight::Steer;
use crate::math::{abs16, approx, bearing, cdiv, clamp_q, s16};
use crate::types::*;
use stealth_i18n::text;

/// How high over the mountains the AI's aircraft keep (altitude units, 300 m).
const TERRAIN_CLEARANCE: i32 = 614;
/// The room kept over the rolling hills ahead (altitude units, about 60 m).
const HILLS_CLEARANCE: i32 = 123;
/// Flying straight out of a circle round its approach point, it climbs to this much over the
/// ground ahead (altitude units, about 200 m).
const EXTEND_ROOM: i32 = 400;
/// On final a campaign aircraft keeps this much over the ground ahead (altitude units, 75 m)
/// until this close to the runway (map units, 2 km).
const FINAL_ROOM: i32 = 150;
const FINAL_FLAT: i32 = 0x80;
/// Its climb then (about 8 degrees).
const FINAL_CLIMB: i32 = 0x600;
/// A campaign aircraft chasing another leads it by a second for this many world units of
/// range (about 500 m).
const PURSUIT_LEAD: f32 = 1000.0;
/// Base traffic: its oval circuit around the base (map units: half its width east-west, half
/// its length along the runway; 4 x 6 km), the height it flies it at (altitude units, 730 m),
/// how long it flies it (seconds, about two laps) and rests on the ground before the next
/// (seconds).
const CIRCUIT: (i32, i32) = (0x100, 0x180);
const CIRCUIT_ALT: i32 = 1500;
const CIRCUIT_S: i32 = 520;
const TRAFFIC_REST_S: i32 = 180;

impl Sim {
    pub(crate) fn update_aircraft(&mut self, present: &[usize]) {
        for &k in present {
            self.activate(k);
            self.pl.threat_count_last = self.pl.threat_count;
            self.pl.threat_count = 0;
        }
        let eslot = self.n_bullets + ((self.frame & 0xC) >> 2) as usize;
        if let Some(b) = self.bullets.get_mut(eslot) {
            *b = Bullet::default();
        }
        let nf = self.flights.len();
        self.ai_cms_age();
        self.flight_vel.clear();
        self.flight_vel.resize(nf, [0.0; 3]);
        for i in 0..nf {
            if self.flights[i].flags & fflags::EXISTS == 0 {
                self.flights[i].mode = 0;
                continue;
            }
            let kind = self.ac_type(self.flights[i].spec).kind;
            if self.flights[i].flags & fflags::AIRBORNE == 0 || self.flights[i].speed == 0 {
                self.flights[i].mode = 0;
                self.scramble(i);
                continue;
            }
            if present.len() > 1 {
                let k = self.flight_target(i);
                self.activate(k as usize);
            }
            let alive = self.flights[i].flags & fflags::DESTROYED == 0;
            let was = self.flights[i];
            match kind {
                crate::AircraftKind::Drone if alive && self.uav_of_flight(i).is_some() => {
                    self.fly_uav(i)
                }
                crate::AircraftKind::Drone if alive => self.fly_drone(i),
                crate::AircraftKind::Helicopter if alive && self.supply_run(i).is_some() => {
                    self.fly_supply(i)
                }
                crate::AircraftKind::Helicopter if alive => self.fly_helo(i),
                _ => self.fly(i),
            }
            let u = self.flights[i];
            let fs = self.fs.max(1) as f32;
            self.flight_vel[i] = [
                u.wx.wrapping_sub(was.wx) as f32 * fs,
                u.wy.wrapping_sub(was.wy) as f32 * fs,
                (u.alt - was.alt) as f32 * fs,
            ];
        }
    }

    /// The pilot flight unit `i` goes for: the one its radar holds, else the one the alarm is
    /// about, else the nearest.
    fn flight_target(&mut self, i: usize) -> u8 {
        if self.flight_pilot.len() < self.flights.len() {
            self.flight_pilot.resize(self.flights.len(), 0);
        }
        let u = self.flights[i];
        let cur = self.flight_pilot[i];
        let k = if u.flags & fflags::CONTACT == 0 && self.alarm_timer != 0 {
            // The nearest of the pilots the defences reported (each where he was seen), else
            // the one the alarm is about.
            let seen = self
                .present()
                .into_iter()
                .filter_map(|k| self.pilot_ref(k).seen.map(|(x, y, _)| (k, x, y)))
                .min_by_key(|&(k, x, y)| (approx(s16(x - u.x), s16(y - u.y)), k))
                .map(|(k, _, _)| k as u8);
            seen.unwrap_or_else(|| self.nearest_pilot(self.alarm_pilot, true, u.x, u.y))
        } else {
            self.nearest_pilot(cur, u.flags & fflags::CONTACT != 0, u.x, u.y)
        };
        self.flight_pilot[i] = k;
        k
    }

    /// The defences have reported the player (or a hit) at least once: `ref_*` hold where.
    fn ref_known(&self) -> bool {
        (self.ref_x, self.ref_y) != (0, 0)
    }

    fn fly(&mut self, i: usize) {
        let fs = self.fs;
        let nf = self.flights.len();
        let n = self.objects.len() as i32;
        let t = self.ac_type(self.flights[i].spec).clone();
        let (px, py, pz) = (self.pl.p.mx, self.pl.p.my, self.pl.p.view_z);
        let mut u = self.flights[i];
        // --- target selection ---
        let (tx, ty, tz, mode);
        let mut chase = None;
        // A campaign fighter's quarry nearer than the pilot the defences reported: it stays on it.
        let quarry_nearer = self.campaign_pursuit(i).is_some_and(|(x, y, _)| {
            let (sx, sy) = self.pl.seen.map_or((self.ref_x, self.ref_y), |(x, y, _)| (x, y));
            approx(s16(x - u.x), s16(y - u.y)) < approx(s16(sx - u.x), s16(sy - u.y))
        });
        // A strike aircraft pressed on its way home fights first.
        let pressed = self.campaign_defending(i) && u.flags & fflags::FINAL == 0;
        if u.flags & fflags::RTB != 0 && !pressed {
            let b = self.obj(u.wp);
            if u.flags & fflags::FINAL != 0 {
                // A campaign aircraft flies straight for the runway (the classic point beyond
                // it moves with the aircraft, which a fast jet that turns wide circles for
                // ever).
                let campaign = self.campaign_flight(i).is_some();
                tx = if campaign { b.x } else { s16(b.x - 2 * s16(u.x - b.x)) };
                ty = b.y;
                let glide = if (abs16(u.x - b.x) + b.flags) & 0x200 != 0 { 140 } else { 12 };
                // It keeps clear of the hills ahead on the way in, down to the runway's flat
                // ground.
                tz = if campaign && approx(s16(u.x - b.x), s16(u.y - b.y)) > FINAL_FLAT {
                    glide.max(self.ground_ahead(&u) + FINAL_ROOM)
                } else {
                    glide
                };
            } else {
                tx = b.x;
                ty = s16(b.y + self.ns * 0x280);
                tz = approx(u.x - tx, u.y - ty) + 2000;
            }
            mode = 2;
        } else if self.is_traffic(i) {
            // Base traffic: clockwise round its circuit, for a point an eighth of the way ahead
            // (the oval stretched to a circle).
            let b = self.obj(u.wp);
            let east = clamp_q(u.x - b.x, -0x2000, 0x2000) * CIRCUIT.1 / CIRCUIT.0;
            let north = clamp_q(b.y - u.y, -0x3000, 0x3000);
            let a = s16(bearing(east, north) + 0x2000);
            tx = b.x + self.trig.sin_mul(a, CIRCUIT.0);
            ty = b.y - self.trig.cos_mul(a, CIRCUIT.1);
            tz = CIRCUIT_ALT;
            mode = 3;
        } else if self.alarm_timer != 0
            && (u.flags & 0x140 == 0 || self.alarm_timer > self.alarm_recent())
            // A friendly fighter (a campaign CAP) never goes for the player, nor does a
            // campaign helicopter or strike aircraft.
            && !self
                .campaign_flight(i)
                .is_some_and(|f| f.blue || f.role != crate::campaign::Role::Fighter)
            && !quarry_nearer
        {
            mode = 1;
            if self.decoy_active() {
                tx = self.pl.cms[0].x;
                ty = self.pl.cms[0].y;
                tz = clamp_q(pz, 1000, 30000);
            } else if let Some((x, y, z)) = self.pl.seen.filter(|_| self.present().len() > 1) {
                // More pilots in the world: the one it goes for, where he was seen.
                tx = x;
                ty = y;
                tz = z;
            } else {
                tx = self.ref_x;
                ty = self.ref_y;
                tz = self.ref_alt;
            }
        } else if let Some(j) = self.campaign_pursued(i) {
            // Campaign: going for an aircraft of the other side, for where it will be (lead
            // pursuit: more the farther it is, up to a few seconds' flight).
            mode = 4;
            let e = self.flights[j];
            let v = self.flight_vel.get(j).copied().unwrap_or_default();
            let d = approx(s16(e.x - u.x), s16(e.y - u.y)) as f32;
            let lead = (d * 32.0 / PURSUIT_LEAD).clamp(0.3, 3.0);
            tx = e.x + (v[0] * lead / 32.0) as i32;
            ty = e.y + (v[1] * lead / 32.0) as i32;
            tz = e.alt;
            chase = Some(v[0].hypot(v[1]) as f64);
        } else {
            mode = 3;
            let (mut x, mut y, mut z);
            if let Some((gx, gy)) = self.campaign_goal(i, &u) {
                x = gx;
                y = gy;
                z = if self.campaign_striking(i) {
                    crate::campaign::STRIKE_ALT
                } else {
                    // Well over the mountains ahead (a heavy aircraft climbs slowly).
                    clamp_q(u.alt, 6000, 16000).max(self.ground_ahead(&u) + 2 * TERRAIN_CLEARANCE)
                };
            } else if u.flags & fflags::ESCORT != 0
                && self.escort_leader >= 0
                && (self.escort_leader as usize) < nf
            {
                let l = self.flights[self.escort_leader as usize];
                let a = s16((i as i32 & 7) * 0x800 + l.heading - 0x1800);
                x = s16(l.x + self.trig.sin_mul(a, l.speed));
                y = s16(l.y - self.trig.cos_mul(a, l.speed));
                z = l.alt + (i as i32 & 7) * 0x40;
            } else {
                if ((i as i32 * 8 + self.mission_tick) & 0xBF) == 0
                    && u.flags & fflags::ALERTED_PATROL == 0
                    && n > 1
                {
                    // A new station: of a few enemy places, the one nearest where the
                    // defences last reported the player, else nearest the flight (it stays
                    // on its own patch of sky). The pilots know nothing of a player nobody
                    // has seen; the better they are, the more places they weigh.
                    let (ax, ay) =
                        if self.ref_known() { (self.ref_x, self.ref_y) } else { (u.x, u.y) };
                    let mut best = i32::MAX;
                    for _ in 0..2 + 2 * self.diff {
                        let tt = (self.rng.random(n) + 1).min(n - 1);
                        let o = self.objects[tt as usize];
                        if o.flags & oflags::FRIENDLY != 0 {
                            continue;
                        }
                        let d = approx(o.x - ax, o.y - ay);
                        if d < best {
                            best = d;
                            u.wp = tt;
                        }
                    }
                    if (approx(px - u.x, py - u.y) & 0xFFC0) > 0x5780 && i != 0 {
                        u.flags &= 0x1C1;
                        u.timer = 0;
                        u.mode = 0;
                        self.flights[i] = u;
                        return;
                    }
                }
                let o = self.obj(u.wp);
                x = o.x;
                y = o.y;
                z = if self.ref_known() {
                    clamp_q(self.ref_alt + 1000, 5000, 20000)
                } else {
                    clamp_q(u.alt, 6000, 16000)
                };
            }
            if u.flags & fflags::CONTACT != 0 {
                x = px;
                y = py;
                z = u.alt;
            }
            tx = x;
            ty = y;
            tz = z;
        }
        u.mode = mode;
        let dx = s16(tx - u.x);
        let dy = s16(ty - u.y);
        let brg = bearing(dx, s16(-dy));
        let rng = approx(dx, dy);
        let mut pitch_cmd = clamp_q(bearing(s16(tz - u.alt) >> 5, rng), -0x2000, 0x1000);
        // A campaign aircraft on final under the ground ahead (it turned in low over the
        // hills): it climbs clear first (the glide's pitch is a shallow one so far out).
        if u.flags & fflags::FINAL != 0
            && self.campaign_flight(i).is_some()
            && approx(s16(u.x - tx), s16(u.y - ty)) > FINAL_FLAT
            && u.alt < self.ground_ahead(&u) + FINAL_ROOM
        {
            pitch_cmd = pitch_cmd.max(FINAL_CLIMB);
        }
        // --- steering ---
        let mut roll_cmd;
        let mut circle = false;
        let dogfight = mode == 1 && rng < 0x400;
        if dogfight {
            self.pl.threat_count += 1;
            if rng < 0x200
                && self.pl.eject == 0
                && self.frame & 3 == 0
                && self.alarm_timer > self.alarm_recent()
                && abs16(s16(u.heading - brg)) < 0x400
                && abs16(s16(u.pitch - pitch_cmd)) < 0x400
                && self.sees((u.x, u.y, u.alt), (self.pl.p.mx, self.pl.p.my, self.pl.p.view_z))
            {
                self.fire_gun(&u);
            }
            let ag = if i == 0 { 1 } else { clamp_q((i as i32 & 3) + self.diff, 0, 2) };
            let h = s16(u.heading + if abs16(u.bank) < 0x4000 { u.bank >> 2 } else { 0 });
            let rel = (s16(brg - h) >> 13) & 7;
            let ph = s16(self.pl.p.heading
                + if abs16(self.pl.p.roll) < 0x4000 { self.pl.p.roll >> 1 } else { 0 });
            let asp = ((s16(u.heading - ph) >> 13) + 4) & 7;
            let v =
                self.maneuver.get((ag * 64 + asp * 8 + rel) as usize).copied().unwrap_or(0) as i32;
            roll_cmd = v << 12;
            if v == 0x100 {
                pitch_cmd = 0x6000;
                roll_cmd = if self.frame & 0x800 != 0 { 0x4000 } else { -0x4000 };
            }
            if v == 0x200 {
                pitch_cmd = s16(0xA000);
                roll_cmd = if self.frame & 0x800 != 0 { -0x4000 } else { 0x4000 };
            }
            if pitch_cmd == s16(0xA000) && u.alt < 3000 - (u.pitch >> 3) {
                pitch_cmd = s16(u.pitch + 0x1000);
            }
            if abs16(u.bank) > 0x4000 {
                roll_cmd = 0;
                pitch_cmd = 0;
            }
        } else {
            roll_cmd = clamp_q(s16(brg - u.heading), -0x2000, 0x2000) * 2;
            if mode == 1 && 2 * self.diff < self.pl.threat_count_last {
                roll_cmd = 0x3000;
                circle = true;
            }
            // On its way in (on final too), the point inside its turn circle and well off the
            // nose (a fast jet at the approach point at an angle to the runway): it would circle
            // it for ever, low. It flies on straight, climbing clear of the ground ahead,
            // until it can turn onto it.
            if mode == 2 && !self.ai_physics(i, &u) && self.campaign_flight(i).is_some() {
                use crate::ai_flight::{inside_turn, rad};
                let rate = (t.agility as f64 * 512.0).max(1.0) * std::f64::consts::PI / 32768.0;
                let radius = u.speed.max(1) as f64 / rate;
                let to = (s16(tx - u.x) as f64 * 32.0, s16(ty - u.y) as f64 * 32.0);
                if inside_turn(to, rad(u.heading), rad(s16(brg - u.heading)), radius) {
                    roll_cmd = 0;
                    if u.alt < self.ground_ahead(&u) + EXTEND_ROOM {
                        pitch_cmd = pitch_cmd.max(0x800);
                    }
                }
            }
        }
        if mode == 1
            && self.obj(self.pl.nearest_base).flags & oflags::FRIENDLY != 0
            && self.pl.nearest_base_dist < 0x780
        {
            roll_cmd = 0x3000;
            circle = true;
        }
        let noe = self.noe(i, &u);
        if self.ai_physics(i, &u) {
            self.terrain_flag(i, &mut u);
            let st =
                Steer { tx, ty, tz, mode, rng, dogfight, roll_cmd, pitch_cmd, circle, noe, chase };
            self.fly_physics(i, &mut u, &t, st);
            u.flags &= !fflags::HIT;
        } else {
            if self.opts.realistic_ai_flight {
                self.ai_classic(i, &u);
            }
            if noe {
                // Nap of the earth: the terrain following's climb angle, the track curving
                // with the bank (the heading turns by an eighth of it a second).
                let rad = crate::ai_flight::rad;
                let pos = [u.wx as f64, u.wy as f64, u.alt as f64];
                let turn = rad(u.bank) / 8.0;
                let to_go = rad(s16(brg - u.heading));
                let v = u.speed.max(1) as f64;
                let clearance = crate::terrain_follow::NOE_CLEARANCE;
                let gamma = self.noe_gamma(pos, rad(u.heading), v, turn, to_go, clearance);
                pitch_cmd = crate::ai_flight::bam(gamma);
            }
            self.classic_moves(i, &mut u, &t, roll_cmd, pitch_cmd, mode, noe);
        }
        let ground = self.hill_alt(u.x, u.y);
        if u.alt < ground {
            u.flags &= if i != 0 { 0x1C1 } else { 0 };
            self.flash = Flash { x: u.x, y: u.y, alt: ground, timer: -16 };
            self.emit(SimEvent::Explosion { x: u.x, y: u.y, alt: ground, kind: -16 });
            self.emit(SimEvent::AircraftCrashed { unit: i });
            if i as i32 == self.pl.air_lock {
                self.pl.air_lock = -1;
            }
        }
        if rng < 0x10 && mode == 2 {
            let base = self.place_name(u.wp.max(0) as usize);
            let ty = t.name.clone();
            let msg = if u.flags & fflags::FINAL != 0 {
                let first = u.flags & fflags::ROLLOUT == 0;
                u.flags |= fflags::ROLLOUT;
                first.then(|| text!("sim-landing", base = base.as_str(), aircraft = ty.as_str()))
            } else {
                u.flags |= fflags::FINAL;
                Some(text!("sim-on-final", base = base.as_str(), aircraft = ty.as_str()))
            };
            if let Some(m) = msg
                && !self.is_traffic(i)
            {
                self.msg(m);
            }
        }
        let mut stopped = false;
        if u.flags & fflags::ROLLOUT != 0 {
            u.bank = 0;
            u.pitch = 0;
            u.heading = if self.ns == 1 { 0 } else { s16(0x8000) };
            u.alt =
                if self.obj(self.pl.nearest_base).flags & oflags::CARRIER != 0 { 140 } else { 12 };
            if u.speed > 0 {
                u.speed -= cdiv(60, fs);
            } else {
                u.flags &= 0x1C1;
                stopped = true;
                if i == 0 && (5..=8).contains(&self.pl.targets[0].mission_type) {
                    u.flags = 0;
                    self.pl.target_escaped = true;
                }
            }
            // Base traffic down: off the runway until its next circuit (`spawn_traffic`).
            if self.is_traffic(i) && u.speed < 100 {
                u.flags = 0;
                u.timer = (TRAFFIC_REST_S * fs).min(0x7FFF);
                self.flights[i] = u;
                return;
            }
        }
        u.timer = s16(u.timer - 1);
        if u.timer == 0 && self.is_traffic(i) {
            // Base traffic: its circuits flown, it lands.
            u.flags |= fflags::RTB;
        } else if u.timer == 0 {
            u.flags |= fflags::RTB;
            let mut best = (u.wp, 0x7FFF);
            let friendly = self.campaign_flight(i).is_some_and(|f| f.blue);
            for (k, o) in self.objects.iter().enumerate().take(self.woc).skip(3) {
                // An airfield of its side (a campaign may have taken one).
                if o.flags & 0x101 == 1 && (o.flags & oflags::FRIENDLY != 0) == friendly {
                    let d = approx(o.x - u.x, o.y - u.y);
                    if d < best.1 {
                        best = (k as i32, d);
                    }
                }
            }
            u.wp = best.0;
        }
        self.flights[i] = u;
        if stopped && u.flags & (fflags::EXISTS | fflags::AIRBORNE) == fflags::EXISTS {
            self.park_landed(i);
        }
        let per = (fs << 4).max(1);
        let phase = ((((i as i32) & 8) >> 3) + (i as i32 & 7) * 2) * fs;
        if self.frame % per == phase
            && u.flags & fflags::DESTROYED == 0
            && u.flags & fflags::AIRBORNE != 0
        {
            self.radar_sweep(i);
        }
    }

    /// The classic moves of flight unit `i` for the steering commands.
    #[allow(clippy::too_many_arguments)]
    fn classic_moves(
        &mut self,
        i: usize,
        u: &mut FlightUnit,
        t: &crate::AircraftType,
        mut roll_cmd: i32,
        mut pitch_cmd: i32,
        mode: i32,
        noe: bool,
    ) {
        let fs = self.fs;
        let agil = t.agility as i32;
        roll_cmd = clamp_q(roll_cmd, -agil << 12, agil << 12);
        roll_cmd = clamp_q(s16(roll_cmd - u.bank), -agil << 8, agil << 8);
        if u.flags & fflags::TAKEOFF != 0 {
            if u.speed < 150 {
                u.pitch = 0;
            } else {
                u.pitch = s16(u.pitch + 0x100);
            }
            roll_cmd = 0;
            if u.speed < t.max_speed as i32 {
                u.speed += cdiv(40, fs);
            } else if u.alt > 300 {
                u.flags &= !fflags::TAKEOFF;
            }
        }
        if u.flags & 0x30 != 0 {
            roll_cmd = 0x400;
        }
        self.terrain_flag(i, u);
        if u.flags & fflags::TERRAIN != 0 && !noe {
            pitch_cmd = 0x3000;
        }
        // --- integration ---
        u.bank = s16(u.bank + cdiv(roll_cmd * 4, fs));
        u.heading = s16(u.heading + cdiv(u.bank >> 3, fs));
        let mut dp = s16(pitch_cmd - u.pitch);
        if u.flags & fflags::DESTROYED != 0 {
            dp = -0x200;
            if self.frame & 1 == 0 {
                let slot = ((self.frame & 0x1E) >> 1) as usize;
                self.add_smoke(slot, u.x, u.y, u.alt);
            }
        }
        let ground = self
            .tiles
            .as_ref()
            .map_or(0, |t| t.relief.height(u.x, u.y).max(t.hills.ground_alt(u.x, u.y)));
        // The room kept under a dive (a low flyer's terrain following keeps its own).
        let room = if noe { crate::terrain_follow::NOE_CLEARANCE / 2 } else { 200 };
        if u.pitch < 0
            && u.alt - ground < room - self.trig.sin_mul(u.pitch, 2000)
            && u.flags & 0x220 == 0
        {
            dp = 0x400;
        }
        u.pitch = s16(u.pitch + cdiv(clamp_q(dp, -0x400, 0x400) * 4, fs));
        if abs16(u.pitch) > 0x4000 {
            u.heading = s16(u.heading + 0x8000);
            u.bank = s16(u.bank + 0x8000);
            u.pitch = s16(0x8000 - u.pitch);
        }
        u.flags &= !fflags::HIT;
        let mut mv = (((0x8000 - u.pitch) & 0xFFFF) * u.speed) >> 15;
        mv -= self.trig.sin_mul(u.bank, mv).abs() >> 1;
        if mode == 1 && self.diff >= self.pl.threat_count_last {
            mv += mv >> 1;
        }
        mv = cdiv(mv * 4, fs) >> 2;
        let h = self.trig.cos_mul(u.pitch, mv);
        u.wx = u.wx.wrapping_add(self.trig.sin_mul(u.heading, h));
        u.wy = u.wy.wrapping_sub(self.trig.cos_mul(u.heading, h));
        u.alt += self.trig.sin_mul(u.pitch, mv);
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        if u.alt > 30000 {
            u.pitch = 0;
        }
    }

    fn terrain_flag(&mut self, i: usize, u: &mut FlightUnit) {
        if ((i as i32 ^ self.frame) & 3) == 0 {
            // Mountains ahead: climb clear of them (not the airfield traffic, nor a wreck).
            let traffic = fflags::TAKEOFF | fflags::FINAL | fflags::ROLLOUT | fflags::DESTROYED;
            // The rolling hills too, with less room (they are low and gentle).
            let terrain = self.terrain_ahead(u);
            let hills = self.hills_ahead(u);
            let low = (terrain > 0 && u.alt < terrain + TERRAIN_CLEARANCE)
                || (hills > 0 && u.alt < hills + HILLS_CLEARANCE);
            if u.flags & traffic == 0 && low {
                u.flags |= fflags::TERRAIN;
            } else {
                u.flags &= !fflags::TERRAIN;
            }
        }
    }

    /// Enemy gun round.
    pub(crate) fn fire_gun(&mut self, u: &FlightUnit) {
        let slot = self.n_bullets + ((self.frame & 0xC) >> 2) as usize;
        let v = cdiv(224, self.fs);
        let vz = self.trig.sin_mul(s16(-u.pitch), v) << 5;
        let h = self.trig.cos_mul(u.pitch, v);
        let vx = self.trig.sin_mul(u.heading, h);
        let vy = -self.trig.cos_mul(u.heading, h);
        // Map units, in a round's fine units.
        let (f, m) = (ROUND_FINE, 32 * ROUND_FINE);
        if let Some(b) = self.bullets.get_mut(slot) {
            *b = Bullet {
                x: u.x * m,
                y: u.y * m,
                alt: u.alt * f,
                vx: vx * m,
                vy: vy * m,
                vz: vz * f,
            };
        }
    }

    /// Scramble check for an inactive unit: may launch it from a random alert base.
    fn scramble(&mut self, i: usize) {
        if self.quiet
            || self.scrambles_left == Some(0)
            || (((self.mission_tick >> (self.roe + 4)) ^ i as i32) & 7) != 0
            || self.is_traffic(i)
            || i >= self.mission_flights
            || i == 0
        {
            return;
        }
        if self.woc == 0 {
            return;
        }
        let b = self.rng.random(self.woc as i32).clamp(0, self.woc as i32 - 1) as usize;
        let u = self.flights[i];
        let o = self.objects[b];
        let range = self.ac_type(u.spec).range as i32;
        if (self.alarm_timer > self.alarm_recent() || u.flags & fflags::SCRAMBLE != 0)
            && (o.flags & 0x181) == 1
            && o.alert == u.spec
            && (range > approx(self.ref_x - o.x, self.ref_y - o.y) >> 6
                || self.seen_near(o.x, o.y, range))
        {
            self.spawn(i, b);
            if let Some(n) = self.scrambles_left.as_mut() {
                *n -= 1;
            }
        }
    }

    /// A pilot the defences reported (more pilots in the world: each where he was seen) is
    /// within `range` km of (x, y): a base there scrambles for him too.
    fn seen_near(&self, x: i32, y: i32, range: i32) -> bool {
        self.present().into_iter().any(|k| {
            self.pilot_ref(k).seen.is_some_and(|(sx, sy, _)| range > approx(sx - x, sy - y) >> 6)
        })
    }

    /// Put flight unit `i` on the runway (or carrier deck) of base object `b`.
    pub(crate) fn spawn(&mut self, i: usize, b: usize) {
        let o = self.objects[b];
        let t = self.ac_type(self.flights[i].spec).clone();
        let ns = self.ns;
        let fs = self.fs;
        let leader = self.escort_leader;
        let u = &mut self.flights[i];
        u.heading = if ns == 1 { 0 } else { s16(0x8000) };
        if o.flags & oflags::CARRIER != 0 {
            u.x = o.x + 3 * ns;
            u.y = o.y - 12 * ns;
            u.alt = 140;
            u.speed = 100;
            u.heading = s16(u.heading - 0x400);
        } else {
            u.x = o.x;
            u.y = o.y + 30 * ns;
            u.alt = 12;
            u.speed = 10;
        }
        u.wx = u.x << 5;
        u.wy = u.y << 5;
        u.pitch = 0;
        u.bank = 0;
        u.flags |= 0x403;
        u.wp = b as i32;
        u.timer = s16(cdiv((t.range as i32 * fs) << 11, t.max_speed as i32));
        if leader == -1 {
            u.flags &= !fflags::ESCORT;
        }
        if !self.is_traffic(i) {
            let base = self.place_name(b);
            self.msg(text!("sim-taking-off", base = base.to_string(), aircraft = t.name.clone()));
        }
        self.emit(SimEvent::AircraftTakeoff { unit: i, base: b });
    }

    /// One of the base traffic flight slots (the mission's last 4 flight units).
    pub(crate) fn is_traffic(&self, i: usize) -> bool {
        i < self.mission_flights && i + 4 >= self.mission_flights
    }

    /// Base traffic slots: two flying, two parked (None with fewer than 8 mission flights).
    fn traffic_slots(&self) -> Option<[usize; 4]> {
        let n = self.mission_flights;
        (n >= 8).then(|| [n - 1, n - 2, n - 3, n - 4])
    }

    /// Base traffic: the last 4 flight units of the mission live at the nearest friendly base.
    pub(crate) fn update_base_traffic(&mut self) {
        let Some([t0, t1, t2, t3]) = self.traffic_slots() else { return };
        let b = self.pl.nearest_base;
        if b == self.traffic_base {
            return;
        }
        self.traffic_base = b;
        if b < 0 {
            return;
        }
        let o = self.obj(b);
        if o.flags & oflags::DISABLED != 0 {
            self.traffic_base = -1;
            return;
        }
        let carrier = o.flags & oflags::CARRIER != 0;
        let friendly = o.flags & oflags::FRIENDLY != 0;
        let neutral = o.flags & oflags::NEUTRAL != 0 && !friendly;
        let (s1, s2) = if neutral {
            if carrier { (24, 24) } else { (27, 27) }
        } else if carrier {
            (8, 8)
        } else if friendly {
            (21, 28)
        } else {
            (0, 0)
        };
        let park = if friendly || neutral { 10 } else { 11 };
        // The two apron spots nearest the base (within 4 km), if any.
        let mut spots: Vec<(i32, i32, i32)> = self
            .parking
            .iter()
            .copied()
            .filter(|&(x, y, _)| ((x >> 5) - o.x).abs() < 0x100 && ((y >> 5) - o.y).abs() < 0x100)
            .collect();
        spots.sort_by_key(|&(x, y, _)| ((x >> 5) - o.x).pow(2) + ((y >> 5) - o.y).pow(2));
        // The second up half a circuit after the first.
        let wait = (CIRCUIT_S / 2 * self.fs).min(0x7FFF);
        for (k, spec, rest) in [(t0, s1, 0), (t1, s2, wait)] {
            let u = &mut self.flights[k];
            u.flags = 0;
            u.spec = spec;
            u.wp = b;
            u.timer = rest;
        }
        for (n, k) in [t2, t3].into_iter().enumerate() {
            let u = &mut self.flights[k];
            u.spec = park;
            u.flags = fflags::EXISTS | fflags::AIRBORNE;
            u.speed = 0;
            u.wp = b;
            u.alt = if carrier { 140 } else { 12 };
            if let Some(&(wx, wy, heading)) = spots.get(n).filter(|_| !carrier) {
                u.x = wx >> 5;
                u.y = wy >> 5;
                u.wx = wx;
                u.wy = wy;
                u.heading = s16(heading);
            } else if !carrier && (!self.parking.is_empty() || self.opts.campaign) {
                // No apron spot left at this base, or none known yet (the scenery is still
                // being laid out, or a dedicated server has none): not parked on its runway
                // instead. Only a classic mission parks on the base object.
                u.flags = 0;
                continue;
            } else if carrier {
                // Aft on the deck's far side from the player's start (4 units to the near
                // side, see `start_state_at`), clear of it and of each other.
                u.x = o.x + 4 * self.ns;
                u.y = o.y + self.ns * (10 + 5 * n as i32);
                u.wx = u.x << 5;
                u.wy = u.y << 5;
                u.heading = if self.ns == 1 { 0 } else { s16(0x8000) };
            } else {
                u.x = o.x + 2 + 2 * n as i32;
                u.y = o.y;
                u.wx = u.x << 5;
                u.wy = u.y << 5;
                u.heading = if self.ns == 1 { 0 } else { s16(0x8000) };
            }
            u.pitch = 0;
            u.bank = 0;
        }
    }

    /// Taxies flight unit `i`, stopped at the end of its landing roll, off the runway to a free
    /// apron spot of its base (past the two the parked base traffic takes), or into a hangar
    /// (out of the world) when there is none. On a carrier's deck it stays.
    fn park_landed(&mut self, i: usize) {
        let u = self.flights[i];
        if self.parking.is_empty() || u.alt > 12 {
            return;
        }
        let o = self.obj(u.wp);
        let mut spots: Vec<(i32, i32, i32)> = self
            .parking
            .iter()
            .copied()
            .filter(|&(x, y, _)| ((x >> 5) - o.x).abs() < 0x100 && ((y >> 5) - o.y).abs() < 0x100)
            .collect();
        spots.sort_by_key(|&(x, y, _)| ((x >> 5) - o.x).pow(2) + ((y >> 5) - o.y).pow(2));
        let taken = |x: i32, y: i32| {
            self.flights.iter().enumerate().any(|(k, f)| {
                k != i
                    && f.flags & fflags::EXISTS != 0
                    && (f.wx - x).abs() < 0x40
                    && (f.wy - y).abs() < 0x40
            })
        };
        let free = spots.iter().skip(2).copied().find(|&(x, y, _)| !taken(x, y));
        let u = &mut self.flights[i];
        let Some((wx, wy, heading)) = free else {
            u.flags = 0;
            return;
        };
        u.wx = wx;
        u.wy = wy;
        u.x = wx >> 5;
        u.y = wy >> 5;
        u.heading = s16(heading);
        u.speed = 0;
    }

    /// Every 128 frames one of the two traffic aircraft (alternating) not up nor resting
    /// appears on its circuit, on the downwind leg east of the base.
    pub(crate) fn spawn_traffic(&mut self) {
        let Some([t0, t1, ..]) = self.traffic_slots() else { return };
        if self.traffic_base < 0 {
            return;
        }
        self.traffic_toggle = !self.traffic_toggle;
        let k = if self.traffic_toggle { t0 } else { t1 };
        if self.flights[k].flags & fflags::EXISTS != 0 {
            return;
        }
        let fs = self.fs;
        let o = self.obj(self.traffic_base);
        let u = &mut self.flights[k];
        // Resting (frames left; this slot comes up every 256).
        if u.timer > 0 {
            u.timer = (u.timer - 256).max(0);
            return;
        }
        u.flags = fflags::EXISTS | fflags::AIRBORNE;
        u.alt = CIRCUIT_ALT;
        u.speed = 250;
        u.wp = self.traffic_base;
        u.x = o.x + CIRCUIT.0;
        u.y = o.y;
        u.wx = u.x << 5;
        u.wy = u.y << 5;
        u.heading = s16(0x8000);
        u.pitch = 0;
        u.bank = 0;
        u.timer = (CIRCUIT_S * fs).min(0x7FFF);
    }

    /// New smoke puff; ages all others by 1.
    pub(crate) fn add_smoke(&mut self, slot: usize, x: i32, y: i32, alt: i32) {
        for s in self.smoke.iter_mut() {
            s.age += 1;
        }
        if let Some(s) = self.smoke.get_mut(slot) {
            *s = Smoke { x, y, alt, age: 0 };
        }
    }

    /// Smoke of the last ground kill and the falling wreck.
    pub(crate) fn update_effects(&mut self) {
        if self.smoke_src > 0 {
            for s in self.smoke.iter_mut() {
                s.alt += 4;
                s.y += s.alt >> 9;
            }
            if self.frame & 7 == 0 {
                let o = self.obj(self.smoke_src);
                let slot = ((self.frame >> 3) & 15) as usize;
                self.add_smoke(slot, o.x, o.y, 0x40);
            }
        }
        if self.wreck.alt > 0 {
            if self.wreck.vel > -16 {
                self.wreck.vel -= 12;
            }
            self.wreck.alt += self.wreck.vel;
        }
    }
}
