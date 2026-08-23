use smallvec::SmallVec;

pub struct AudioBuffer {
    pub data: SmallVec<[Vec<f32>; 2]>, // Samples per channel
    pub sample_rate: u32,
}
