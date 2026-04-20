//! 「随机音乐」：多条 8 拍循环动机（含类卡农低音线条），用 seed 控制移调与音符速度，保证输出不同。

fn fmt_duration(secs: f64) -> String {
    if secs == secs.floor() {
        format!("{:.0}", secs)
    } else {
        format!("{:.2}", secs)
    }
}

fn midi_to_hz(m: f64) -> f64 {
    440.0 * (2.0_f64).powf((m - 69.0) / 12.0)
}

/// 多条 8 音符公有领域式动机（帕赫贝尔卡农常用低音走向、分解和弦等），循环播放。
static PHRASES: &[&[u8]] = &[
    &[50, 57, 54, 51, 55, 52, 50, 57],
    &[48, 55, 52, 49, 53, 50, 48, 55],
    &[60, 64, 67, 72, 67, 64, 60, 55],
    &[52, 55, 59, 52, 55, 60, 52, 55],
    &[55, 59, 62, 65, 62, 59, 55, 52],
    &[47, 54, 51, 48, 52, 49, 47, 54],
];

const N_NOTES: usize = 8;

/// 生成 8 段 `sine=r=...:f=...:d=...` lavfi 输入（与 `filter_concat8` 配套）。
pub fn sine_inputs(seed: u32, sample_rate: u32) -> [String; N_NOTES] {
    let phrase_idx = (seed as usize) % PHRASES.len();
    let phrase = PHRASES[phrase_idx];
    let transpose = (seed % 19) as i32 - 9;
    let per_note = 0.11 + ((seed >> 4) % 28) as f64 * 0.0065;

    let mut out: [String; N_NOTES] = std::array::from_fn(|_| String::new());
    for (i, &m) in phrase.iter().enumerate() {
        let midi = (m as i32 + transpose).clamp(24, 108) as f64;
        let hz = midi_to_hz(midi);
        out[i] = format!(
            "sine=r={}:f={:.4}:d={:.4}",
            sample_rate, hz, per_note
        );
    }
    out
}

/// `[0:a]...[7:a]concat` + 循环至总长 + `atrim`；`n_inputs` 恒为 8。
pub fn filter_concat_loop_atrim(duration_secs: f64) -> String {
    let d = fmt_duration(duration_secs);
    format!(
        "[0:a][1:a][2:a][3:a][4:a][5:a][6:a][7:a]concat=n=8:v=0:a=1[seg];[seg]aloop=loop=-1,atrim=duration={}[aout]",
        d
    )
}

/// 视频混流：输入 0 为视频，1..=8 为 sine；输出 `[mus]`。
pub fn filter_video_music_track(duration_secs: f64) -> String {
    let d = fmt_duration(duration_secs);
    format!(
        "[1:a][2:a][3:a][4:a][5:a][6:a][7:a][8:a]concat=n=8:v=0:a=1[seg];[seg]aloop=loop=-1,atrim=duration={}[mus]",
        d
    )
}
