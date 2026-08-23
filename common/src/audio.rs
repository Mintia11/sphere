use smallvec::SmallVec;

pub struct AudioBuffer {
    pub data: SmallVec<[Vec<f32>; 2]>, // Samples per channel
    pub sample_rate: u32,
}

impl AudioBuffer {
    pub fn frame_count(&self) -> usize {
        self.data.first().map_or(0, |ch| ch.len())
    }

    pub fn channels(&self) -> usize {
        self.data.len()
    }

    pub fn write_interleaved(&self, out: &mut Vec<f32>) {
        out.clear();
        out.reserve(self.frame_count() * self.channels());
        for i in 0..self.frame_count() {
            for channel in &self.data {
                out.push(channel[i]);
            }
        }
    }
}

pub fn downmix_interleaved(buffer: &AudioBuffer, target_channels: usize, out: &mut Vec<f32>) {
    out.clear();
    let frames = buffer.frame_count();
    out.reserve(frames * target_channels);

    let src_channels = buffer.channels();

    if src_channels == target_channels {
        for i in 0..frames {
            for ch in &buffer.data {
                out.push(ch[i]);
            }
        }
        return;
    }

    match (src_channels, target_channels) {
        (1, n) => {
            for i in 0..frames {
                let sample = buffer.data[0][i];
                for _ in 0..n {
                    out.push(sample);
                }
            }
        }
        (src, 2) if src >= 5 => {
            const CENTER_GAIN: f32 = std::f32::consts::FRAC_1_SQRT_2;
            const SURROUND_GAIN: f32 = std::f32::consts::FRAC_1_SQRT_2;
            for i in 0..frames {
                let l = buffer.data[0][i];
                let r = buffer.data[1][i];
                let c = buffer.data[2][i];
                let ls = buffer.data[4][i];
                let rs = buffer.data[5][i];
                out.push(l + CENTER_GAIN * c + SURROUND_GAIN * ls);
                out.push(r + CENTER_GAIN * c + SURROUND_GAIN * rs);
            }
        }
        (src, n) if src > n => {
            for i in 0..frames {
                for out_ch in 0..n {
                    let sample: f32 = buffer
                        .data
                        .iter()
                        .enumerate()
                        .filter(|(idx, _)| idx % n == out_ch)
                        .map(|(_, ch)| ch[i])
                        .sum();
                    out.push(sample);
                }
            }
        }
        (src, n) => {
            for i in 0..frames {
                for out_ch in 0..n {
                    out.push(buffer.data[out_ch % src][i]);
                }
            }
        }
    }
}

pub struct AudioInfo {
    pub channel_count: usize,
    pub sample_rate: u32,
}
