//! The atmosphere effects' switches (OPTIONS > GRAPHICS > Effects, the in-flight menu's
//! EFFECTS page): one per effect. `--effects <list>` sets them from the command line: names
//! (`Effect::name`) turn effects on, `-name` off, `all` / `none` all of them.

use serde::{Deserialize, Serialize};
use stealth_i18n::{t, t_lines, ts};

/// Which atmosphere effects are on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(default)]
pub struct Effects {
    /// Camera shake and airframe buffet (shake.rs).
    pub shake: bool,
    /// Flak, explosions and missile launches light their surroundings (flashes.rs).
    pub flashes: bool,
    /// Conventional war at night: tracer barrages and searchlights over the towns, which catch
    /// the other side's drones (barrage.rs).
    pub barrage: bool,
    /// The cockpit's red flood light at night and the master caution's pulse (cockpit3d).
    pub cockpit_night: bool,
    /// Night vision goggles: halos round bright lights, the tube's edge, stronger grain.
    pub nvg: bool,
    /// Rain falls as snow above the freezing level (weather.rs).
    pub snow: bool,
    /// Ground mist over the low ground and the sea at dawn, dusk and night (palette.wgsl).
    pub mist: bool,
    /// The clouds' shadows on the ground (palette.wgsl).
    pub cloud_shadows: bool,
    /// Shafts of sunlight through the gaps of a broken deck (sunbeams.rs).
    pub sunbeams: bool,
    /// Wingtip vortices pulling hard, rotor downwash dust and spray (trails.rs, explosions.rs).
    pub vapour: bool,
    /// The northern lights over the North Cape, shooting stars, low stars twinkling.
    pub night_sky: bool,
    /// Retro (off by default): VGA colours, 64 steps a channel, dithered (palette.wgsl).
    pub vga: bool,
    /// Retro (off by default, desktop): CRT scanlines (palette.wgsl).
    pub scanlines: bool,
    /// Retro (off by default, desktop): the 3D view in big square pixels, the anti-aliasing
    /// off (pixelize.rs).
    pub pixelize: bool,
    /// Pixelize: the side of a block, in screen pixels (`PIXEL_BLOCKS`).
    pub pixel_block: u32,
    /// Pixelize leaves the HUD sharp (`ViewPart::Hud`).
    pub sharp_hud: bool,
    /// Pixelize leaves the cockpit's structure sharp (`ViewPart::Cockpit`).
    pub sharp_cockpit: bool,
    /// Pixelize leaves the MFDs, the gauges, the lamps and the kneeboard sharp (`ViewPart::Gauges`).
    pub sharp_gauges: bool,
}

/// A part of the cockpit view the pixelize look can leave sharp (it still pixelizes the rest).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewPart {
    /// The HUD (and the helmet display).
    Hud,
    /// The cockpit itself: the panel, the frame, the seat, the buttons, the stick, the throttle.
    Cockpit,
    /// The live displays and the instrument faces: the MFDs, the attitude ball, the gauges, the
    /// lamps, the readouts, the labels and the kneeboard.
    Gauges,
}

impl Effects {
    /// Whether pixelize leaves this part of the view sharp.
    pub fn sharp(&self, part: ViewPart) -> bool {
        match part {
            ViewPart::Hud => self.sharp_hud,
            ViewPart::Cockpit => self.sharp_cockpit,
            ViewPart::Gauges => self.sharp_gauges,
        }
    }
}

/// The sizes of a pixelize block the player can choose (screen pixels).
pub const PIXEL_BLOCKS: std::ops::RangeInclusive<u32> = 2..=5;

/// The Steam Frame's build (Linux on ARM): ground mist, cloud shadows and vapour start off.
/// Compiled into the world shader mist and cloud shadows cost its GPU (Adreno 750) about 1.4
/// and 2.4 ms a frame in VR, even with no mist or clouds about; a helicopter's downwash dust
/// near the ground about 1.7 ms; enough to drop the frame rate from 72 to 36 fps. Also the
/// Android app's: phone GPUs are weaker still.
pub const HEAVY_EFFECTS_OFF: bool =
    cfg!(any(all(target_arch = "aarch64", target_os = "linux"), target_os = "android"));

impl Default for Effects {
    fn default() -> Self {
        Self {
            shake: true,
            flashes: true,
            barrage: true,
            cockpit_night: true,
            nvg: true,
            snow: true,
            mist: !HEAVY_EFFECTS_OFF,
            cloud_shadows: !HEAVY_EFFECTS_OFF,
            sunbeams: true,
            vapour: !HEAVY_EFFECTS_OFF,
            night_sky: true,
            vga: false,
            scanlines: false,
            pixelize: false,
            pixel_block: *PIXEL_BLOCKS.end(),
            sharp_hud: false,
            sharp_cockpit: false,
            sharp_gauges: false,
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
    Sunbeams,
    Vapour,
    NightSky,
    Vga,
    Scanlines,
    Pixelize,
    SharpHud,
    SharpCockpit,
    SharpGauges,
}

impl Effect {
    pub const ALL: [Effect; 17] = [
        Effect::Shake,
        Effect::Flashes,
        Effect::Barrage,
        Effect::CockpitNight,
        Effect::Nvg,
        Effect::Snow,
        Effect::Mist,
        Effect::CloudShadows,
        Effect::Sunbeams,
        Effect::Vapour,
        Effect::NightSky,
        Effect::Vga,
        Effect::Scanlines,
        Effect::Pixelize,
        Effect::SharpHud,
        Effect::SharpCockpit,
        Effect::SharpGauges,
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
            Effect::Sunbeams => "sunbeams",
            Effect::Vapour => "vapour",
            Effect::NightSky => "night-sky",
            Effect::Vga => "vga",
            Effect::Scanlines => "scanlines",
            Effect::Pixelize => "pixelize",
            Effect::SharpHud => "sharp-hud",
            Effect::SharpCockpit => "sharp-cockpit",
            Effect::SharpGauges => "sharp-gauges",
        }
    }

    /// Its name in the menus.
    pub fn label(self) -> &'static str {
        t!(&format!("effect-{}", self.name()))
    }

    /// Its name in the in-flight menu.
    pub fn menu_label(self) -> &'static str {
        ts!(&format!("fm-effect-{}", self.name()))
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
            Effect::Sunbeams => e.sunbeams,
            Effect::Vapour => e.vapour,
            Effect::NightSky => e.night_sky,
            Effect::Vga => e.vga,
            Effect::Scanlines => e.scanlines,
            Effect::Pixelize => e.pixelize,
            Effect::SharpHud => e.sharp_hud,
            Effect::SharpCockpit => e.sharp_cockpit,
            Effect::SharpGauges => e.sharp_gauges,
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
            Effect::Sunbeams => e.sunbeams = on,
            Effect::Vapour => e.vapour = on,
            Effect::NightSky => e.night_sky = on,
            Effect::Vga => e.vga = on,
            Effect::Scanlines => e.scanlines = on,
            Effect::Pixelize => e.pixelize = on,
            Effect::SharpHud => e.sharp_hud = on,
            Effect::SharpCockpit => e.sharp_cockpit = on,
            Effect::SharpGauges => e.sharp_gauges = on,
        }
    }

    /// A retro look (OPTIONS > GRAPHICS > Retro, the in-flight GRAPHICS page), not an
    /// atmosphere effect.
    pub fn retro(self) -> bool {
        matches!(self, Effect::Vga | Effect::Scanlines | Effect::Pixelize) || self.pixel_part()
    }

    /// One of the switches that leave a part of the view sharp: only with pixelize on.
    pub fn pixel_part(self) -> bool {
        matches!(self, Effect::SharpHud | Effect::SharpCockpit | Effect::SharpGauges)
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
        assert!(Effects::parse("pixelize").pixelize);
        assert!(!Effects::default().pixelize);
        assert!(PIXEL_BLOCKS.contains(&Effects::default().pixel_block));
        let e = Effects::parse("pixelize,sharp-hud,sharp-gauges");
        assert!(e.sharp(ViewPart::Hud) && e.sharp(ViewPart::Gauges) && !e.sharp(ViewPart::Cockpit));
        assert!(!Effects::default().sharp(ViewPart::Hud));
    }
}
