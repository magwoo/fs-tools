use std::io::{self, Read, Seek, SeekFrom, Write};
use std::ops::Range;

#[cfg(not(feature = "shared"))]
type File = std::fs::File;
#[cfg(feature = "shared")]
type File = shared_file::SharedFile;

type Frame = Range<u64>;

pub struct FramedFile {
    file: File,
    frame: Frame,
    current_pos: u64,
}

impl FramedFile {
    pub fn new(file: File, frame: Frame) -> io::Result<Self> {
        Self::from_range(file, frame)
    }

    pub fn from_range(mut file: File, frame: Frame) -> io::Result<Self> {
        file.seek(SeekFrom::Start(frame.start))?;

        Ok(Self {
            file,
            frame,
            current_pos: 0,
        })
    }

    pub fn from_len(file: File, start: u64, len: u64) -> io::Result<Self> {
        let end = start + len;

        Self::from_range(file, start..end)
    }

    pub fn position(&self) -> u64 {
        self.current_pos
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub fn frame_start(&self) -> u64 {
        self.frame().start
    }

    pub fn frame_end(&self) -> u64 {
        self.frame().end
    }

    pub fn frame_len(&self) -> u64 {
        self.frame_end() - self.frame_start()
    }

    pub fn remaining_len(&self) -> u64 {
        self.frame_len() - self.position()
    }

    pub fn into_raw_file(self) -> File {
        self.file
    }
}

impl Read for FramedFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let max_to_read = self.remaining_len() as usize;
        let buf_len = buf.len();
        let to_read = max_to_read.min(buf_len);

        if to_read == 0 {
            return Ok(0);
        }

        let bytes_read = self.file.read(&mut buf[..to_read])?;
        self.current_pos += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl Write for FramedFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let max_to_write = self.remaining_len() as usize;
        let buf_len = buf.len();
        let to_write = max_to_write.min(buf_len);

        if to_write == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "Write exceeds file range",
            ));
        }

        let bytes_written = self.file.write(&buf[..to_write])?;
        self.current_pos += bytes_written as u64;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for FramedFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => (self.frame_len() as i64 + offset) as u64,
            SeekFrom::Current(offset) => (self.current_pos as i64 + offset) as u64,
        };

        if new_pos > self.frame_len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Seek position out of bounds",
            ));
        }

        self.file
            .seek(SeekFrom::Start(self.frame_start() + new_pos))?;
        self.current_pos = new_pos;
        Ok(self.current_pos)
    }
}
