use std::sync::Arc;

use rustfft::{Fft, FftPlanner, num_complex::Complex};

pub struct Mdct {
    fft: Arc<dyn Fft<f32>>,
    fft_scratch: Box<[Complex<f32>]>,

    scratch: Box<[Complex<f32>]>,
    twiddle: Box<[Complex<f32>]>,
}

impl Mdct {
    pub fn new(size: usize, scale: f64) -> Self {
        let fft: Arc<dyn Fft<f32>> = FftPlanner::new().plan_fft_forward(size / 2);

        let fft_scratch =
            vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()].into_boxed_slice();
        let scratch = vec![Complex::new(0.0, 0.0); size / 2].into_boxed_slice();

        let mut twiddle = Vec::with_capacity(size / 2);

        let alpha = 1.0 / 8.0
            + if scale.is_sign_positive() {
                0.0
            } else {
                (size / 2) as f64
            };
        let pi_n = std::f64::consts::PI / size as f64;
        let sqrt_scale = scale.abs().sqrt();

        for k in 0..(size / 2) {
            let theta = pi_n * (alpha + k as f64);
            let re = sqrt_scale * theta.cos();
            let im = sqrt_scale * theta.sin();
            twiddle.push(Complex::new(re as f32, im as f32));
        }

        Self {
            fft,
            fft_scratch,

            scratch,
            twiddle: twiddle.into_boxed_slice(),
        }
    }

    pub fn imdct(&mut self, input: &[f32], output: &mut [f32]) {
        let n = self.fft.len() * 2;

        assert!(input.len() == n);
        assert!(output.len() == 2 * n);

        for (i, (&w, t)) in self.twiddle.iter().zip(self.scratch.iter_mut()).enumerate() {
            let even = input[i * 2];
            let odd = -input[n - 1 - i * 2];

            let re = odd * w.im - even * w.re;
            let im = odd * w.re + even * w.im;
            *t = Complex::new(re, im);
        }

        self.fft
            .process_with_scratch(&mut self.scratch, &mut self.fft_scratch);

        let (vec0, vec1) = output.split_at_mut(n / 2);
        let (vec1, vec2) = vec1.split_at_mut(n / 2);
        let (vec2, vec3) = vec2.split_at_mut(n / 2);

        for (i, (x, &w)) in self.scratch[..n / 4]
            .iter()
            .zip(self.twiddle[..n / 4].iter())
            .enumerate()
        {
            let val = w * x.conj();

            let fi = 2 * i;
            let ri = (n / 2) - 1 - 2 * i;

            vec0[ri] = -val.im;
            vec1[fi] = val.im;
            vec2[ri] = val.re;
            vec3[fi] = val.re;
        }

        for (i, (x, &w)) in self.scratch[n / 4..]
            .iter()
            .zip(self.twiddle[n / 4..].iter())
            .enumerate()
        {
            let val = w * x.conj();

            let fi = 2 * i;
            let ri = (n / 2) - 1 - 2 * i;

            vec0[fi] = -val.re;
            vec1[ri] = val.re;
            vec2[fi] = val.im;
            vec3[ri] = val.im;
        }
    }
}
