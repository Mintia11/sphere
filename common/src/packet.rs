//! Packet representation

use bytes::Bytes;

use crate::{
    time::Timestamp,
    track::{TrackId, TrackInfo},
};

bitflags::bitflags! {
    /// Flags associated with a packet, indicating its properties.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PacketFlags: u32 {
        /// The packet is a keyframe, which can be decoded independently of other frames.
        const KEYFRAME = 1 << 0;
        /// The packet should be discarded, indicating that it may be corrupted or not needed for playback.
        const DISCARD  = 1 << 1;
        /// The packet is corrupt, indicating that it may contain errors or be incomplete.
        const CORRUPT  = 1 << 2;
    }
}

/// A packet represents a unit of encoded media data, associated with a specific track and timestamps.
#[derive(Debug, Clone)]
pub struct Packet {
    /// The unique identifier of the track to which this packet belongs.
    pub track: TrackId,
    /// The presentation timestamp (PTS) of the packet, indicating when it should be presented during playback.
    pub pts: Timestamp,
    /// The decoding timestamp (DTS) of the packet, indicating when it should be decoded.
    pub dts: Timestamp,
    /// The duration of the packet, in the units of the associated timebase. This may be `None` if the duration is unknown.
    pub duration: Option<i64>,
    /// The encoded data of the packet.
    pub data: Bytes,
    /// Flags associated with the packet.
    pub flags: PacketFlags,
}

pub trait PacketDecoder {
    fn track(&mut self, track: &TrackInfo) -> Result<(), Error>;
    fn can_decode_track(&self) -> Result<bool, Error>;
    fn info_strings(&self) -> Vec<String>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encountered invalid data: {0}")]
    InvalidData(String),
}
