/// FluidSynth 音频渲染模块
/// 使用 oxisynth (纯 Rust 实现的 FluidSynth)
/// 支持多种音色和鼓点

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

/// 生成鼓点模式（时间偏移 + MIDI 音符）
/// 返回 (时间偏移_拍数, MIDI鼓音符, 力度)
pub fn generate_drum_pattern(bpm: u32, bars: u32) -> Vec<(f32, u8, u8)> {
    let beat_duration = 60.0 / bpm as f32;
    let mut drums = Vec::new();

    for bar in 0..bars {
        let bar_start = bar as f32 * 4.0 * beat_duration;

        for beat in 0..4 {
            let beat_time = bar_start + beat as f32 * beat_duration;

            // 每拍大鼓（1、3拍重）
            if beat == 0 || beat == 2 {
                drums.push((beat_time, 36, 90)); // Bass Drum
            } else {
                drums.push((beat_time, 36, 60));
            }

            // 小鼓（2、4拍）
            if beat == 1 || beat == 3 {
                drums.push((beat_time, 38, 80)); // Acoustic Snare
            }

            // 闭镲（每半拍）
            drums.push((beat_time, 42, 50)); // Closed Hi-Hat
            drums.push((beat_time + 0.5 * beat_duration, 42, 40));

            // 开镲（每2小节结尾）
            if bar % 2 == 1 && beat == 3 {
                drums.push((beat_time + 0.5 * beat_duration, 46, 70)); // Open Hi-Hat
            }
        }
    }

    drums.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    drums
}

/// 使用 FluidSynth 渲染音符序列为音频（支持多音色和鼓点）
pub fn render_with_fluidsynth(
    notes: &[(f32, f32)],
    bpm: u32,
    duration: f64,
    soundfont_path: &Path,
    output_path: &Path,
    sample_rate: u32,
    instrument: u8,
    enable_drums: bool,
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
    let scale_factor = duration / (total_beats as f64 * beat_duration);

    // 生成 MIDI 事件
    let mut events = Vec::new();

    // Program Change: 选择乐器（Channel 0）
    events.push((0.0, MidiEvent::ProgramChange {
        channel: 0,
        program_id: instrument,
    }));

    // 主旋律音符（Channel 0）
    let mut current_time = 0.0;
    for (freq, note_duration) in notes {
        let midi_note = freq_to_midi_note(*freq);
        let duration_secs = (*note_duration as f64 * beat_duration * scale_factor) as f32;
        let velocity = 70 + rand::thread_rng().gen_range(0..20); // 微小力度变化

        events.push((current_time, MidiEvent::NoteOn {
            channel: 0,
            key: midi_note,
            vel: velocity,
        }));
        events.push((current_time + duration_secs * 0.95, MidiEvent::NoteOff {
            channel: 0,
            key: midi_note,
        }));
        current_time += duration_secs;
    }

    // 鼓点（Channel 9 = MIDI Channel 10）
    if enable_drums {
        let total_bars = (duration / (4.0 * beat_duration)).ceil() as u32;
        let drum_pattern = generate_drum_pattern(bpm, total_bars.max(1));

        for (time, drum_note, velocity) in &drum_pattern {
            if *time < duration as f32 {
                events.push((*time, MidiEvent::NoteOn {
                    channel: 9,
                    key: *drum_note,
                    vel: *velocity,
                }));
                events.push((*time + 0.15, MidiEvent::NoteOff {
                    channel: 9,
                    key: *drum_note,
                }));
            }
        }
    }

    // 排序事件
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 渲染音频
    let total_samples = (duration * sample_rate as f64) as usize;
    let mut audio_buffer = vec![0.0f32; total_samples * 2];

    let mut event_idx = 0;
    let samples_per_sec = sample_rate as f32;

    for sample_idx in 0..total_samples {
        let current_time_sec = sample_idx as f32 / samples_per_sec;

        while event_idx < events.len() && events[event_idx].0 <= current_time_sec {
            synth.send_event(events[event_idx].1.clone())
                .map_err(|e| format!("Failed to send event: {:?}", e))?;
            event_idx += 1;
        }

        let mut stereo_buffer = [0.0f32, 0.0f32];
        synth.write(&mut stereo_buffer[..]);

        audio_buffer[sample_idx * 2] = stereo_buffer[0];
        audio_buffer[sample_idx * 2 + 1] = stereo_buffer[1];
    }

    write_wav_file(output_path, &audio_buffer, sample_rate)?;
    Ok(())
}

/// 写入 WAV 文件
fn write_wav_file(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path).map_err(|e| e.to_string())?;

    let num_samples = samples.len() / 2;
    let num_channels = 2u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = (num_samples * num_channels as usize * bits_per_sample as usize / 8) as u32;

    file.write_all(b"RIFF").map_err(|e| e.to_string())?;
    file.write_all(&(36 + data_size).to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"WAVE").map_err(|e| e.to_string())?;
    file.write_all(b"fmt ").map_err(|e| e.to_string())?;
    file.write_all(&16u32.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&1u16.to_le_bytes()).map_err(|e| e.to_string())?; // PCM
    file.write_all(&num_channels.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&sample_rate.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&byte_rate.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&block_align.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&bits_per_sample.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"data").map_err(|e| e.to_string())?;
    file.write_all(&data_size.to_le_bytes()).map_err(|e| e.to_string())?;

    for &sample in samples {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        file.write_all(&sample_i16.to_le_bytes()).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 检查 SoundFont 是否存在
pub fn check_soundfont_exists(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    crate::soundfont_manager::get_soundfont_path(app)
}
