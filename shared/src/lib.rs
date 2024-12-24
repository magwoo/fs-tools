use std::fs::File;
use std::io::{self, Seek};
use std::io::{prelude::*, SeekFrom};
use std::os::unix::fs::FileExt;
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedFile {
    cur_pos: u64,
    file: Arc<File>,
}

impl SharedFile {
    pub fn new(mut file: File) -> io::Result<Self> {
        Ok(Self {
            cur_pos: file.stream_position()?,
            file: Arc::new(file),
        })
    }

    pub fn len(&self) -> io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }

    pub fn is_empty(&self) -> io::Result<bool> {
        self.len().map(|l| l == 0)
    }
}

impl Read for SharedFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let readed = self.file.read_at(buf, self.cur_pos)?;

        self.cur_pos += readed as u64;

        Ok(readed)
    }
}

impl Write for SharedFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let writed = self.file.write_at(buf, self.cur_pos)?;

        self.cur_pos += writed as u64;

        Ok(writed)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for SharedFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let file_len = self.len()?;

        let new_pos = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => (file_len as i64 - offset) as u64,
            SeekFrom::Current(offset) => (self.cur_pos as i64 + offset) as u64,
        };

        if new_pos > file_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek position out of bounds",
            ));
        }

        self.cur_pos = new_pos;

        Ok(new_pos)
    }
}
