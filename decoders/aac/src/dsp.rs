use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use common::{audio::AudioBuffer, mcdt::Mdct};
use crossbeam_channel::{Receiver, Sender};
use smallvec::SmallVec;

use crate::{
    ics::{Ics, WindowSequence},
    window::{generate_kaiser_bessel_window, generate_sine_window},
};

pub fn run_dsp(stop: Arc<AtomicBool>, raw_data: Receiver<Vec<Ics>>, samples: Sender<AudioBuffer>) {
    let mut short_mdct = Mdct::new(128, 1.0 / 256.0);
    let mut long_mdct = Mdct::new(1024, 1.0 / 2048.0);

    let mut kb_long_window = Box::new([0.0f32; 1024]);
    let mut kb_short_window = Box::new([0.0f32; 128]);
    generate_kaiser_bessel_window(4.0, 1.0, 1024, kb_long_window.as_mut_slice());
    generate_kaiser_bessel_window(6.0, 1.0, 128, kb_short_window.as_mut_slice());

    let mut sine_long_window = Box::new([0.0f32; 1024]);
    let mut sine_short_window = Box::new([0.0f32; 128]);
    generate_sine_window(1.0, 1024, sine_long_window.as_mut_slice());
    generate_sine_window(1.0, 128, sine_short_window.as_mut_slice());

    let mut prev_window_shape = false;
    let mut pcm_long = Box::new([0.0; 2048]);
    let mut pcm_short = Box::new([0.0; 1152]);
    let mut delay = Box::new([0.0; 1024]);

    while !stop.load(Ordering::Relaxed) {
        let Ok(packet) = raw_data.recv_timeout(Duration::from_millis(100)) else {
            continue;
        };

        let mut decoded_samples = SmallVec::new();
        for ics in packet {
            let (long_window, short_window) = if ics.info.window_shape {
                (kb_long_window.as_slice(), kb_short_window.as_slice())
            } else {
                (sine_long_window.as_slice(), sine_short_window.as_slice())
            };

            let (prev_long_window, prev_short_window) = if prev_window_shape {
                (kb_long_window.as_slice(), kb_short_window.as_slice())
            } else {
                (sine_long_window.as_slice(), sine_short_window.as_slice())
            };

            let mut dst = vec![0.0; 1024];
            match ics.info.window_sequence {
                WindowSequence::EightShort => {
                    for (ain, aout) in ics
                        .spectral
                        .coefficents
                        .chunks_exact(128)
                        .zip(pcm_long.chunks_exact_mut(256))
                    {
                        short_mdct.imdct(ain, aout);
                    }

                    pcm_short.fill(0.0);

                    for (w, src) in pcm_long.chunks_exact(256).enumerate() {
                        if w > 0 {
                            for i in 0..128 {
                                pcm_short[w * 128 + i] += src[i] * short_window[i];
                                pcm_short[w * 128 + i + 128] +=
                                    src[i + 128] * short_window[127 - i];
                            }
                        } else {
                            for i in 0..128 {
                                pcm_short[i] = src[i] * prev_short_window[i];
                                pcm_short[i + 128] = src[i + 128] * short_window[127 - i];
                            }
                        }
                    }
                }
                _ => {
                    long_mdct.imdct(ics.spectral.coefficents.as_slice(), pcm_long.as_mut_slice());
                }
            }

            const SHORT_WIN_POINT0: usize = 512 - 64;
            const SHORT_WIN_POINT1: usize = 512 + 64;

            match ics.info.window_sequence {
                WindowSequence::LongStart => {
                    for i in 0..1024 {
                        dst[i] = delay[i] + (pcm_long[i] * prev_long_window[i]);
                    }
                    delay[..SHORT_WIN_POINT0]
                        .copy_from_slice(&pcm_long[1024..(SHORT_WIN_POINT0 + 1024)]);
                    for i in SHORT_WIN_POINT0..SHORT_WIN_POINT1 {
                        delay[i] = pcm_long[i + 1024] * short_window[127 - (i - SHORT_WIN_POINT0)];
                    }
                    delay[SHORT_WIN_POINT1..].fill(0.0);
                }
                WindowSequence::EightShort => {
                    dst[..SHORT_WIN_POINT0].copy_from_slice(&delay[..SHORT_WIN_POINT0]);
                    for i in SHORT_WIN_POINT0..1024 {
                        dst[i] = delay[i] + pcm_short[i - SHORT_WIN_POINT0];
                    }
                    for i in 0..SHORT_WIN_POINT1 {
                        delay[i] = pcm_short[i + 512 + 64];
                    }
                    delay[SHORT_WIN_POINT1..].fill(0.0);
                }
                WindowSequence::LongStop => {
                    dst[..SHORT_WIN_POINT0].copy_from_slice(&delay[..SHORT_WIN_POINT0]);
                    for i in SHORT_WIN_POINT0..SHORT_WIN_POINT1 {
                        dst[i] = delay[i] + pcm_long[i] * prev_short_window[i - SHORT_WIN_POINT0];
                    }
                    for i in SHORT_WIN_POINT1..1024 {
                        dst[i] = delay[i] + pcm_long[i];
                    }
                    for i in 0..1024 {
                        delay[i] = pcm_long[i + 1024] * long_window[1023 - i];
                    }
                }
                WindowSequence::OnlyLong => {
                    for i in 0..1024 {
                        dst[i] = delay[i] + (pcm_long[i] * prev_long_window[i]);
                    }
                    for i in 0..1024 {
                        delay[i] = pcm_long[i + 1024] * long_window[1023 - i];
                    }
                }
            }

            prev_window_shape = ics.info.window_shape;

            decoded_samples.push(dst);
        }

        let packet = AudioBuffer {
            data: decoded_samples,
        };
        samples.send(packet).unwrap();
    }
}
