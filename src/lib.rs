use quick_protobuf::{MessageRead,Reader};
use std::io::{Read,Seek,SeekFrom};

pub mod proto;
use proto::fileformat::{Blob,BlobHeader};

type Error = Box<dyn std::error::Error+Send+Sync+'static>;

pub struct OsmPbfDenormalize<F: Read+Seek> {
  handle: Box<F>,
}

impl<F> OsmPbfDenormalize<F> where F: Read+Seek {
  pub fn open(handle: Box<F>) -> Self {
    Self { handle }
  }
  pub fn scan(&mut self) -> Result<(),Error> {
    self.handle.seek(SeekFrom::Start(0))?;
    Ok(())
  }
  pub fn read_header(&mut self) -> Result<BlobHeader,Error> {
    let mut len_buf = [0,0,0,0];
    self.handle.seek(SeekFrom::Start(0))?;
    let n = self.handle.read(&mut len_buf)?;
    if n != 4 { panic!["{} != 4", n] }
    let len = u32::from_be_bytes(len_buf) as usize;
    self.handle.seek(SeekFrom::Start(4))?;
    let mut buf = vec![0u8;len];
    let n = self.handle.read(&mut buf)?;
    if n != len { panic!["not enough bytes read. expected {}, got {}", len, n] }
    let mut reader = Reader::from_bytes(buf);
    let blob_header = reader.read(BlobHeader::from_reader)?;
    Ok(blob_header)
  }
  pub fn read_blob(&mut self, offset: u64) -> Result<Blob,Error> {
    let mut len_buf = [0,0,0,0];
    self.handle.seek(SeekFrom::Start(offset))?;
    let n = self.handle.read(&mut len_buf)?;
    if n != 4 { panic!["{} != 4", n] }
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8;len];
    let n = self.handle.read(&mut buf)?;
    if n != len { panic!["not enough bytes read. expected {}, got {}", len, n] }
    let mut reader = Reader::from_bytes(buf);
    let blob = reader.read(Blob::from_reader)?;
    Ok(blob)
  }
}
