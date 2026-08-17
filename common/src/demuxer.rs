//! Demuxing related functionality

use crate::{
    packet::Packet,
    time::Timestamp,
    track::{TrackId, TrackInfo},
};

/// A trait for demuxers, which are responsible for reading packets from a media source and providing information about the available tracks.
pub trait Demuxer {
    /// Returns a slice of `TrackInfo` representing the available tracks in the media source.
    fn tracks(&self) -> &[TrackInfo];

    /// Reads the next packet from the media source.
    /// Returns `Ok(Some(packet))` if a packet was read, `Ok(None)` if the end of the stream was reached, or an `io::Error` if an error occurred.
    fn read_packet(&mut self) -> Result<Option<Packet>, DemuxingError>;

    /// Seeks to a specific timestamp in the specified track.
    fn seek(&mut self, track: TrackId, target: Timestamp) -> Result<(), DemuxingError>;
}

/// An error type for demuxing operations
#[derive(Debug, thiserror::Error)]
pub enum DemuxingError {}
