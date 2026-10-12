# Data sources

Every file here comes from the AutoEq project by Jaakko Pasanen, MIT licence (full text in `THIRD_PARTY_LICENSES`
at the repository root), fetched on 2026-10-12 from commit `7ae0f56d53074872b028649617a22bbb4232feb7`
(`master`, committed 2025-07-20). The headset measurement was taken by RTINGS.com on a Brüel & Kjær 5128 head and
torso simulator and is part of AutoEq's measurement collection.

| File | Source in AutoEq | Rig | Change |
|---|---|---|---|
| `nova5_rtings_5128.csv` | [measurements/rtings/data/over-ear/Bruel & Kjaer 5128/SteelSeries Arctis Nova 5.csv](https://raw.githubusercontent.com/jaakkopasanen/AutoEq/7ae0f56d53074872b028649617a22bbb4232feb7/measurements/rtings/data/over-ear/Bruel%20%26%20Kjaer%205128/SteelSeries%20Arctis%20Nova%205.csv) | B&K 5128 (RTINGS) | resampled |
| `target_lmg_5128_0.6.csv` | [targets/LMG 5128 0.6.csv](https://raw.githubusercontent.com/jaakkopasanen/AutoEq/7ae0f56d53074872b028649617a22bbb4232feb7/targets/LMG%205128%200.6.csv) | B&K 5128 | resampled |
| `target_df_5128.csv` | [targets/Diffuse field 5128.csv](https://raw.githubusercontent.com/jaakkopasanen/AutoEq/7ae0f56d53074872b028649617a22bbb4232feb7/targets/Diffuse%20field%205128.csv) | B&K 5128 | resampled |
| `nova5_autoeq_parametric_eq.txt` | [results/Rtings/Bruel & Kjaer 5128 over-ear/SteelSeries Arctis Nova 5/SteelSeries Arctis Nova 5 ParametricEQ.txt](https://raw.githubusercontent.com/jaakkopasanen/AutoEq/7ae0f56d53074872b028649617a22bbb4232feb7/results/Rtings/Bruel%20%26%20Kjaer%205128%20over-ear/SteelSeries%20Arctis%20Nova%205/SteelSeries%20Arctis%20Nova%205%20ParametricEQ.txt) | AutoEq fit of the measurement above to LMG 5128 0.6 | none |

SHA-256 of the files as fetched:

| Source | SHA-256 |
|---|---|
| `SteelSeries Arctis Nova 5.csv` | `6a72bf04b9de6675ac06eb4f27e95e5b275e2395aa10d1eb8e43623ab0aab295` |
| `LMG 5128 0.6.csv` | `d4207b122b56826250689da48f948f830c2d92f2b451a46df185b5fce12b786d` |
| `Diffuse field 5128.csv` | `f906951f4ef7229fee579ca0bb574eec4aeea93f69e1e224ca4f94461836d361` |
| `SteelSeries Arctis Nova 5 ParametricEQ.txt` | `923c508f815c373a6431199c5651da24699c9eb97f580c92c87b314ca028f39d` |

## Resampling

The source CSVs (`frequency,raw`, 695 points, about 1/96 octave) were resampled to 1/24 octave: 240 points at
`20 · 2^(k/24)` Hz for k = 0..239 plus 20000 Hz, each the linear interpolation of the source on a logarithmic
frequency axis, written as `freq_hz,spl_db` with two decimals. Levels are as published; the app aligns them at run
time (`crates/core/src/eq/frequency_response.rs::FrequencyResponse::normalised`).

## Use

The app shows the measurement and both targets in the equalizer graph
(`crates/core/src/eq/reference.rs::plot`), and the built-in preset "Harman target"
(`crates/core/src/eq/library.rs::library`) is the parametric EQ above rounded to the headset's grid.
