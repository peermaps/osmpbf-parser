use quick_protobuf::{MessageRead,Reader};
use std::io::{Read,Seek,SeekFrom};

pub mod proto;
pub use proto::fileformat::{Blob,BlobHeader};
mod decode;
pub mod element;
pub use element::{Element,Info,Node,Way,Relation,Member,MemberType};
mod scan;
pub use scan::Scan;

pub type Error = Box<dyn std::error::Error+Send+Sync+'static>;

pub struct OsmPbfDenormalize<F: Read+Seek> {
  handle: Box<F>,
}

impl<F> OsmPbfDenormalize<F> where F: Read+Seek {
  pub fn open(handle: Box<F>) -> Self {
    Self { handle }
  }
  pub fn read_fileblock(&mut self, offset: u64) -> Result<(u64,BlobHeader,Blob),Error> {
    let (s,blob_header) = self.read_blob_header(offset)?;
    let blob = self.read_blob(offset + s, blob_header.datasize as usize)?;
    Ok((s + blob_header.datasize as u64, blob_header, blob))
  }
  pub fn read_blob_header(&mut self, offset: u64) -> Result<(u64,BlobHeader),Error> {
    let mut len_buf = [0,0,0,0];
    self.handle.seek(SeekFrom::Start(offset))?;
    let n = self.handle.read(&mut len_buf)?;
    if n != 4 { panic!["{} != 4", n] }
    let len = u32::from_be_bytes(len_buf) as usize;
    self.handle.seek(SeekFrom::Start(offset+4))?;
    let mut buf = vec![0u8;len];
    let n = self.handle.read(&mut buf)?;
    if n != len { panic!["not enough bytes read. expected {}, got {}", len, n] }
    let mut reader = Reader::from_bytes(buf);
    let blob_header = reader.read(BlobHeader::from_reader)?;
    Ok(((len+4) as u64, blob_header))
  }
  pub fn read_blob(&mut self, offset: u64, len: usize) -> Result<Blob,Error> {
    self.handle.seek(SeekFrom::Start(offset))?;
    let mut buf = vec![0u8;len];
    let n = self.handle.read(&mut buf)?;
    if n != len { panic!["not enough bytes read. expected {}, got {}", len, n] }
    let mut reader = Reader::from_bytes(buf);
    let blob = reader.read(Blob::from_reader)?;
    Ok(blob)
  }
  pub fn read(&mut self, offset: u64) -> Result<(u64,Vec<element::Element>),Error> {
    let (len,_blob_header,blob) = self.read_fileblock(offset)?;
    if offset == 0 { // header
      Ok((len, vec![]))
    } else {
      Ok((len, blob.decode_primitive()?))
    }
  }
}
