/// 音符频率数据和旋律模板
/// 使用标准音高：A4 = 440Hz

/// 音符结构：(频率Hz, 时长倍数)
pub type Note = (f32, f32);

/// C大调音阶 (C4-C5)
pub const C_MAJOR_SCALE: &[f32] = &[
    261.63, // C4
    293.66, // D4
    329.63, // E4
    349.23, // F4
    392.00, // G4
    440.00, // A4
    493.88, // B4
    523.25, // C5
];

/// 五声音阶 (C D E G A)
pub const PENTATONIC_SCALE: &[f32] = &[
    261.63, // C4
    293.66, // D4
    329.63, // E4
    392.00, // G4
    440.00, // A4
];

/// 琶音模板：C大调三和弦 (C E G C)
pub fn get_arpeggio_pattern() -> Vec<Note> {
    vec![
        (261.63, 1.0), // C4
        (329.63, 1.0), // E4
        (392.00, 1.0), // G4
        (523.25, 1.0), // C5
        (392.00, 1.0), // G4
        (329.63, 1.0), // E4
        (261.63, 2.0), // C4 (长音)
    ]
}

/// 民谣风格旋律：简单的 8 小节旋律
pub fn get_folk_melody() -> Vec<Note> {
    vec![
        (261.63, 1.0), // C
        (261.63, 1.0), // C
        (293.66, 1.0), // D
        (329.63, 1.0), // E
        (329.63, 1.0), // E
        (293.66, 1.0), // D
        (261.63, 2.0), // C (长音)
        (329.63, 1.0), // E
        (329.63, 1.0), // E
        (349.23, 1.0), // F
        (392.00, 2.0), // G (长音)
        (392.00, 1.0), // G
        (349.23, 1.0), // F
        (329.63, 1.0), // E
        (293.66, 1.0), // D
        (261.63, 2.0), // C (长音)
    ]
}

/// 根据模板名称获取旋律
pub fn get_melody_by_template(template: &str) -> Vec<Note> {
    match template {
        "scale" => {
            // 上行音阶 + 下行音阶
            let mut notes = Vec::new();
            for &freq in C_MAJOR_SCALE {
                notes.push((freq, 1.0));
            }
            for &freq in C_MAJOR_SCALE.iter().rev().skip(1) {
                notes.push((freq, 1.0));
            }
            notes
        }
        "arpeggio" => get_arpeggio_pattern(),
        "folk" => get_folk_melody(),
        "random" => {
            // 随机从五声音阶选择 16 个音符
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let mut notes = Vec::new();
            for _ in 0..16 {
                let freq = PENTATONIC_SCALE[rng.gen_range(0..PENTATONIC_SCALE.len())];
                let duration = if rng.gen_bool(0.2) { 2.0 } else { 1.0 };
                notes.push((freq, duration));
            }
            notes
        }
        _ => get_melody_by_template("scale"), // 默认返回音阶
    }
}

/// 移调：将旋律整体升高或降低若干半音
pub fn transpose(notes: &[(f32, f32)], semitones: i32) -> Vec<Note> {
    let ratio = 2_f32.powf(semitones as f32 / 12.0);
    notes.iter().map(|&(freq, dur)| (freq * ratio, dur)).collect()
}
