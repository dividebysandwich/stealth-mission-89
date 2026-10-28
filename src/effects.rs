//! The atmosphere effects' switches (OPTIONS > GRAPHICS > Effects, the in-flight menu's
//! EFFECTS page): one per effect, all cheap. `--effects <list>` sets them from the command
//! line: names (`Effect::name`) turn effects on, `-name` off, `all` / `none` all of them.

use serde::{Deserialize, Serialize};
use stealth_i18n::{t, t_lines, tf};

/// Which atmosphere effects are on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(default)]
pub struct Effects {
    /// Camera shake and airframe buffet (shake.rs).
    pub shake: bool,
    /// Flak, explosions and missile launches light their surroundings (flashes.rs).
    pub flashes: bool,
    /// Conventional war at night: tracer barrages and searchlights over the towns (barrage.rs).
    pub barrage: bool,
    /// The cockpit's red flood light at night and the master caution's pulse (cockpit3d).
    pub cockpit_night: bool,
    /// Night vision goggles: halos round bright lights, the tube's edge, stronger grain.
    pub nvg: bool,
    /// Rain falls as snow above the freezing level (weather.rs).
    pub snow: bool,
    /// Ground mist over the low ground and the sea at dawn, dusk and night (palette.wgsl).
    pub mist: bool,
    /// The stratus deck's shadows on the ground (palette.wgsl).
    pub cloud_shadows: bool,
    /// Wingtip vortices pulling hard, rotor downwash dust and spray (trails.rs, explosions.rs).
    pub vapour: bool,
    /// The northern lights over the North Cape, shooting stars, low stars twinkling.
    pub night_sky: bool,
    /// Retro (off by default): VGA colours, 64 steps a channel, dithered (palette.wgsl).
    pub vga: bool,
    /// Retro (off by default, desktop): CRT scanlines (palette.wgsl).
    pub scanlines: bool,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            shake: true,
            flashes: true,
            barrage: true,
            cockpit_night: true,
            nvg: true,
            snow: true,
            mist: true,
            cloud_shadows: true,
            vapour: true,
            night_sky: true,
            vga: false,
            scanlines: false,
        }
    }
}

/// One effect switch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    Shake,
    Flashes,
    Barrage,
    CockpitNight,
    Nvg,
    Snow,
    Mist,
    CloudShadows,
    Vapour,
    NightSky,
    Vga,
    Scanlines,
}

impl Effect {
    pub const ALL: [Effect; 12] = [
        Effect::Shake,
        Effect::Flashes,
        Effect::Barrage,
        Effect::CockpitNight,
        Effect::Nvg,
        Effect::Snow,
        Effect::Mist,
        Effect::CloudShadows,
        Effect::Vapour,
        Effect::NightSky,
        Effect::Vga,
        Effect::Scanlines,
    ];

    /// Its name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Effect::Shake => "shake",
            Effect::Flashes => "flashes",
            Effect::Barrage => "barrage",
            Effect::CockpitNight => "cockpit-night",
            Effect::Nvg => "nvg",
            Effect::Snow => "snow",
            Effect::Mist => "mist",
            Effect::CloudShadows => "cloud-shadows",
            Effect::Vapour => "vapour",
            Effect::NightSky => "night-sky",
            Effect::Vga => "vga",
            Effect::Scanlines => "scanlines",
        }
    }

    /// Its name in the menus.
    pub fn label(self) -> &'static str {
        t!(&format!("effect-{}", self.name()))
    }

    /// Its name in the in-flight menu.
    pub fn menu_label(self) -> &'static str {
        tf!(&format!("fm-effect-{}", self.name()))
    }

    /// What it does (the options window's notes under it).
    pub fn facts(self) -> &'static [&'static str] {
        t_lines!(&format!("effect-{}.facts", self.name()))
    }

    pub fn get(self, e: &Effects) -> bool {
        match self {
            Effect::Shake => e.shake,
            Effect::Flashes => e.flashes,
            Effect::Barrage => e.barrage,
            Effect::CockpitNight => e.cockpit_night,
            Effect::Nvg => e.nvg,
            Effect::Snow => e.snow,
            Effect::Mist => e.mist,
            Effect::CloudShadows => e.cloud_shadows,
            Effect::Vapour => e.vapour,
            Effect::NightSky => e.night_sky,
            Effect::Vga => e.vga,
            Effect::Scanlines => e.scanlines,
        }
    }

    pub fn set(self, e: &mut Effects, on: bool) {
        match self {
            Effect::Shake => e.shake = on,
            Effect::Flashes => e.flashes = on,
            Effect::Barrage => e.barrage = on,
            Effect::CockpitNight => e.cockpit_night = on,
            Effect::Nvg => e.nvg = on,
            Effect::Snow => e.snow = on,
            Effect::Mist => e.mist = on,
            Effect::CloudShadows => e.cloud_shadows = on,
            Effect::Vapour => e.vapour = on,
            Effect::NightSky => e.night_sky = on,
            Effect::Vga => e.vga = on,
            Effect::Scanlines => e.scanlines = on,
        }
    }

    /// A retro look (OPTIONS > GRAPHICS > Retro, the in-flight GRAPHICS page), not an
    /// atmosphere effect.
    pub fn retro(self) -> bool {
        matches!(self, Effect::Vga | Effect::Scanlines)
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
