use quick_protobuf::{MessageRead,Reader};
use std::io::{Read,Seek,SeekFrom};

pub mod proto;
pub use proto::fileformat::{Blob,BlobHeader};
mod decode;
pub mod element;
pub use element::{Element,Info,Node,Way,Relation,Member,MemberType};

type Error = Box<dyn std::error::Error+Send+Sync+'static>;

pub struct OsmPbfDenormalize<F: Read+Seek+GetLen+TryClone> {
  handle: Box<F>,
}

pub trait GetLen {
  fn get_len(&self) -> Result<u64,Error>;
}
impl GetLen for std::fs::File {
  fn get_len(&self) -> Result<u64,Error> {
    Ok(self.metadata()?.len())
  }
}
pub trait TryClone {
  fn try_clone(&self) -> Result<Self,Error> where Self: Sized;
}
impl TryClone for std::fs::File {
  fn try_clone(&self) -> Result<Self,Error> {
    std::fs::File::try_clone(self).map_err(|e| e.into())
  }
}

impl<F> OsmPbfDenormalize<F> where F: Read+Seek+GetLen+TryClone {
  pub fn open(handle: Box<F>) -> Self {
    Self { handle }
  }
  pub fn read_fileblock(&mut self, offset: u64) -> Result<(u64,BlobHeader,Blob),Error> {
    Self::read_fileblock_h(&mut self.handle, offset)
  }
  pub fn read_fileblock_h(h: &mut F, offset: u64) -> Result<(u64,BlobHeader,Blob),Error> {
    let (s,blob_header) = Self::read_blob_header_h(h, offset)?;
    let blob = Self::read_blob_h(h, offset + s, blob_header.datasize as usize)?;
    Ok((s + blob_header.datasize as u64, blob_header, blob))
  }
  pub fn read_blob_header(&mut self, offset: u64) -> Result<(u64,BlobHeader),Error> {
    Self::read_blob_header_h(&mut self.handle, offset)
  }
  pub fn read_blob_header_h(h: &mut F, offset: u64) -> Result<(u64,BlobHeader),Error> {
    let mut len_buf = [0,0,0,0];
    h.seek(SeekFrom::Start(offset))?;
    let n = h.read(&mut len_buf)?;
    if n != 4 { panic!["{} != 4", n] }
    let len = u32::from_be_bytes(len_buf) as usize;
    h.seek(SeekFrom::Start(offset+4))?;
    let mut buf = vec![0u8;len];
    let n = h.read(&mut buf)?;
    if n != len { panic!["not enough bytes read. expected {}, got {}", len, n] }
    let mut reader = Reader::from_bytes(buf);
    let blob_header = reader.read(BlobHeader::from_reader)?;
    Ok(((len+4) as u64, blob_header))
  }
  pub fn read_blob(&mut self, offset: u64, len: usize) -> Result<Blob,Error> {
    Self::read_blob_h(&mut self.handle, offset, len)
  }
  pub fn read_blob_h(h: &mut F, offset: u64, len: usize) -> Result<Blob,Error> {
    h.seek(SeekFrom::Start(offset))?;
    let mut buf = vec![0u8;len];
    let n = h.read(&mut buf)?;
    if n != len { panic!["not enough bytes read. expected {}, got {}", len, n] }
    let mut reader = Reader::from_bytes(buf);
    let blob = reader.read(Blob::from_reader)?;
    Ok(blob)
  }
}
