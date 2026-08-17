use std::io::Seek;

use crate::embl::{
    EBMLElement, EBMLMasterElement,
    io::{EBMLRead, Error},
};

#[derive(Debug, Default)]
pub struct Info {
    pub segment_uid: Option<[u8; 16]>,
    pub segment_filename: Option<String>,
    pub prev_uid: Option<[u8; 16]>,
    pub prev_filename: Option<String>,
    pub next_uid: Option<[u8; 16]>,
    pub next_filename: Option<String>,
    pub segment_family: Option<[u8; 16]>,
    pub timecode_scale: u64,
    pub duration: Option<f64>,
    pub date_utc: Option<u64>,
    pub title: Option<String>,
    pub muxing_app: String,
    pub writing_app: String,
}

impl<T: EBMLRead + Seek> EBMLMasterElement<T> for Info {
    const ID: u32 = 0x1549A966;

    fn visit_child(&mut self, sub_element: EBMLElement, reader: &mut T) -> Result<(), Error> {
        match sub_element.id {
            0x73A4 => {
                self.segment_uid = Some(
                    reader
                        .binary(sub_element.data_size())?
                        .try_into()
                        .map_err(|_| {
                            Error::InvalidData("Segment UID must be 16 bytes".to_string())
                        })?,
                )
            }
            0x7384 => self.segment_filename = Some(reader.string(sub_element.data_size())?),
            0x3CB923 => {
                self.prev_uid = Some(
                    reader
                        .binary(sub_element.data_size())?
                        .try_into()
                        .map_err(|_| Error::InvalidData("Prev UID must be 16 bytes".to_string()))?,
                )
            }
            0x3C83AB => self.prev_filename = Some(reader.string(sub_element.data_size())?),
            0x3EB923 => {
                self.next_uid = Some(
                    reader
                        .binary(sub_element.data_size())?
                        .try_into()
                        .map_err(|_| Error::InvalidData("Next UID must be 16 bytes".to_string()))?,
                )
            }
            0x3E83BB => self.next_filename = Some(reader.string(sub_element.data_size())?),
            0x4444 => {
                self.segment_family = Some(
                    reader
                        .binary(sub_element.data_size())?
                        .try_into()
                        .map_err(|_| {
                            Error::InvalidData("Segment Family must be 16 bytes".to_string())
                        })?,
                )
            }
            0x2AD7B1 => self.timecode_scale = reader.uinteger(sub_element.data_size())?,
            0x4489 => self.duration = Some(reader.float(sub_element.data_size())?),
            0x4461 => self.date_utc = Some(reader.uinteger(sub_element.data_size())?),
            0x7BA9 => self.title = Some(reader.string(sub_element.data_size())?),
            0x4D80 => self.muxing_app = reader.string(sub_element.data_size())?,
            0x5741 => self.writing_app = reader.string(sub_element.data_size())?,
            _ => {}
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), Error> {
        if self.timecode_scale == 0 {
            return Err(Error::InvalidData(
                "Timecode scale must be greater than 0".to_string(),
            ));
        }

        if self.muxing_app.is_empty() {
            return Err(Error::InvalidData(
                "Muxing app must not be empty".to_string(),
            ));
        }

        if self.writing_app.is_empty() {
            return Err(Error::InvalidData(
                "Writing app must not be empty".to_string(),
            ));
        }

        Ok(())
    }
}
