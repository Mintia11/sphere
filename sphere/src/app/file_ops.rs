use std::{fs::File, path::Path};

use common::demuxer::Demuxer;
use matroska::MatroskaDemuxer;

pub fn open_demuxer(path: impl AsRef<Path>) -> Option<Box<dyn Demuxer>> {
    let path = path.as_ref();
    let file = File::open(path).expect("Failed to open file");

    let ext = path.extension()?;
    let ext = ext.to_str()?;

    match ext {
        "mkv" => Some(Box::new(
            MatroskaDemuxer::new(file).expect("Failed to create matroska demuxer"),
        )),
        _ => None,
    }
}
