/// FluidSynth 音频渲染模块
/// 使用 oxisynth (纯 Rust 实现的 FluidSynth)
/// 支持多种音色、鼓点、音量增益、循环播放

use std::path::Path;
use oxisynth::{SoundFont, Synth, MidiEvent};
use rand::Rng;

/// 将音符频率转换为 MIDI 音符编号
fn freq_to_midi_note(freq: f32) -> u8 {
    let midi_note = 69.0 + 12.0 * (freq / 440.0).log2();
    midi_note.round().clamp(0.0, 127.0) as u8
}

/// General MIDI 常用乐器
pub const GM_INSTRUMENTS: &[(u8, &str)] = &[
    (0, "Acoustic Grand Piano"),
    (1, "Bright Acoustic Piano"),
    (6, "Harpsichord"),
    (8, "Celesta"),
    (11, "Vibraphone"),
    (13, "Marimba"),
    (15, "Dulcimer"),
    (20, "Reed Organ"),
    (22, "Accordion"),
    (25, "Acoustic Guitar (nylon)"),
    (26, "Acoustic Guitar (steel)"),
    (41, "Violin"),
    (42, "Viola"),
    (43, "Cello"),
    (47, "Harp"),
    (57, "Trumpet"),
    (67, "Tenor Sax"),
    (69, "Oboe"),
    (72, "Clarinet"),
    (74, "Flute"),
    (76, "Pan Flute"),
    (80, "Ocarina"),
];

/// 随机选择一个乐器
pub fn random_instrument() -> (u8, &'static str) {
    let idx = rand::thread_rng().gen_range(0..GM_INSTRUMENTS.len());
    GM_INSTRUMENTS[idx]
}

/// 生成鼓点模式
fn generate_drum_pattern(bpm: u32, bars: u32) -> Vec<(f32, u8, u8)> {
    let beat_duration = 60.0 / bpm as f32;
    let mut drums = Vec::new();

    for bar in 0..bars {
        let bar_start = bar as f32 * 4.0 * beat_duration;
        for beat in 0..4 {
            let beat_time = bar_start + beat as f32 * beat_duration;
            if beat == 0 || beat == 2 {
                drums.push((beat_time, 36, 90)); // Bass Drum
            } else {
                drums.push((beat_time, 36, 60));
            }
            if beat == 1 || beat == 3 {
                drums.push((beat_time, 38, 80)); // Snare
            }
            drums.push((beat_time, 42, 50)); // Closed HH
            drums.push((beat_time + 0.5 * beat_duration, 42, 40));
            if bar % 2 == 1 && beat == 3 {
                drums.push((beat_time + 0.5 * beat_duration, 46, 70)); // Open HH
            }
        }
    }
    drums.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    drums
}

/// 使用 FluidSynth 渲染，支持自然 BPM + 循环 + 增益
pub fn render_with_fluidsynth(
    notes: &[(f32, f32)],
    bpm: u32,
    duration: f64,
    soundfont_path: &Path,
    output_path: &Path,
    sample_rate: u32,
    instrument: u8,
    enable_drums: bool,
    gain_db: f64,
) -> Result<(), String> {
    let mut synth = Synth::new(oxisynth::SynthDescriptor {
        sample_rate: sample_rate as f32,
        ..Default::default()
    }).map_err(|e| format!("Failed to create synth: {:?}", e))?;

    let mut file = std::fs::File::open(soundfont_path)
        .map_err(|e| format!("Failed to open soundfont: {:?}", e))?;
    let soundfont = SoundFont::load(&mut file)
        .map_err(|e| format!("Failed to load soundfont: {:?}", e))?;
    synth.add_font(soundfont, true);

    let beat_duration = 60.0 / bpm as f64;
    let total_beats: f32 = notes.iter().map(|(_, dur)| dur).sum();
    let one_pass_time = total_beats as f64 * beat_duration;

    if one_pass_time <= 0.0 {
        return Err("Empty melody".to_string());
    }

    // 生成单次遍历的 MIDI 事件
    let mut one_pass_events: Vec<(f32, MidiEvent)> = Vec::new();

    // Program Change
    one_pass_events.push((0.0, MidiEvent::ProgramChange {
        channel: 0,
        program_id: instrument,
    }));

    // 主旋律 (Channel 0)
    let mut t = 0.0;
    for (freq, dur) in notes {
        let midi = freq_to_midi_note(*freq);
        let secs = (*dur as f64 * beat_duration) as f32;
        let vel = 70 + rand::thread_rng().gen_range(0..20);
        one_pass_events.push((t, MidiEvent::NoteOn { channel: 0, key: midi, vel }));
        one_pass_events.push((t + secs * 0.95, MidiEvent::NoteOff { channel: 0, key: midi }));
        t += secs;
    }

    // 鼓点 (Channel 9)
    if enable_drums {
        let bars = ((one_pass_time / (4.0 * beat_duration)).ceil() as u32).max(1);
        for (time, note, vel) in generate_drum_pattern(bpm, bars) {
            one_pass_events.push((time, MidiEvent::NoteOn { channel: 9, key: note, vel }));
            one_pass_events.push((time + 0.15, MidiEvent::NoteOff { channel: 9, key: note }));
        }
    }

    one_pass_events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 如果一轮不够，循环生成多轮事件
    let num_loops = (duration / one_pass_time).ceil() as u32;
    let mut all_events: Vec<(f32, MidiEvent)> = Vec::new();
    let one_pass_f32 = one_pass_time as f32;

    for loop_idx in 0..num_loops {
        let offset = loop_idx as f32 * one_pass_f32;
        for (t, ev) in &one_pass_events {
            let abs_time = offset + t;
            if abs_time < duration as f32 {
                all_events.push((abs_time, ev.clone()));
            }
        }
    }

    all_events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 增益倍数
    let gain_mult = 10.0_f64.powf(gain_db / 20.0) as f32;

    // 渲染
    let total_samples = (duration * sample_rate as f64) as usize;
    let mut buf = vec![0.0f32; total_samples * 2];
    let mut ev_idx = 0;
    let sps = sample_rate as f32;

    for si in 0..total_samples {
        let now = si as f32 / sps;
        while ev_idx < all_events.len() && all_events[ev_idx].0 <= now {
            synth.send_event(all_events[ev_idx].1.clone())
                .map_err(|e| format!("Failed to send event: {:?}", e))?;
            ev_idx += 1;
        }
        let mut sb = [0.0f32, 0.0f32];
        synth.write(&mut sb[..]);
        buf[si * 2] = (sb[0] * gain_mult).clamp(-1.0, 1.0);
        buf[si * 2 + 1] = (sb[1] * gain_mult).clamp(-1.0, 1.0);
    }

    write_wav(output_path, &buf, sample_rate)?;
    Ok(())
}

fn write_wav(path: &Path, samples: &[f32], sr: u32) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;
    let mut f = File::create(path).map_err(|e| e.to_string())?;
    let nc = 2u16;
    let bps = 16u16;
    let ns = (samples.len() / 2) as u32;
    let br = sr * nc as u32 * bps as u32 / 8;
    let ba = nc * bps / 8;
    let ds = ns * nc as u32 * bps as u32 / 8;

    f.write_all(b"RIFF").map_err(|e| e.to_string())?;
    f.write_all(&(36 + ds).to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(b"WAVE").map_err(|e| e.to_string())?;
    f.write_all(b"fmt ").map_err(|e| e.to_string())?;
    f.write_all(&16u32.to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(&1u16.to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(&nc.to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(&sr.to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(&br.to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(&ba.to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(&bps.to_le_bytes()).map_err(|e| e.to_string())?;
    f.write_all(b"data").map_err(|e| e.to_string())?;
    f.write_all(&ds.to_le_bytes()).map_err(|e| e.to_string())?;

    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        f.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 检查 SoundFont 是否存在
pub fn check_soundfont_exists(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    crate::soundfont_manager::get_soundfont_path(app)
}
