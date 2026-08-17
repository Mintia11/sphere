use std::fs::File;

use common::{demuxer::Demuxer, packet::PacketFlags};
use matroska::MatroskaDemuxer;

fn main() {
    let mut args = std::env::args();
    let file = args.nth(1).expect("usage: sphere <file>");

    let mut file = File::open(file).expect("Failed to open file");
    let mut demuxer =
        MatroskaDemuxer::new(&mut file).expect("Failed to initialize matroska demuxer");

    for track in demuxer.tracks() {
        println!(
            "Track {} - kind: {:?}, codec: {:?}",
            track.id, track.kind, track.codec
        );
    }

    while let Some(packet) = demuxer.read_packet().expect("Failed to read packet") {
        println!(
            "Packet of track {} - pts: {:1.3}, dts: {:1.3}{}",
            packet.track,
            packet.pts.to_seconds(),
            packet.dts.to_seconds(),
            if packet.flags.contains(PacketFlags::KEYFRAME) {
                ", keyframe"
            } else {
                ""
            }
        );
    }
}
