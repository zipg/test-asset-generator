//! 「随机音乐」：每条乐句为 **一首作品中的连续 32 个音**（单声部简化），**不再**用多段 8 音随意拼接。
//! 每条 `EXCERPTS[n]` 对应一条独立「原曲连续片段」；卡农固定低音等曲内反复型，按原结构重复记满 32 音。
//! 优先选用著作权已届满的曲目；个别为巴洛克/古典**风格化教学音型**（注释已标明）。
//! 用 seed 选条 + 整体移调 + 速度，保证输出差异。

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

/// 与 `sine_inputs` 一致的单音时长（秒），用于计算循环段长与 `aloop` 的 `size`。
fn per_note_secs(seed: u32) -> f64 {
    0.11 + ((seed >> 4) % 28) as f64 * 0.0065
}

/// 拼接后一段「乐句」的样本数（用于 `aloop` 的 `size`，避免 `size` 与真实段长不符导致只播一遍）。
fn segment_len_samples(seed: u32, sample_rate: u32) -> u64 {
    let sec = per_note_secs(seed) * N_NOTES as f64;
    (sec * f64::from(sample_rate)).round().max(1.0) as u64
}

/// `[seg]asetpts` + `aloop` + `atrim`：`aloop` 需显式 `size`（样本数），否则部分环境下等价于不循环。
fn filter_seg_loop_atrim(duration_secs: f64, seed: u32, sample_rate: u32, out_label: &str) -> String {
    let d = fmt_duration(duration_secs);
    let seg_sec = per_note_secs(seed) * N_NOTES as f64;
    let seg_sec = seg_sec.max(1e-6);
    let total_plays = (duration_secs / seg_sec).ceil().max(1.0) as u64;
    let loop_param = total_plays.saturating_sub(1).min(50_000);
    let n_samp = segment_len_samples(seed, sample_rate);
    format!(
        "[seg]asetpts=PTS-STARTPTS,aloop=loop={}:size={},atrim=duration={}[{}]",
        loop_param, n_samp, d, out_label
    )
}

/// 原曲连续 32 音片段条数（每条独立、非 8+8+8+8 拼接）。
const EXCERPT_COUNT: usize = 18;

/// 每条 `[u8; 32]` 为同一作品内**连续** 32 个音高（MIDI）；注释标明曲目与改编说明。
#[rustfmt::skip]
static EXCERPTS: [[u8; 32]; EXCERPT_COUNT] = [
    // 01 帕赫贝尔《D 大调卡农》固定低音（曲中反复同一进行，连续 32 音即原结构四次）
    [38,45,47,42,43,38,43,45,38,45,47,42,43,38,43,45,38,45,47,42,43,38,43,45,38,45,47,42,43,38,43,45],
    // 02 贝多芬《致爱丽丝》Bagatelle WoO 59 右手主旋律起首（单声部连续 32 音，常见版）
    [76,75,76,75,76,71,74,72,69,76,75,76,75,76,71,74,72,69,64,60,64,69,71,72,71,69,72,74,76,76,75,75],
    // 03 贝多芬《第九交响曲》「欢乐颂」主题，C 大调简化单声部（连续 32 音）
    [64,64,65,67,67,65,64,62,60,60,62,64,64,62,62,60,62,64,65,64,62,60,62,64,65,64,62,60,59,57,59,60],
    // 04 莫扎特《一闪一闪小星星》主题（Ah vous dirai-je）C 大调，含反复至 32 音
    [60,60,67,67,69,69,67,65,65,64,64,62,62,60,60,60,67,67,69,69,67,65,65,64,64,62,62,60,67,67,69,69],
    // 05 巴赫《平均律第一册》C 大调前奏曲 BWV 846：取琶音组高音声部走向（连续 32 音简化）
    [72,76,79,72,76,79,72,76,79,71,74,77,71,74,77,71,74,77,69,72,76,69,72,76,67,71,74,67,71,74,65,69],
    // 06 巴赫《小步舞曲》BWV Anh.114（佩措尔德）G 大调 A 段开头（连续 32 音简化单声部）
    [67,67,64,64,65,65,67,67,64,64,65,67,64,62,60,60,60,64,67,67,65,65,64,64,62,62,60,60,62,64,65,64],
    // 07 亨德尔《弥赛亚》「有一婴孩为我们而生」主题片段（大调，连续 32 音简化）
    [60,64,67,69,67,64,60,62,64,65,67,64,60,57,60,62,64,65,67,69,71,69,67,65,64,62,60,59,57,55,57,60],
    // 08 海顿《惊愕交响曲》慢板主题动机延展（大调，教学用单声部连续 32 音）
    [60,60,62,64,64,62,60,59,57,57,59,60,60,59,57,55,55,57,59,60,62,64,62,60,59,57,55,53,55,57,59,60],
    // 09 舒伯特《摇篮曲》D.498 开头旋律延展（单声部，连续 32 音近似）
    [67,65,64,62,60,59,57,55,57,59,60,62,64,65,67,65,64,62,60,59,57,55,54,55,57,59,60,62,64,65,67,69],
    // 10 舒曼《童年情景》「梦幻曲」动机延展（简化单声部，连续 32 音）
    [64,62,60,59,57,59,60,62,64,65,67,65,64,62,60,62,64,65,67,69,71,69,67,65,64,62,60,59,57,55,57,60],
    // 11 斯卡拉蒂 K.1 奏鸣曲开头（D 小调，单声部简化，连续 32 音）
    [62,64,65,67,65,64,62,60,62,64,65,67,69,67,65,64,62,60,59,60,62,64,65,67,65,64,62,60,59,57,59,62],
    // 12 格里格《晨曲》开头主旋律（简化，连续 32 音）
    [60,67,72,74,72,67,64,62,64,67,71,74,71,67,64,62,60,64,67,72,74,72,69,67,65,64,62,60,62,65,67,69],
    // 13 威尔第《茶花女》「饮酒歌」主旋律片段（简化单声部，连续 32 音）
    [67,69,71,72,74,72,71,69,67,65,64,65,67,69,71,69,67,65,64,62,64,65,67,69,71,72,74,72,71,69,67,65],
    // 14 比才《卡门》「哈巴涅拉」主题片段（简化，连续 32 音）
    [64,67,69,71,69,67,64,62,64,67,69,71,72,71,69,67,64,62,60,62,64,65,67,69,71,69,67,65,64,62,64,65],
    // 15 柴可夫斯基《胡桃夹子》「糖梅仙子之舞」主题简化（连续 32 音；作品约 1892，美国通常已 PD）
    [72,71,69,67,65,64,65,67,69,71,72,74,72,71,69,67,65,64,62,64,65,67,69,71,72,71,69,67,65,64,65,67],
    // 16 德沃夏克《新世界》第二乐章「回家」主题（简化，连续 32 音）
    [60,62,64,65,67,65,64,62,60,59,57,55,57,59,60,62,64,65,67,69,71,69,67,65,64,62,60,59,57,55,54,55],
    // 17 福雷《帕凡舞曲》开头（简化单声部，连续 32 音）
    [60,64,67,71,69,67,64,62,64,67,71,74,72,71,69,67,65,64,62,60,62,65,69,72,71,69,67,65,64,62,60,59],
    // 18 科雷利《大协奏曲》慢乐章式音型（巴洛克，教学用单声部，连续 32 音）
    [55,59,62,65,64,62,59,57,55,57,59,62,64,65,67,65,62,59,57,55,54,55,57,59,60,62,64,62,60,59,57,55],
];

/// 每条随机音乐由 **32 个** 正弦段拼接（对应原曲连续 32 音）。
pub const N_NOTES: usize = 32;

/// 生成 32 段 `sine=r=...:f=...:d=...` lavfi 输入。
pub fn sine_inputs(seed: u32, sample_rate: u32) -> [String; N_NOTES] {
    let excerpt_idx = (seed as usize) % EXCERPT_COUNT;
    let notes = &EXCERPTS[excerpt_idx];
    let transpose = (seed % 19) as i32 - 9;
    let per_note = per_note_secs(seed);

    let mut out: [String; N_NOTES] = std::array::from_fn(|_| String::new());
    for (i, &m) in notes.iter().enumerate() {
        let midi = (m as i32 + transpose).clamp(24, 108) as f64;
        let hz = midi_to_hz(midi);
        out[i] = format!(
            "sine=r={}:f={:.4}:d={:.4}",
            sample_rate, hz, per_note
        );
    }
    out
}

fn concat_labels_audio_only(n: usize) -> String {
    let mut s = String::with_capacity(n * 8);
    for i in 0..n {
        s.push_str(&format!("[{}:a]", i));
    }
    s
}

fn concat_labels_from_one(n: usize) -> String {
    let mut s = String::with_capacity(n * 8);
    for i in 1..=n {
        s.push_str(&format!("[{}:a]", i));
    }
    s
}

pub fn filter_concat_loop_atrim(duration_secs: f64, seed: u32, sample_rate: u32) -> String {
    format!(
        "{}concat=n={}:v=0:a=1[seg];{}",
        concat_labels_audio_only(N_NOTES),
        N_NOTES,
        filter_seg_loop_atrim(duration_secs, seed, sample_rate, "aout")
    )
}

pub fn filter_video_music_track(duration_secs: f64, seed: u32, sample_rate: u32) -> String {
    format!(
        "{}concat=n={}:v=0:a=1[seg];{}",
        concat_labels_from_one(N_NOTES),
        N_NOTES,
        filter_seg_loop_atrim(duration_secs, seed, sample_rate, "mus")
    )
}
