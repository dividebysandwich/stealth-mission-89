//! The atmosphere effects' switches (OPTIONS > GRAPHICS > Effects, the in-flight menu's
//! EFFECTS page): one per effect, all cheap. `--effects <list>` sets them from the command
//! line: names (`Effect::name`) turn effects on, `-name` off, `all` / `none` all of them.

use serde::{Deserialize, Serialize};

/// Which atmosphere effects are on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(default)]
pub struct Effects {
    /// Camera shake and airframe buffet (shake.rs).
    pub shake: bool,
}

impl Default for Effects {
    fn default() -> Self {
        Self { shake: true }
    }
}

/// One effect switch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    Shake,
}

impl Effect {
    pub const ALL: [Effect; 1] = [Effect::Shake];

    /// Its name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Effect::Shake => "shake",
        }
    }

    /// Its name in the menus.
    pub fn label(self) -> &'static str {
        match self {
            Effect::Shake => "Camera shake",
        }
    }

    /// Its name in the in-flight menu.
    pub fn menu_label(self) -> &'static str {
        match self {
            Effect::Shake => "CAMERA SHAKE",
        }
    }

    /// What it does (the options window's notes under it).
    pub fn facts(self) -> &'static [&'static str] {
        match self {
            Effect::Shake => &[
                "The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway",
                "VR: the cockpit vibrates, your head stays still",
            ],
        }
    }

    pub fn get(self, e: &Effects) -> bool {
        match self {
            Effect::Shake => e.shake,
        }
    }

    pub fn set(self, e: &mut Effects, on: bool) {
        match self {
            Effect::Shake => e.shake = on,
        }
    }

    pub fn toggle(self, e: &mut Effects) {
        self.set(e, !self.get(e));
    }

    fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
    }
}

impl Effects {
    /// Everything off.
    pub fn none() -> Self {
        let mut e = Self::default();
        for f in Effect::ALL {
            f.set(&mut e, false);
        }
        e
    }

    /// `--effects <list>` on top of the defaults: comma separated names, `-name` turns one off,
    /// `all` / `none` all of them. Unknown names are logged and skipped.
    pub fn parse(list: &str) -> Self {
        let mut e = Self::default();
        for word in list.split(',').map(str::trim).filter(|w| !w.is_empty()) {
            match word {
                "all" => {
                    for f in Effect::ALL {
                        f.set(&mut e, true);
                    }
                }
                "none" => e = Self::none(),
                _ => {
                    let (on, name) = match word.strip_prefix('-') {
                        Some(n) => (false, n),
                        None => (true, word),
                    };
                    match Effect::parse(name) {
                        Some(f) => f.set(&mut e, on),
                        None => bevy::log::warn!("--effects: no effect {name}"),
                    }
                }
            }
        }
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_effect_lists() {
        assert_eq!(Effects::parse(""), Effects::default());
        assert!(!Effects::parse("-shake").shake);
        assert!(!Effects::parse("none").shake);
        assert!(Effects::parse("none,shake").shake);
    }
}
