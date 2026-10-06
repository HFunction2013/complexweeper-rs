// Sound effects: six embedded WAV clips (four mine-blast sounds, a win fanfare
// and a one-second tick) loaded through macroquad's audio backend.
//
// The clips come from the original Zig version (extracted from its sounds.bin,
// converted to 16-bit PCM for compatibility with miniaudio). Playing them is a
// no-op whenever audio fails to initialize or the user mutes sound.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

pub const MINE_1_WAV: &[u8] = include_bytes!("../assets/sounds/mine_1.wav");
pub const MINE_2_WAV: &[u8] = include_bytes!("../assets/sounds/mine_2.wav");
pub const MINE_3_WAV: &[u8] = include_bytes!("../assets/sounds/mine_3.wav");
pub const MINE_4_WAV: &[u8] = include_bytes!("../assets/sounds/mine_4.wav");
pub const WIN_WAV: &[u8] = include_bytes!("../assets/sounds/win.wav");
pub const TICK_WAV: &[u8] = include_bytes!("../assets/sounds/tick.wav");

pub struct Sounds {
    /// One blast sound per mine type (index 0..4, matching Game.mine values).
    pub mine: [Sound; 4],
    pub win: Sound,
    pub tick: Sound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundKind {
    /// Explode a mine of the given type (1..=4).
    Mine(u8),
    Win,
    Tick,
}

impl Sounds {
    /// Load every embedded clip. Returns None if the audio backend is
    /// unavailable so the game can keep running silently.
    pub async fn load() -> Option<Sounds> {
        let mine_1 = load_sound_from_bytes(MINE_1_WAV).await.ok()?;
        let mine_2 = load_sound_from_bytes(MINE_2_WAV).await.ok()?;
        let mine_3 = load_sound_from_bytes(MINE_3_WAV).await.ok()?;
        let mine_4 = load_sound_from_bytes(MINE_4_WAV).await.ok()?;
        let win = load_sound_from_bytes(WIN_WAV).await.ok()?;
        let tick = load_sound_from_bytes(TICK_WAV).await.ok()?;
        Some(Sounds {
            mine: [mine_1, mine_2, mine_3, mine_4],
            win,
            tick,
        })
    }

    pub fn play(&self, kind: SoundKind) {
        let (sound, volume) = match kind {
            SoundKind::Mine(t) => (&self.mine[((t as usize).saturating_sub(1)).min(3)], 1.0),
            SoundKind::Win => (&self.win, 1.0),
            SoundKind::Tick => (&self.tick, 0.5),
        };
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }
}
