//! Signal generation for EXP-011 sense-making experiments.

/// Signal type for environment modulation.
#[derive(Debug, Clone)]
pub enum SignalType {
    SquareWave { period: u64, amplitude: f32 },
    SineWave { period: u64, amplitude: f32 },
    Random { seed: u64 },
    None,
}

impl SignalType {
    /// Generate signal value at given tick. Returns 0.0-1.0.
    pub fn value_at(&self, tick: u64) -> f32 {
        match self {
            SignalType::SquareWave { period, amplitude } => {
                let phase = (tick % period) as f32 / *period as f32;
                if phase < 0.5 { *amplitude } else { 0.0 }
            }
            SignalType::SineWave { period, amplitude } => {
                let phase = (tick % period) as f32 / *period as f32;
                let val = (phase * 2.0 * std::f32::consts::PI).sin();
                ((val + 1.0) / 2.0) * amplitude // normalize to 0..amplitude
            }
            SignalType::Random { seed } => {
                // Simple hash-based pseudo-random, changes every 100 ticks
                let block = tick / 100;
                let hash = (block.wrapping_mul(6364136223846793005).wrapping_add(*seed)) >> 33;
                (hash % 100) as f32 / 100.0
            }
            SignalType::None => 0.5, // constant
        }
    }
}

/// Experiment group configuration for EXP-011.
#[derive(Debug, Clone)]
pub struct SenseMakingGroup {
    pub name: String,
    pub signal: SignalType,
    pub delta: u64,      // ticks between signal change and food change
    pub use_real_cpu: bool,
}

impl SenseMakingGroup {
    pub fn group_a() -> Self {
        SenseMakingGroup {
            name: "A_square_d200".into(),
            signal: SignalType::SquareWave { period: 2000, amplitude: 1.0 },
            delta: 200,
            use_real_cpu: false,
        }
    }
    pub fn group_b() -> Self {
        SenseMakingGroup {
            name: "B_sine_d200".into(),
            signal: SignalType::SineWave { period: 2000, amplitude: 1.0 },
            delta: 200,
            use_real_cpu: false,
        }
    }
    pub fn group_c() -> Self {
        SenseMakingGroup {
            name: "C_realcpu_d200".into(),
            signal: SignalType::None, // placeholder, real CPU sampled at runtime
            delta: 200,
            use_real_cpu: true,
        }
    }
    pub fn group_d() -> Self {
        SenseMakingGroup {
            name: "D_square_d0".into(),
            signal: SignalType::SquareWave { period: 2000, amplitude: 1.0 },
            delta: 0, // synchronous — no prediction advantage
            use_real_cpu: false,
        }
    }
    pub fn group_e() -> Self {
        SenseMakingGroup {
            name: "E_random_d200".into(),
            signal: SignalType::Random { seed: 7777 },
            delta: 200,
            use_real_cpu: false,
        }
    }
    pub fn group_f() -> Self {
        SenseMakingGroup {
            name: "F_nosignal".into(),
            signal: SignalType::None,
            delta: 0,
            use_real_cpu: false,
        }
    }

    pub fn all_groups() -> Vec<Self> {
        vec![
            Self::group_a(), Self::group_b(), Self::group_c(),
            Self::group_d(), Self::group_e(), Self::group_f(),
        ]
    }
}
