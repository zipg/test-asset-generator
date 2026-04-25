/// 音符频率数据和旋律模板
///
/// 使用标准音高：A4 = 440Hz
/// 半音关系：每升高一个半音，频率乘以 2^(1/12)

pub type Note = (f32, f32); // (frequency Hz, duration in beats)

// C 大调音阶 (C4-C5)
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

// 五声音阶（中国风）
pub const PENTATONIC_SCALE: &[f32] = &[
    261.63, // C4
    293.66, // D4
    329.63, // E4
    392.00, // G4
    440.00, // A4
    523.25, // C5
];

// 小调音阶 (A minor)
pub const A_MINOR_SCALE: &[f32] = &[
    220.00, // A3
    246.94, // B3
    261.63, // C4
    293.66, // D4
    329.63, // E4
    349.23, // F4
    392.00, // G4
    440.00, // A4
];

/// 琶音模式：C 大调三和弦琶音
pub fn get_arpeggio_pattern() -> Vec<Note> {
    vec![
        (261.63, 0.5), (329.63, 0.5), (392.00, 0.5), (523.25, 0.5),
        (392.00, 0.5), (329.63, 0.5), (261.63, 1.0),
    ]
}

/// 小星星（Twinkle Twinkle Little Star）
pub fn get_twinkle_twinkle() -> Vec<Note> {
    vec![
        (261.63, 0.5), (261.63, 0.5), (392.00, 0.5), (392.00, 0.5),
        (440.00, 0.5), (440.00, 0.5), (392.00, 1.0),
        (349.23, 0.5), (349.23, 0.5), (329.63, 0.5), (329.63, 0.5),
        (293.66, 0.5), (293.66, 0.5), (261.63, 1.0),
        (392.00, 0.5), (392.00, 0.5), (349.23, 0.5), (349.23, 0.5),
        (329.63, 0.5), (329.63, 0.5), (293.66, 1.0),
        (392.00, 0.5), (392.00, 0.5), (349.23, 0.5), (349.23, 0.5),
        (329.63, 0.5), (329.63, 0.5), (293.66, 1.0),
        (261.63, 0.5), (261.63, 0.5), (392.00, 0.5), (392.00, 0.5),
        (440.00, 0.5), (440.00, 0.5), (392.00, 1.0),
        (349.23, 0.5), (349.23, 0.5), (329.63, 0.5), (329.63, 0.5),
        (293.66, 0.5), (293.66, 0.5), (261.63, 1.0),
    ]
}

/// 欢乐颂（Ode to Joy）主题
pub fn get_ode_to_joy() -> Vec<Note> {
    vec![
        (329.63, 0.5), (329.63, 0.5), (349.23, 0.5), (392.00, 0.5),
        (392.00, 0.5), (349.23, 0.5), (329.63, 0.5), (293.66, 0.5),
        (261.63, 0.5), (261.63, 0.5), (293.66, 0.5), (329.63, 0.5),
        (329.63, 0.75), (293.66, 0.25), (293.66, 1.0),
        (329.63, 0.5), (329.63, 0.5), (349.23, 0.5), (392.00, 0.5),
        (392.00, 0.5), (349.23, 0.5), (329.63, 0.5), (293.66, 0.5),
        (261.63, 0.5), (261.63, 0.5), (293.66, 0.5), (329.63, 0.5),
        (293.66, 0.75), (261.63, 0.25), (261.63, 1.0),
    ]
}

/// 卡农（Canon in D）主旋律片段
pub fn get_canon_in_d() -> Vec<Note> {
    vec![
        (349.23, 1.0), (293.66, 1.0), (261.63, 1.0), (293.66, 1.0),
        (329.63, 1.0), (246.94, 1.0), (261.63, 1.0), (392.00, 1.0),
        (349.23, 0.5), (293.66, 0.5), (349.23, 0.5), (392.00, 0.5),
        (349.23, 0.5), (293.66, 0.5), (261.63, 1.0),
        (293.66, 0.5), (329.63, 0.5), (293.66, 0.5), (261.63, 0.5),
        (246.94, 0.5), (220.00, 0.5), (246.94, 1.0),
    ]
}

/// 天空之城（Castle in the Sky）主题
pub fn get_castle_in_sky() -> Vec<Note> {
    vec![
        (440.00, 0.75), (493.88, 0.25), (523.25, 0.5), (493.88, 0.5),
        (523.25, 0.5), (587.33, 1.5),
        (440.00, 0.5), (440.00, 0.5), (392.00, 0.5), (440.00, 0.5),
        (392.00, 2.0),
        (329.63, 0.75), (329.63, 0.25), (392.00, 0.5), (329.63, 0.5),
        (392.00, 0.5), (440.00, 1.5),
        (329.63, 0.5), (329.63, 0.5), (293.66, 0.5), (329.63, 0.5),
        (293.66, 2.0),
    ]
}

/// 茉莉花（Jasmine Flower）中国民歌
pub fn get_jasmine_flower() -> Vec<Note> {
    vec![
        (329.63, 1.0), (392.00, 0.5), (329.63, 0.5),
        (293.66, 1.0), (329.63, 1.0),
        (392.00, 1.0), (329.63, 0.5), (293.66, 0.5),
        (261.63, 2.0),
        (293.66, 1.0), (329.63, 0.5), (293.66, 0.5),
        (261.63, 1.0), (293.66, 1.0),
        (329.63, 1.0), (293.66, 0.5), (261.63, 0.5),
        (220.00, 2.0),
    ]
}

/// 生日快乐歌（Happy Birthday）
pub fn get_happy_birthday() -> Vec<Note> {
    vec![
        (261.63, 0.75), (261.63, 0.25), (293.66, 1.0), (261.63, 1.0),
        (349.23, 1.0), (329.63, 2.0),
        (261.63, 0.75), (261.63, 0.25), (293.66, 1.0), (261.63, 1.0),
        (392.00, 1.0), (349.23, 2.0),
        (261.63, 0.75), (261.63, 0.25), (523.25, 1.0), (440.00, 1.0),
        (349.23, 1.0), (329.63, 1.0), (293.66, 1.0),
        (466.16, 0.75), (466.16, 0.25), (440.00, 1.0), (349.23, 1.0),
        (392.00, 1.0), (349.23, 2.0),
    ]
}

/// 民谣风格旋律
pub fn get_folk_melody() -> Vec<Note> {
    let pattern = vec![
        (261.63, 0.5), (261.63, 0.5), (293.66, 0.5), (329.63, 0.5),
        (329.63, 0.5), (293.66, 0.5), (261.63, 1.0),
        (329.63, 0.5), (329.63, 0.5), (349.23, 0.5), (392.00, 1.0),
        (392.00, 0.5), (349.23, 0.5), (329.63, 0.5), (293.66, 0.5),
        (261.63, 1.0),
    ];
    let mut notes = pattern.clone();
    notes.extend(pattern);
    notes
}

/// 根据模板名称获取旋律
pub fn get_melody_by_template(template: &str) -> Vec<Note> {
    match template {
        "scale" => {
            let mut notes = Vec::new();
            for _ in 0..4 {
                for &freq in C_MAJOR_SCALE {
                    notes.push((freq, 0.5));
                }
                for &freq in C_MAJOR_SCALE.iter().rev().skip(1) {
                    notes.push((freq, 0.5));
                }
            }
            notes
        }
        "arpeggio" => get_arpeggio_pattern(),
        "folk" => get_folk_melody(),
        "twinkle" => get_twinkle_twinkle(),
        "ode_to_joy" => get_ode_to_joy(),
        "canon" => get_canon_in_d(),
        "castle_sky" => get_castle_in_sky(),
        "jasmine" => get_jasmine_flower(),
        "birthday" => get_happy_birthday(),
        "random" => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let mut notes = Vec::new();
            for _ in 0..32 {
                let freq = PENTATONIC_SCALE[rng.gen_range(0..PENTATONIC_SCALE.len())];
                let duration = if rng.gen_bool(0.2) { 1.0 } else { 0.5 };
                notes.push((freq, duration));
            }
            notes
        }
        _ => get_melody_by_template("scale"),
    }
}

/// 移调：将旋律升高或降低指定的半音数
pub fn transpose(notes: &[(f32, f32)], semitones: i32) -> Vec<Note> {
    let ratio = 2_f32.powf(semitones as f32 / 12.0);
    notes.iter().map(|&(freq, dur)| (freq * ratio, dur)).collect()
}

/// 将简短主题扩展为完整 A-B-A 结构的完整乐谱
/// Theme A ×2 → Bridge → Theme B ×2 (移调变奏) → Theme A' ×2 (再现) → Coda
/// 一个 30 音符的主题可扩展为约 250 音符的完整作品
pub fn expand_to_aba(theme: &[Note]) -> Vec<Note> {
    use rand::Rng;
    if theme.is_empty() {
        return theme.to_vec();
    }
    let mut full: Vec<Note> = Vec::new();

    // Intro — 从主题开头放慢引申
    for &(f, d) in theme.iter().take(4) {
        full.push((f, d * 1.5));
    }

    // Theme A — 原主题重复 2 次
    for _ in 0..2 {
        full.extend_from_slice(theme);
    }

    // Bridge — 过渡段（主题结尾反转 + 压缩）
    for &(f, d) in theme.iter().rev().take(6) {
        full.push((f * 1.334, d * 0.75));
    }

    // Theme B — 变奏（移调 + 节奏随机化）
    let mut rng = rand::thread_rng();
    let b_section: Vec<Note> = theme.iter()
        .map(|&(f, d)| {
            let swing: f32 = rng.gen_range(0.75..1.25);
            (f * 1.498, d * swing)
        })
        .collect();
    for _ in 0..2 {
        full.extend(b_section.clone());
    }

    // Theme A' — 再现主题（带装饰）
    let a2: Vec<Note> = theme.iter()
        .enumerate()
        .flat_map(|(i, &(f, d))| {
            let mut n = vec![(f, d)];
            // 每隔几个音加一个经过音
            if i % 3 == 1 {
                n.push((f * 1.122, d * 0.25));
            }
            n
        })
        .collect();
    for _ in 0..2 {
        full.extend(a2.clone());
    }

    // Coda — 尾声（拉长收束）
    for &(f, d) in theme.iter().rev().take(4).collect::<Vec<_>>().iter().rev() {
        full.push((*f, *d * 2.0));
    }
    // 终止音
    if let Some(&(last, _)) = theme.last() {
        full.push((last, 3.0));
    }

    full
}
