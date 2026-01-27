use num_complex::Complex64;
use serde::{Deserialize, Serialize};

// ===========================================================
// TODO: This is from the old toolapi and might need a cleanup
// ===========================================================

/// Threshold used to sparsify maps - below this voxels are treated as empty.
const PD_THRESHOLD: f64 = 1e-6;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TissueProperties {
    pub t1: f64,
    pub t2: f64,
    pub t2dash: f64,
    pub adc: f64,
}

impl From<VoxelPhantom> for TissueProperties {
    fn from(value: VoxelPhantom) -> Self {
        Self {
            t1: value.t1.iter().sum::<f64>() / value.t1.len() as f64,
            t2: value.t2.iter().sum::<f64>() / value.t2.len() as f64,
            t2dash: value.t2dash.iter().sum::<f64>() / value.t2dash.len() as f64,
            adc: value.adc.iter().sum::<f64>() / value.adc.len() as f64,
        }
    }
}

impl From<VoxelGridPhantom> for TissueProperties {
    fn from(value: VoxelGridPhantom) -> Self {
        Self {
            t1: value.t1.iter().sum::<f64>() / value.t1.len() as f64,
            t2: value.t2.iter().sum::<f64>() / value.t2.len() as f64,
            t2dash: value.t2dash.iter().sum::<f64>() / value.t2dash.len() as f64,
            adc: value.adc.iter().sum::<f64>() / value.adc.len() as f64,
        }
    }
}

impl From<MultiTissuePhantom> for VoxelGridPhantom {
    fn from(value: MultiTissuePhantom) -> Self {
        let voxel_count = value.grid_size[0] * value.grid_size[1] * value.grid_size[2];
        let mut pd = vec![0.0; voxel_count];
        let mut b0 = vec![0.0; voxel_count];
        let mut t1 = vec![0.0; voxel_count];
        let mut t2 = vec![0.0; voxel_count];
        let mut t2dash = vec![0.0; voxel_count];
        let mut adc = vec![0.0; voxel_count];

        for tissue in &value.tissues {
            for i in 0..voxel_count {
                pd[i] += tissue.pd[i];
                b0[i] += tissue.pd[i] * tissue.b0[i];
                t1[i] += tissue.pd[i] * tissue.props.t1;
                t2[i] += tissue.pd[i] * tissue.props.t2;
                t2dash[i] += tissue.pd[i] * tissue.props.t2dash;
                adc[i] += tissue.pd[i] * tissue.props.adc;
            }
        }
        // normalize
        for i in 0..voxel_count {
            if pd[i] > 0.0 {
                b0[i] /= pd[i];
                t1[i] /= pd[i];
                t2[i] /= pd[i];
                t2dash[i] /= pd[i];
                adc[i] /= pd[i];
            }
        }

        Self {
            voxel_shape: value.voxel_shape,
            grid_size: value.grid_size,
            grid_spacing: value.grid_spacing,
            b0,
            b1: value.b1,
            coil_sens: value.coil_sens,
            pd,
            t1,
            t2,
            t2dash,
            adc,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTissuePhantom {
    pub voxel_shape: VoxelShape,
    /// space between voxels in grid
    pub grid_spacing: [f64; 3],
    /// number of voxels in each direction in the flattened maps
    pub grid_size: [usize; 3],

    pub tissues: Vec<PhantomTissue>,
    pub b1: Vec<Vec<Complex64>>,
    pub coil_sens: Vec<Vec<Complex64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhantomTissue {
    pub pd: Vec<f64>,
    pub b0: Vec<f64>,
    pub props: TissueProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelGridPhantom {
    pub voxel_shape: VoxelShape,
    /// space between voxels in grid
    pub grid_spacing: [f64; 3],
    /// number of voxels in each direction in the flattened maps
    pub grid_size: [usize; 3],
    pub pd: Vec<f64>,
    pub t1: Vec<f64>,
    pub t2: Vec<f64>,
    pub t2dash: Vec<f64>,
    pub adc: Vec<f64>,
    pub b0: Vec<f64>,
    pub b1: Vec<Vec<Complex64>>,
    pub coil_sens: Vec<Vec<Complex64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelPhantom {
    pub voxel_shape: VoxelShape,
    pub pos: Vec<[f64; 3]>,
    pub pd: Vec<f64>,
    pub t1: Vec<f64>,
    pub t2: Vec<f64>,
    pub t2dash: Vec<f64>,
    pub adc: Vec<f64>,
    pub b0: Vec<f64>,
    pub b1: Vec<Vec<Complex64>>,
    pub coil_sens: Vec<Vec<Complex64>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VoxelShape {
    /// Axis aligned sinc shape, given by the width of its first zero-crossing
    AASinc([f64; 3]),
    /// Axis aligned box shape, given by its width
    AABox([f64; 3]),
}

impl From<MultiTissuePhantom> for VoxelPhantom {
    fn from(phantom: MultiTissuePhantom) -> Self {
        // Convert grid information into vector of voxel positions
        let sz = phantom.grid_size.map(|x| x as i32);
        let mut pos = Vec::with_capacity(phantom.grid_size.iter().product());
        for x in -(sz[0] / 2)..=(sz[0] - 1) / 2 {
            for y in -(sz[1] / 2)..=(sz[1] - 1) / 2 {
                for z in -(sz[2] / 2)..=(sz[2] - 1) / 2 {
                    pos.push([
                        x as f64 * phantom.grid_spacing[0],
                        y as f64 * phantom.grid_spacing[1],
                        z as f64 * phantom.grid_spacing[2],
                    ]);
                }
            }
        }

        fn mask<'a, T: Copy + 'a>(
            vals_and_pds: impl Iterator<Item = (&'a [T], &'a [f64])>,
        ) -> Vec<T> {
            vals_and_pds
                .flat_map(|(vals, pds)| {
                    vals.iter()
                        .zip(pds)
                        .filter_map(|(&x, &pd)| (pd >= PD_THRESHOLD).then_some(x))
                })
                .collect()
        }
        fn fill<'a, T: Copy>(val_and_pds: impl Iterator<Item = (T, &'a [f64])>) -> Vec<T> {
            val_and_pds
                .flat_map(|(val, pd)| {
                    std::iter::repeat_n(val, pd.iter().filter(|&&pd| pd >= PD_THRESHOLD).count())
                })
                .collect()
        }

        let pos = mask(phantom.tissues.iter().map(|t| (&pos[..], &t.pd[..])));
        let pd = mask(phantom.tissues.iter().map(|t| (&t.pd[..], &t.pd[..])));
        let b0 = mask(phantom.tissues.iter().map(|t| (&t.b0[..], &t.pd[..])));
        let b1: Vec<_> = phantom
            .b1
            .iter()
            .map(|b1| mask(phantom.tissues.iter().map(|t| (&b1[..], &t.pd[..]))))
            .collect();
        let coil_sens: Vec<_> = phantom
            .coil_sens
            .iter()
            .map(|cs| mask(phantom.tissues.iter().map(|t| (&cs[..], &t.pd[..]))))
            .collect();

        let t1 = fill(phantom.tissues.iter().map(|t| (t.props.t1, &t.pd[..])));
        let t2 = fill(phantom.tissues.iter().map(|t| (t.props.t2, &t.pd[..])));
        let t2dash = fill(phantom.tissues.iter().map(|t| (t.props.t2dash, &t.pd[..])));
        let adc = fill(phantom.tissues.iter().map(|t| (t.props.adc, &t.pd[..])));

        Self {
            voxel_shape: phantom.voxel_shape,
            pos,
            pd,
            t1,
            t2,
            t2dash,
            adc,
            b0,
            b1,
            coil_sens,
        }
    }
}

impl From<VoxelGridPhantom> for VoxelPhantom {
    fn from(phantom: VoxelGridPhantom) -> Self {
        // Convert grid information into vector of voxel positions
        let sz = phantom.grid_size.map(|x| x as i32);
        let mut pos = Vec::with_capacity(phantom.grid_size.iter().product());
        for x in -(sz[0] / 2)..=(sz[0] - 1) / 2 {
            for y in -(sz[1] / 2)..=(sz[1] - 1) / 2 {
                for z in -(sz[2] / 2)..=(sz[2] - 1) / 2 {
                    pos.push([
                        x as f64 * phantom.grid_spacing[0],
                        y as f64 * phantom.grid_spacing[1],
                        z as f64 * phantom.grid_spacing[2],
                    ]);
                }
            }
        }

        fn select<T: Copy>(map: &[T], pd: &[f64]) -> Vec<T> {
            map.into_iter()
                .zip(pd)
                .filter_map(|(&x, &pd)| (pd >= PD_THRESHOLD).then_some(x))
                .collect()
        }
        fn select_multi<T: Copy>(map: &[Vec<T>], pd: &[f64]) -> Vec<Vec<T>> {
            map.iter().map(|channel| select(&channel, pd)).collect()
        }

        Self {
            voxel_shape: phantom.voxel_shape,
            pos: select(&pos, &phantom.pd),
            pd: select(&phantom.pd, &phantom.pd),
            t1: select(&phantom.t1, &phantom.pd),
            t2: select(&phantom.t2, &phantom.pd),
            t2dash: select(&phantom.t2dash, &phantom.pd),
            adc: select(&phantom.adc, &phantom.pd),
            b0: select(&phantom.b0, &phantom.pd),
            b1: select_multi(&phantom.b1, &phantom.pd),
            coil_sens: select_multi(&phantom.coil_sens, &phantom.pd),
        }
    }
}
