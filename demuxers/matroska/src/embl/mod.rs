use std::io::Seek;

use crate::embl::io::{EBMLRead, Error};

pub use element::EBMLElement;

pub mod element;
pub mod io;

pub trait EBMLMasterElement<T: EBMLRead + Seek>
where
    Self: Sized + Default,
{
    const ID: u32;

    fn visit_child(&mut self, sub_element: EBMLElement, reader: &mut T) -> Result<(), Error>;

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct EBMLHeader {
    pub version: u64,
    pub read_version: u64,
    pub max_id_length: u64,
    pub max_size_length: u64,
    pub doc_type: String,
    pub doc_type_version: u64,
    pub doc_type_read_version: u64,
}

impl<T: EBMLRead + Seek> EBMLMasterElement<T> for EBMLHeader {
    const ID: u32 = 0x1A45DFA3;

    fn visit_child(&mut self, sub_element: EBMLElement, reader: &mut T) -> Result<(), Error> {
        match sub_element.id {
            0x4286 => self.version = reader.uinteger(sub_element.data_size())?,
            0x42F7 => self.read_version = reader.uinteger(sub_element.data_size())?,
            0x42F2 => self.max_id_length = reader.uinteger(sub_element.data_size())?,
            0x42F3 => self.max_size_length = reader.uinteger(sub_element.data_size())?,
            0x4282 => self.doc_type = reader.string(sub_element.data_size())?,
            0x4287 => self.doc_type_version = reader.uinteger(sub_element.data_size())?,
            0x4285 => self.doc_type_read_version = reader.uinteger(sub_element.data_size())?,
            _ => {}
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), Error> {
        if self.version != 1 {
            return Err(Error::InvalidData(format!(
                "Invalid EBML version: {}",
                self.version
            )));
        }

        if self.read_version != 1 {
            return Err(Error::InvalidData(format!(
                "Invalid EBML read version: {}",
                self.read_version
            )));
        }

        if self.max_id_length < 4 {
            return Err(Error::InvalidData(format!(
                "Invalid EBML max ID length: {}",
                self.max_id_length
            )));
        }

        if self.max_size_length < 8 {
            return Err(Error::InvalidData(format!(
                "Invalid EBML max size length: {}",
                self.max_size_length
            )));
        }

        Ok(())
    }
}
