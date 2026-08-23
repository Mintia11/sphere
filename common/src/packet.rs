//! Packet representation

use bytes::Bytes;

use crate::{
    audio::{AudioBuffer, AudioInfo},
    huffman,
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

pub enum Frame {
    Video {
        image: etna::Image,
        pts: Timestamp,
    },
    Audio {
        samples: AudioBuffer,
        pts: Timestamp,
    },
}

pub trait PacketDecoder: Send {
    fn track(&mut self, track: &TrackInfo) -> Result<(), Error>;
    fn can_decode_track(&self) -> Result<bool, Error>;
    fn info_strings(&self) -> Vec<String>;
    fn start_decode_session(&mut self) -> Result<(), Error>;
    fn send_packet(&mut self, packet: Packet) -> Result<(), Error>;
    fn grab_frame(&self) -> Result<Option<Frame>, Error>;

    fn audio_info(&self) -> Option<AudioInfo> {
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encountered invalid data: {0}")]
    InvalidData(String),

    #[error("Vulkan error: {0}")]
    VulkanError(#[from] etna::vk::Result),

    #[error("Etna error: {0}")]
    EtnaError(#[from] etna::error::Error),

    #[error("Error while doing huffman decoding: {0}")]
    HuffmanError(#[from] huffman::Error),
}
