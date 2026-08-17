//! Track-related types

use crate::time::TimeBase;

/// A unique identifier for a track.
pub type TrackId = u32;

/// Information about a media track, including its ID, kind, codec, timebase, and optional extra data.
#[derive(Debug, Clone)]
pub struct TrackInfo {
    /// The unique identifier for the track.
    pub id: TrackId,
    /// The kind of media track (e.g., video, audio, subtitle).
    pub kind: TrackKind,
    /// The codec used for the track (e.g., H264, AAC).
    pub codec: CodecId,
    /// The timebase associated with the track, used for timestamp calculations.
    pub timebase: TimeBase,
    /// Optional extra data associated with the track, such as codec-specific initialization data.
    pub extra_data: Option<Box<[u8]>>,
    /// The bitstream format of the track, indicating how the encoded data is structured.
    pub bitstream_format: BitstreamFormat,
}

/// An enumeration of the different kinds of media tracks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    /// The track is a video track, containing visual content.
    Video,

    /// The track is an audio track, containing sound content.
    Audio,

    /// The track is a subtitle track, containing text or graphical subtitles.
    Subtitle,

    /// The track is of an unknown kind, which may be used for unsupported or unrecognized track types.
    Unknown,
}

/// An enumeration of the different codec identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecId {
    /// The track uses the H.264 video codec.
    H264,

    /// The track uses the AAC audio codec.
    Aac,

    Unknown(String),
}

/// An enumeration of the different bitstream formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitstreamFormat {
    /// The track uses the Annex B bitstream format, commonly used for H.264 video.
    AnnexB,

    /// The track uses a length-prefixed bitstream format, where each NAL unit is preceded by its length.
    LengthPrefixed { nal_length_size: u8 },

    /// The track uses an opaque bitstream format, which may be specific to a particular codec or container.
    Opaque,
}
