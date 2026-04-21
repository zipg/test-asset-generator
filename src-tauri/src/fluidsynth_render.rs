/// FluidSynth 音频渲染模块
/// 使用 oxisynth (纯 Rust 实现的 FluidSynth)

use std::path::Path;
use oxisynth::{SoundFont, Synth, MidiEvent};
use crate::melody::Note;

/// 将音符频率转换为 MIDI 音符编号
fn freq_to_midi_note(freq: f32) -> u8 {
    // A4 = 440Hz = MIDI 69
    let midi_note = 69.0 + 12.0 * (freq / 440.0).log2();
    midi_note.round().clamp(0.0, 127.0) as u8
}

/// 使用 FluidSynth 渲染音符序列为音频
pub fn render_with_fluidsynth(
    notes: &[(f32, f32)],
    bpm: u32,
    duration: f64,
    soundfont_path: &Path,
    output_path: &Path,
    sample_rate: u32,
) -> Result<(), String> {
    // 创建合成器
    let mut synth = Synth::new(oxisynth::SynthDescriptor {
        sample_rate: sample_rate as f32,
        ..Default::default()
    }).map_err(|e| format!("Failed to create synth: {:?}", e))?;

    // 加载 SoundFont
    let mut file = std::fs::File::open(soundfont_path)
        .map_err(|e| format!("Failed to open soundfont: {:?}", e))?;
    let soundfont = SoundFont::load(&mut file)
        .map_err(|e| format!("Failed to load soundfont: {:?}", e))?;

    synth.add_font(soundfont, true);

    // 计算每拍的秒数
    let beat_duration = 60.0 / bpm as f64;

    // 计算总拍数并调整以匹配目标时长
    let total_beats: f32 = notes.iter().map(|(_, dur)| dur).sum();
    let scale_factor = duration / (total_beats as f64 * beat_duration);

    // 生成 MIDI 事件
    let mut current_time = 0.0;
    let mut events = Vec::new();

    for (freq, note_duration) in notes {
        let midi_note = freq_to_midi_note(*freq);
        let duration_secs = (*note_duration as f64 * beat_duration * scale_factor) as f32;

        // Note On
        events.push((current_time, MidiEvent::NoteOn {
            channel: 0,
            key: midi_note,
            vel: 80,
        }));

        // Note Off
        events.push((current_time + duration_secs, MidiEvent::NoteOff {
            channel: 0,
            key: midi_note,
        }));

        current_time += duration_secs;
    }

    // 排序事件
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 渲染音频
    let total_samples = (duration * sample_rate as f64) as usize;
    let mut audio_buffer = vec![0.0f32; total_samples * 2]; // 立体声

    let mut event_idx = 0;
    let samples_per_sec = sample_rate as f32;

    for sample_idx in 0..total_samples {
        let current_time_sec = sample_idx as f32 / samples_per_sec;

        // 处理当前时间点的所有事件
        while event_idx < events.len() && events[event_idx].0 <= current_time_sec {
            synth.send_event(events[event_idx].1.clone())
                .map_err(|e| format!("Failed to send event: {:?}", e))?;
            event_idx += 1;
        }

        // 渲染一个样本（立体声）
        let mut stereo_buffer = [0.0f32, 0.0f32];
        synth.write(&mut stereo_buffer[..]);

        audio_buffer[sample_idx * 2] = stereo_buffer[0];
        audio_buffer[sample_idx * 2 + 1] = stereo_buffer[1];
    }

    // 写入 WAV 文件
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

    // WAV header
    file.write_all(b"RIFF").map_err(|e| e.to_string())?;
    file.write_all(&(36 + data_size).to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"WAVE").map_err(|e| e.to_string())?;

    // fmt chunk
    file.write_all(b"fmt ").map_err(|e| e.to_string())?;
    file.write_all(&16u32.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&1u16.to_le_bytes()).map_err(|e| e.to_string())?; // PCM
    file.write_all(&num_channels.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&sample_rate.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&byte_rate.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&block_align.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&bits_per_sample.to_le_bytes()).map_err(|e| e.to_string())?;

    // data chunk
    file.write_all(b"data").map_err(|e| e.to_string())?;
    file.write_all(&data_size.to_le_bytes()).map_err(|e| e.to_string())?;

    // 写入样本数据
    for &sample in samples {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        file.write_all(&sample_i16.to_le_bytes()).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 检查 SoundFont 是否存在
pub fn check_soundfont_exists() -> Option<std::path::PathBuf> {
    crate::soundfont_manager::get_soundfont_path()
}
