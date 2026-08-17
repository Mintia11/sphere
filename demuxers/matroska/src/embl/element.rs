use std::io::{Seek, SeekFrom};

use crate::embl::io::{EBMLRead, Error};

#[derive(Debug, Clone, Copy)]
pub struct EBMLElement {
    pub id: u32,
    pub data_start: u64,
    pub data_end: u64,
}

impl EBMLElement {
    pub fn read<T: EBMLRead + Seek>(mut reader: T) -> Result<Self, Error> {
        let id = reader.id()?;
        let size = reader.vint()?;

        let data_start = reader.stream_position()?;
        let data_end = data_start + size;

        Ok(Self {
            id,
            data_start,
            data_end,
        })
    }

    pub fn data<T: EBMLRead + Seek>(&self, mut reader: T) -> Result<Vec<u8>, Error> {
        let size = self.data_end - self.data_start;
        let mut buf = vec![0u8; size as usize];

        let cur_pos = reader.stream_position()?;
        if cur_pos != self.data_start {
            reader.seek(SeekFrom::Start(self.data_start))?;
        }
        reader.read_exact(&mut buf)?;
        reader.seek(SeekFrom::Start(cur_pos))?;

        Ok(buf)
    }

    pub fn data_size(&self) -> usize {
        (self.data_end - self.data_start) as usize
    }

    pub fn for_each_child<T: EBMLRead + Seek>(
        &self,
        reader: &mut T,
        mut callback: impl FnMut(EBMLElement, &mut T) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let cur_pos = reader.stream_position()?;
        if cur_pos != self.data_start {
            reader.seek(SeekFrom::Start(self.data_start))?;
        }

        while reader.stream_position()? < self.data_end {
            let child_element = EBMLElement::read(&mut *reader)?;
            let child_end = child_element.data_end;
            // Skip the Void element
            if child_element.id == 0xEC {
                reader.seek(SeekFrom::Start(child_end))?;
                continue;
            }

            callback(child_element, reader)?;
            reader.seek(SeekFrom::Start(child_end))?;
        }

        reader.seek(SeekFrom::Start(cur_pos))?;
        Ok(())
    }
}
