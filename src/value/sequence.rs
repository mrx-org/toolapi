use std::fmt::Debug;

use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// An MRI sequence that consists of a linear series of instantaneous events.
///
/// This type of sequence is very useful for simple simulation and analysis tools.
/// However, some sequence details like pulse durations and other dynamic effects are lost.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventSeq(pub Vec<Event>);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Event {
    Pulse { angle: f64, phase: f64 },
    Fid { kt: [f64; 4] },
    Adc { phase: f64 },
}

impl From<BlockSeq> for EventSeq {
    fn from(seq: BlockSeq) -> Self {
        Self(
            seq.0
                .iter()
                .flat_map(|block| match block {
                    Block {
                        rf: Some(rf),
                        adc: None,
                        ..
                    } => convert_rf(rf, block),
                    Block {
                        rf: None,
                        adc: Some(adc),
                        ..
                    } => convert_adc(adc, block),
                    Block {
                        rf: None,
                        adc: None,
                        ..
                    } => convert_spoiler(block),
                    Block {
                        rf: Some(_),
                        adc: Some(_),
                        ..
                    } => panic!("not supported: cannot specify rf and adc in same block"),
                })
                .collect(),
        )
    }
}

// ========================================
// Internal helpers for sequence conversion
// ========================================

fn convert_rf(rf: &Pulse, block: &Block) -> Vec<Event> {
    let t_center = rf.delay + rf.duration / 2.0;
    let duration = block.calc_duration();

    let (gx1, gx2) = split_gradm(&block.gx, t_center);
    let (gy1, gy2) = split_gradm(&block.gy, t_center);
    let (gz1, gz2) = split_gradm(&block.gz, t_center);

    vec![
        Event::Fid {
            kt: [gx1, gy1, gz1, t_center],
        },
        Event::Pulse {
            angle: rf.flip_angle,
            phase: rf.phase_offset,
        },
        Event::Fid {
            kt: [gx2, gy2, gz2, duration - t_center],
        },
    ]
}

fn convert_adc(adc: &Adc, block: &Block) -> Vec<Event> {
    let time = (0..adc.sample_count).map(|t| adc.delay + (t as f64 + 0.5) * adc.dwell_time);
    let time: Vec<f64> = time.chain(std::iter::once(block.calc_duration())).collect();

    fn integrate(grad: &Gradient, time: f64) -> f64 {
        let Gradient::Trap(grad) = grad;
        integrate_grad(grad, time).0
    }

    let traj: Vec<_> = time
        .iter()
        .map(|&t| {
            [
                block.gx.as_ref().map_or(0.0, |g| integrate(g, t)),
                block.gy.as_ref().map_or(0.0, |g| integrate(g, t)),
                block.gz.as_ref().map_or(0.0, |g| integrate(g, t)),
                t,
            ]
        })
        .collect();

    traj.iter()
        .scan([0.0; 4], |state, &gradm| {
            let kt = [
                gradm[0] - state[0],
                gradm[1] - state[1],
                gradm[2] - state[2],
                gradm[3] - state[3],
            ];
            *state = gradm;
            Some([
                Event::Adc {
                    phase: adc.phase_offset + state[3] * adc.frequency_offset,
                },
                Event::Fid { kt },
            ])
        })
        .flatten()
        .skip(1)
        .collect()
}

fn convert_spoiler(block: &Block) -> Vec<Event> {
    let gx = block.gx.as_ref().map_or(0.0, |g| g.area());
    let gy = block.gy.as_ref().map_or(0.0, |g| g.area());
    let gz = block.gz.as_ref().map_or(0.0, |g| g.area());
    let duration = block.calc_duration();
    vec![Event::Fid {
        kt: [gx, gy, gz, duration],
    }]
}

// Helpers

fn split_gradm(grad: &Option<Gradient>, time: f64) -> (f64, f64) {
    if let Some(grad) = grad {
        let Gradient::Trap(grad) = grad;
        integrate_grad(grad, time)
    } else {
        (0.0, 0.0)
    }
}

/// Return the area under the gradient from start to time and time to end
fn integrate_grad(grad: &TrapGradient, time: f64) -> (f64, f64) {
    let mut time = time;
    let area = grad.area();

    // time is before start of trap
    if time <= grad.delay {
        return (0.0, area);
    }

    time -= grad.delay;
    // time is during ramp-up
    if time < grad.rise_time {
        let tmp = 0.5 * grad.amplitude * (time / grad.rise_time) * time;
        return (tmp, area - tmp);
    }

    time -= grad.rise_time;
    // time is during flat area
    if time < grad.flat_time {
        let ramp_area = 0.5 * grad.amplitude * grad.rise_time;
        let tmp = ramp_area + grad.amplitude * time;
        return (tmp, area - tmp);
    }

    time -= grad.fall_time;
    // time is during ramp-down
    if time < grad.fall_time {
        let t_rev = grad.fall_time - time;
        let missing_area = 0.5 * grad.amplitude * (t_rev / grad.fall_time) * t_rev;
        return (area - missing_area, missing_area);
    }

    // time is after gradient
    (area, 0.0)
}

/// An MRI sequence which consists of a series of blocks, each of which can have multiple, ongoing events.
///
/// This is on modelled very close to Pulseq, to make it easy to construct it from those sequences.
/// It captures most details that are present at the scanner,
/// giving an accurate description of slew-rates, pulse shapes and more.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlockSeq(pub Vec<Block>);

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Block {
    /// Minimum duration of this block, can be longer if events exceed this
    pub min_duration: f64,
    pub rf: Option<Pulse>,
    pub gx: Option<Gradient>,
    pub gy: Option<Gradient>,
    pub gz: Option<Gradient>,
    pub adc: Option<Adc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pulse {
    // Total duration is the sum of these three:
    pub delay: f64,
    pub duration: f64,
    pub ringdown: f64,

    // General pulse properties
    pub flip_angle: f64,
    pub phase_offset: f64,
    pub frequency_offset: f64,

    /// Different pulse types only differ in this:
    pub shape: PulseShape,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PulseShape {
    Sinc {
        time_bandwidth_product: f64,
        apodization: f64,
    },
    Block,
    /// Samples are evenly distributed over the duration of the pulse
    Custom(CustomShape),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomShape(pub Vec<Complex64>);

impl Debug for CustomShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomShape( <{} samples> )", self.0.len())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Gradient {
    Trap(TrapGradient),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TrapGradient {
    pub amplitude: f64,
    pub delay: f64,
    pub rise_time: f64,
    pub flat_time: f64,
    pub fall_time: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Adc {
    pub sample_count: u64,
    pub dwell_time: f64,
    pub delay: f64,
    pub phase_offset: f64,
    pub frequency_offset: f64,
}

// ================
// Useful functions
// ================

impl Gradient {
    pub fn area(&self) -> f64 {
        let Self::Trap(grad) = self;
        grad.area()
    }
}

impl TrapGradient {
    pub fn area(&self) -> f64 {
        self.amplitude * (0.5 * self.rise_time + self.flat_time + 0.5 * self.fall_time)
    }
}

pub trait Duration {
    fn calc_duration(&self) -> f64;
}

impl<T: Duration> Duration for Option<T> {
    fn calc_duration(&self) -> f64 {
        self.as_ref().map_or(0.0, |inner| inner.calc_duration())
    }
}

impl Duration for Pulse {
    fn calc_duration(&self) -> f64 {
        self.delay + self.duration + self.ringdown
    }
}

impl Duration for Gradient {
    fn calc_duration(&self) -> f64 {
        match self {
            Gradient::Trap(g) => g.delay + g.rise_time + g.flat_time + g.fall_time,
        }
    }
}

impl Duration for Adc {
    fn calc_duration(&self) -> f64 {
        self.delay + self.sample_count as f64 * self.dwell_time
    }
}

impl Duration for Block {
    fn calc_duration(&self) -> f64 {
        [
            self.min_duration,
            self.rf.calc_duration(),
            self.gx.calc_duration(),
            self.gy.calc_duration(),
            self.gz.calc_duration(),
            self.adc.calc_duration(),
        ]
        .into_iter()
        .max_by(|x, y| x.total_cmp(y))
        .unwrap_or(0.0)
    }
}
