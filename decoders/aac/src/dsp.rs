use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use common::audio::AudioBuffer;
use crossbeam_channel::{Receiver, Sender};

use crate::ics::Ics;

pub fn run_dsp(stop: Arc<AtomicBool>, raw_data: Receiver<Vec<Ics>>, samples: Sender<AudioBuffer>) {
    while !stop.load(Ordering::Relaxed) {
        let Ok(packet) = raw_data.recv_timeout(Duration::from_millis(100)) else {
            continue;
        };
    }
}
