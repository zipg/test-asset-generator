/// FluidSynth 音频渲染模块
/// 使用 oxisynth (纯 Rust 实现的 FluidSynth)
/// 支持多种音色、鼓点、音量增益、多乐器和声、循环播放

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

/// 挑选一个与主乐器不同且搭配和谐的副乐器
fn pick_harmony_instrument(main: u8) -> u8 {
    // 按乐器家族分组，跨家族搭配更丰富
    let mut rng = rand::thread_rng();
    let candidates: Vec<u8> = GM_INSTRUMENTS.iter()
        .map(|(id, _)| *id)
        .filter(|id| *id != main)
        .collect();
    if candidates.is_empty() { return 0; }
    // 偏向选择不同家族的乐器
    if rng.gen_bool(0.7) {
        // 优先选择相距较远的乐器（不同家族）
        let far: Vec<u8> = candidates.iter()
            .filter(|id| (*id / 8) != (main / 8))
            .copied()
            .collect();
        if !far.is_empty() {
            return far[rng.gen_range(0..far.len())];
        }
    }
    candidates[rng.gen_range(0..candidates.len())]
}

/// 生成更丰富的鼓点模式（多种鼓音色随机化）
fn generate_drum_pattern(bpm: u32, bars: u32) -> Vec<(f32, u8, u8)> {
    let beat_duration = 60.0 / bpm as f32;
    let mut drums = Vec::new();
    let mut rng = rand::thread_rng();

    for bar in 0..bars {
        let bar_start = bar as f32 * 4.0 * beat_duration;
        for beat in 0..4 {
            let beat_time = bar_start + beat as f32 * beat_duration;

            // Bass Drum: 强拍/次强拍力度大，弱拍力度小
            let bd_note = if rng.gen_bool(0.4) { 35 } else { 36 }; // Acoustic 或 Bass Drum 1
            if beat == 0 || beat == 2 {
                drums.push((beat_time, bd_note, 80 + rng.gen_range(0..15)));
            } else if rng.gen_bool(0.3) {
                drums.push((beat_time, bd_note, 40 + rng.gen_range(0..20)));
            }

            // Snare: 第2、4拍
            let sn_note = if rng.gen_bool(0.3) { 40 } else { 38 }; // Electric 或 Acoustic Snare
            if beat == 1 || beat == 3 {
                drums.push((beat_time, sn_note, 70 + rng.gen_range(0..20)));
            }

            // Hi-Hat / Ride: 八分音符基础节奏
            let hh_note = if rng.gen_bool(0.25) { 51 } else { 42 }; // Ride Cymbal 1 或 Closed HH
            drums.push((beat_time, hh_note, 40 + rng.gen_range(0..25)));
            drums.push((beat_time + 0.5 * beat_duration, hh_note, 30 + rng.gen_range(0..20)));

            // Open HH 在偶数小节末尾
            if bar % 2 == 1 && beat == 3 {
                drums.push((beat_time + 0.5 * beat_duration, 46, 55 + rng.gen_range(0..15)));
            }

            // 每小节第1拍加 Crash Cymbal 点缀（概率）
            if beat == 0 && rng.gen_bool(0.3) {
                let crash = if rng.gen_bool(0.5) { 49 } else { 57 }; // Crash 1 或 Crash 2
                drums.push((beat_time, crash, 50 + rng.gen_range(0..20)));
            }

            // 随机加 Toms（概率低）
            if rng.gen_bool(0.15) {
                let tom = [41, 43, 45, 47, 48, 50][rng.gen_range(0..6)];
                drums.push((beat_time, tom, 45 + rng.gen_range(0..25)));
            }

            // 偶尔加打击乐点缀
            if rng.gen_bool(0.08) {
                let perc = [39, 54, 56, 62, 63][rng.gen_range(0..5)]; // Hand Clap, Tambourine, Cowbell, Congas
                drums.push((beat_time, perc, 35 + rng.gen_range(0..20)));
            }
        }

        // 小节末尾偶尔加 Fill（几个 Toms 连击）
        if rng.gen_bool(0.12) {
            let fill_start = bar_start + 3.0 * beat_duration;
            let toms: Vec<u8> = vec![50, 48, 45, 43, 41];
            for (j, tom) in toms.iter().enumerate() {
                drums.push((fill_start + j as f32 * 0.15, *tom, 50 + rng.gen_range(0..20)));
            }
        }
    }
    drums.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    drums
}

/// 使用 FluidSynth 渲染，支持多乐器和声 + 丰富鼓点 + 循环 + 增益
pub fn render_with_fluidsynth(
    notes: &[(f32, f32)],
    bpm: u32,
    duration: f64,
    soundfont_path: &Path,
    output_path: &Path,
    sample_rate: u32,
    instrument: u8,
    enable_drums: bool,
    enable_harmony: bool,
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

    let mut rng = rand::thread_rng();

    // 生成单次遍历的 MIDI 事件
    let mut one_pass_events: Vec<(f32, MidiEvent)> = Vec::new();

    // ====== MIDI CC 空间与效果设置 (time 0.0) ======

    // Channel 0 — 主旋律: 偏左, 清晰, 适度混响+合唱
    one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 0, ctrl: 7, value: 110 }));   // Volume
    one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 0, ctrl: 10, value: 40 }));   // Pan: 偏左
    one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 0, ctrl: 91, value: 55 }));   // Reverb
    one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 0, ctrl: 93, value: 28 }));   // Chorus
    one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 0, ctrl: 1, value: 22 }));    // Mod: 基础颤音

    // Program Change — Channel 0 主旋律
    one_pass_events.push((0.0, MidiEvent::ProgramChange {
        channel: 0,
        program_id: instrument,
    }));

    // 和声乐器 — Channel 1
    let harmony_instrument = pick_harmony_instrument(instrument);
    if enable_harmony {
        one_pass_events.push((0.0, MidiEvent::ProgramChange {
            channel: 1,
            program_id: harmony_instrument,
        }));
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 1, ctrl: 7, value: 55 }));    // Volume: 明显弱于主旋律
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 1, ctrl: 10, value: 95 }));   // Pan: 偏右
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 1, ctrl: 91, value: 68 }));   // Reverb: 更深
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 1, ctrl: 93, value: 18 }));   // Chorus: 轻
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 1, ctrl: 1, value: 30 }));    // Mod: 稍多颤音
    }

    // 主旋律 (Channel 0) + 和声 (Channel 1)
    let mut t = 0.0;
    for (freq, dur) in notes {
        let midi = freq_to_midi_note(*freq);
        let secs = (*dur as f64 * beat_duration) as f32;
        let vel = 65 + rng.gen_range(0..20);

        // 主旋律
        // 长音符随机加颤音（35%概率），强度随机变化
        let use_vibrato = *dur >= 1.0 && rng.gen_bool(0.35);
        if use_vibrato {
            let vib_strength = 35 + rng.gen_range(0..35); // 35-69 随机强度
            one_pass_events.push((t - 0.005, MidiEvent::ControlChange { channel: 0, ctrl: 1, value: vib_strength }));
        }
        one_pass_events.push((t, MidiEvent::NoteOn { channel: 0, key: midi, vel }));
        one_pass_events.push((t + secs * 0.92, MidiEvent::NoteOff { channel: 0, key: midi }));
        if use_vibrato {
            one_pass_events.push((t + secs * 0.92 + 0.005, MidiEvent::ControlChange { channel: 0, ctrl: 1, value: 22 }));
        }

        if enable_harmony {
            // 三度和声 (Channel 1): 旋律上方 4 个半音（大三度）
            let third = (midi as i16 + 4).clamp(0, 127) as u8;
            let hvel = vel.saturating_sub(10);
            one_pass_events.push((t, MidiEvent::NoteOn { channel: 1, key: third, vel: hvel }));
            one_pass_events.push((t + secs * 0.92, MidiEvent::NoteOff { channel: 1, key: third }));

            // 五度和声 (Channel 1): 旋律上方 7 个半音，只在长音符上加
            if *dur >= 1.0 {
                let fifth = (midi as i16 + 7).clamp(0, 127) as u8;
                let fvel = vel.saturating_sub(15);
                one_pass_events.push((t, MidiEvent::NoteOn { channel: 1, key: fifth, vel: fvel }));
                one_pass_events.push((t + secs * 0.92, MidiEvent::NoteOff { channel: 1, key: fifth }));
            }
        }

        t += secs;
    }

    // ====== 乐器变换: 每8-16小节切换到同类乐器 ======
    let instrument_families: &[(u8, &[u8])] = &[
        // 钢琴类
        (0, &[0, 1, 2, 3, 4, 5, 6, 7, 8]),
        // 吉他类
        (25, &[25, 26, 27, 28, 29, 30]),
        // 弦乐类
        (41, &[41, 42, 43, 44, 45, 47]),
        // 木管类
        (74, &[69, 72, 74, 76, 68, 71, 73]),
        // 铜管类
        (57, &[57, 58, 59, 60, 61, 67, 68]),
        // 打击/色彩类
        (11, &[11, 12, 13, 14, 15, 9, 10, 46]),
    ];
    let family = instrument_families.iter()
        .find(|(_, members)| members.contains(&instrument))
        .map(|(_, members)| *members)
        .unwrap_or(&[]);
    if !family.is_empty() {
        let bars_in_pass = ((one_pass_time / (4.0 * beat_duration)).ceil() as u32).max(1);
        let switch_interval_bars = 8 + rng.gen_range(0..9); // 每 8-16 小节切换
        let mut switch_times: Vec<(f32, u8)> = Vec::new();
        for bar in (switch_interval_bars..bars_in_pass).step_by(switch_interval_bars as usize) {
            if rng.gen_bool(0.6) { // 60% 概率在此小节切换
                let t = bar as f32 * 4.0 * beat_duration as f32;
                let mut idx = rng.gen_range(0..family.len());
                let mut new_inst = family[idx];
                // 避免切到当前乐器或和声乐器
                let mut attempts = 0;
                while (new_inst == instrument || new_inst == harmony_instrument) && attempts < 10 {
                    idx = rng.gen_range(0..family.len());
                    new_inst = family[idx];
                    attempts += 1;
                }
                if new_inst != instrument && new_inst != harmony_instrument {
                    switch_times.push((t, new_inst));
                }
            }
        }
        for (t, inst) in &switch_times {
            one_pass_events.push((*t, MidiEvent::ProgramChange { channel: 0, program_id: *inst }));
            let new_vol = 100 + rng.gen_range(0..21);
            one_pass_events.push((*t, MidiEvent::ControlChange { channel: 0, ctrl: 7, value: new_vol }));
        }
    }

    // 鼓点 (Channel 9)
    if enable_drums {
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 9, ctrl: 7, value: 105 }));   // Volume
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 9, ctrl: 10, value: 64 }));   // Pan: 居中
        one_pass_events.push((0.0, MidiEvent::ControlChange { channel: 9, ctrl: 91, value: 18 }));   // Reverb: 少量
        let bars = ((one_pass_time / (4.0 * beat_duration)).ceil() as u32).max(1);
        for (time, note, vel) in generate_drum_pattern(bpm, bars) {
            one_pass_events.push((time, MidiEvent::NoteOn { channel: 9, key: note, vel }));
            one_pass_events.push((time + 0.12, MidiEvent::NoteOff { channel: 9, key: note }));
        }
    }

    one_pass_events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 循环生成多轮事件
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
