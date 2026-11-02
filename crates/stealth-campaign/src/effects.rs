//! What the players' sorties change in the war, beyond what they destroy: a CAP keeps the
//! enemy's raids and insertions away, an escort its fighters off the helicopters, a struck
//! airfield or city holds back its operations. Each pilot's debrief lists his.

use stealth_i18n::{Text, text};

use crate::state::*;
use crate::tuning::*;

impl CampaignState {
    /// The player flying CAP over (x, y) this moment, if any.
    pub(crate) fn cap_over(&self, x: i32, y: i32) -> Option<PlayerId> {
        self.players.iter().find_map(|slot| {
            let s = slot.sortie.as_ref()?;
            let Objective::Cap { x: cx, y: cy } = s.offer.objective else { return None };
            (s.progress.here && dist(cx - x, cy - y) < CAP_COVER).then_some(slot.id)
        })
    }

    /// A player on CAP covers friendly place `p`: the enemy sends no raid or insertion there.
    pub(crate) fn capped(&self, p: PlaceId) -> bool {
        let p = &self.places[p as usize];
        p.owner == Side::Blue && self.cap_over(p.x, p.y).is_some()
    }

    /// The enemy picks a target: the players on CAP hear which places they kept it from (on
    /// its front, or with an airfield to raid).
    pub(crate) fn credit_cap_cover(&mut self) {
        let front = self.front(Side::Red);
        let covered: Vec<(PlayerId, String)> = self
            .places
            .iter()
            .enumerate()
            .filter(|(k, p)| {
                let airfield = p.airfield.and_then(|a| self.site(a)).is_some_and(Site::alive);
                p.owner == Side::Blue && (airfield || front.contains(&(*k as PlaceId)))
            })
            .filter_map(|(_, p)| Some((self.cap_over(p.x, p.y)?, p.name.clone())))
            .collect();
        for (player, place) in covered {
            self.effect(player, text!("cl-fx-cap", place = place));
        }
    }

    /// The player escorting insertion `id`, near its helicopters this moment.
    pub(crate) fn escort_with(&self, id: u32) -> Option<PlayerId> {
        self.players.iter().find_map(|slot| {
            let s = slot.sortie.as_ref()?;
            (s.offer.objective == Objective::Escort { insertion: id } && s.progress.here)
                .then_some(slot.id)
        })
    }

    /// `player`'s sortie changed the war: once in his debrief.
    pub(crate) fn effect(&mut self, player: PlayerId, t: Text) {
        if let Some(s) = self.slot_mut(player).sortie.as_mut()
            && !s.progress.effects.contains(&t)
        {
            s.progress.effects.push(t);
        }
    }

    /// An enemy site destroyed (by `by`, if a player): its airfield's raids and its city's
    /// convoys and insertions are held back, and the player hears what it did.
    pub(crate) fn site_struck(&mut self, object: ObjectId, by: Option<PlayerId>) {
        let Some(s) = self.site(object).filter(|s| s.alive()) else { return };
        if self.owner(object) != Some(Side::Red) {
            return;
        }
        let (kind, name, repair) = (s.kind, s.name.clone(), s.repair_time());
        let now = self.clock;
        let skill = (self.setup.opponents as usize).min(3);
        let mins = |t: f64| (t / 60.0).round() as i64;
        let fx = match kind {
            SiteKind::Airfield => {
                let every = self.paced(RAID_EVERY[skill]);
                self.raid_at = self.raid_at.max(now + STRUCK_AIRFIELD_DELAY - every);
                Some(text!(
                    "cl-fx-airfield",
                    place = name,
                    mins = mins(repair),
                    delay = mins(STRUCK_AIRFIELD_DELAY)
                ))
            }
            SiteKind::City => {
                let insert = self.paced(ENEMY_INSERT_EVERY[skill]);
                let convoy = self.paced(ENEMY_CONVOY_EVERY[skill]);
                self.enemy_insert_at = self.enemy_insert_at.max(now + STRUCK_CITY_DELAY - insert);
                let c = &mut self.convoy_at[1];
                *c = c.max(now + STRUCK_CITY_DELAY - convoy);
                Some(text!(
                    "cl-fx-city",
                    place = name,
                    loss = CITY_LOSS as i64,
                    income = (INCOME_CITY * self.pace()).round() as i64,
                    delay = mins(STRUCK_CITY_DELAY)
                ))
            }
            // The time it takes counts this loss too.
            SiteKind::Sam => {
                let mut s = self.site(object).expect("checked").clone();
                if s.repaired_at.is_some_and(|t| now - t > REPAIR_MEMORY) {
                    s.hits = 0;
                }
                s.hits = s.hits.saturating_add(1);
                Some(text!("cl-fx-sam", place = name, mins = mins(s.repair_time())))
            }
            _ => None,
        };
        if let (Some(p), Some(t)) = (by, fx) {
            self.effect(p, t);
        }
    }
}
