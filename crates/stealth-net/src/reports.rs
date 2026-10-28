//! A player's flight model reports, played out one a step.
//!
//! His client sends a report each of its frames and the server steps on its own clock; taking
//! whatever came last, a report that comes just after a step leaves the aircraft standing for
//! that step and jumping two the next, and everyone sees it shudder. So the reports are kept
//! by their number and played a frame or so behind the newest, one a step; the margin grows
//! when a report is not there in time and shrinks again when it has been more than enough.

use std::collections::VecDeque;

use stealth_sim::{Commands, PlayerState};

use crate::protocol::merge_commands;

/// Reports behind the newest to start at (a frame of margin).
const START: i32 = 1;
/// At most this far behind (a stall in his game): back to the start margin.
const MOST: i32 = 6;
/// Steps over which the margin is watched; if it never fell under 2, one frame is skipped.
const WINDOW: u32 = 60;

#[derive(Default)]
pub struct Reports {
    /// Reports not played yet by number, with their commands (a lost report's from the
    /// repeats, without its state).
    queue: VecDeque<(u32, Option<PlayerState>, Commands)>,
    newest: Option<u32>,
    /// The number played at the last step.
    play: Option<u32>,
    /// The state played last.
    state: Option<PlayerState>,
    /// The smallest margin in this window and the steps into it.
    least: i32,
    steps: u32,
}

impl Reports {
    /// Report `seq` came in with the commands of the reports before it (repeated in case
    /// they were lost). Old and repeated ones are ignored.
    pub fn received(&mut self, seq: u32, state: PlayerState, recent: &[(u32, Commands)]) {
        if self.newest.is_some_and(|n| seq.wrapping_sub(n) as i32 <= 0) {
            return;
        }
        for (s, c) in recent {
            if self.newest.is_none_or(|n| s.wrapping_sub(n) as i32 > 0) && *s != seq {
                self.queue.push_back((*s, None, c.clone()));
            }
        }
        let cmd = state.cmd.clone();
        self.queue.push_back((seq, Some(state), cmd));
        self.newest = Some(seq);
    }

    /// The report for this step with the commands of every report played: None if nothing
    /// new is due (the simulation goes on with the last).
    pub fn step(&mut self) -> Option<PlayerState> {
        let newest = self.newest?;
        let mut play = match self.play {
            None => newest.wrapping_sub(START as u32),
            Some(p) => p.wrapping_add(1),
        };
        let behind = newest.wrapping_sub(play) as i32;
        if behind < 0 {
            // Not there yet: wait for it (a frame more margin from now on).
            play = newest;
        } else if behind > MOST {
            play = newest.wrapping_sub(START as u32);
        }
        // Margin to spare all the time: a frame less.
        self.least = self.least.min(newest.wrapping_sub(play) as i32);
        self.steps += 1;
        if self.steps >= WINDOW {
            if self.least >= 2 {
                play = play.wrapping_add(1);
            }
            self.steps = 0;
            self.least = i32::MAX;
        }
        self.play = Some(play);
        let mut cmd = Commands::default();
        let mut fresh = None;
        while let Some((s, ..)) = self.queue.front()
            && play.wrapping_sub(*s) as i32 >= 0
        {
            let Some((_, state, c)) = self.queue.pop_front() else { break };
            merge_commands(&mut cmd, &c);
            if state.is_some() {
                fresh = state;
            }
        }
        if let Some(s) = fresh {
            self.state = Some(s);
        } else if cmd == Commands::default() {
            return None;
        }
        let mut ps = self.state.clone()?;
        ps.cmd = cmd;
        Some(ps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(n: u32) -> PlayerState {
        PlayerState { x: n as i32, ..Default::default() }
    }

    #[test]
    fn late_and_bunched_reports_play_one_a_step() {
        let mut r = Reports::default();
        let mut seq = 0u32;
        let mut played = Vec::new();
        for step in 0..200u32 {
            // One a step on average: in pairs, late, and three at once.
            let n = [2, 0, 2, 0, 1, 1, 3, 0, 1, 0][step as usize % 10];
            for _ in 0..n {
                seq += 1;
                r.received(seq, report(seq), &[]);
            }
            if let Some(s) = r.step() {
                played.push((step, s.x));
            }
        }
        let late: Vec<_> = played.iter().filter(|(s, _)| *s > 40).collect();
        for w in late.windows(2) {
            assert_eq!(w[1].1 - w[0].1, (w[1].0 - w[0].0) as i32, "{late:?}");
        }
    }

    #[test]
    fn commands_are_played_once_even_from_a_lost_report() {
        let mut r = Reports::default();
        let fire = Commands { fire: true, ..Default::default() };
        let flare = Commands { flare: true, ..Default::default() };
        r.received(1, report(1), &[]);
        r.step();
        r.step();
        // Report 2 (with fire) is lost; 3 repeats its commands.
        let mut three = report(3);
        three.cmd = flare.clone();
        r.received(3, three, &[(2, fire.clone()), (3, flare.clone())]);
        let mut cmds = Commands::default();
        for _ in 0..4 {
            if let Some(s) = r.step() {
                merge_commands(&mut cmds, &s.cmd);
            }
        }
        assert!(cmds.fire && cmds.flare);
        // Repeats of what was played are not played again.
        r.received(4, report(4), &[(2, fire.clone()), (3, flare)]);
        for _ in 0..4 {
            if let Some(s) = r.step() {
                assert!(!s.cmd.fire && !s.cmd.flare);
            }
        }
    }
}
