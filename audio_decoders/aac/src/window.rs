pub fn generate_sine_window(scale: f32, size: usize, dst: &mut [f32]) {
    let param = std::f32::consts::PI / ((2 * size) as f32);
    for (n, out) in dst.iter_mut().enumerate().take(size) {
        *out = (((n as f32) + 0.5) * param).sin() * scale;
    }
}

pub fn generate_kaiser_bessel_window(alpha: f32, scale: f32, size: usize, dst: &mut [f32]) {
    let dlen = size as f32;
    let alpha2 =
        f64::from((alpha * std::f32::consts::PI / dlen) * (alpha * std::f32::consts::PI / dlen));

    let mut kb: Vec<f64> = Vec::with_capacity(size);
    let mut sum = 0.0;
    for n in 0..size {
        let b = bessel_i0(((n * (size - n)) as f64) * alpha2);
        sum += b;
        kb.push(sum);
    }
    sum += 1.0;
    for (n, out) in dst.iter_mut().enumerate().take(size) {
        *out = (kb[n] / sum).sqrt() as f32 * scale;
    }
}

fn bessel_i0(inval: f64) -> f64 {
    let mut val: f64 = 1.0;
    for n in (1..64).rev() {
        val *= inval / f64::from(n * n);
        val += 1.0;
    }
    val
}
