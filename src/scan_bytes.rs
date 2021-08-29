use desert::{ToBytes,FromBytes,CountBytes,varint};
use crate::{ScanTable,Error};
use std::ops::Bound::Included;

impl ToBytes for ScanTable {
  fn to_bytes(&self) -> Result<Vec<u8>,Error> {
    let mut buf = vec![0;self.count_bytes()];
    self.write_bytes(&mut buf)?;
    Ok(buf)
  }
  fn write_bytes(&self, buf: &mut [u8]) -> Result<usize,Error> {
    let mut offset = 0;
    offset += varint::encode(self.node_interval_offsets.len() as u64, &mut buf[offset..])?;
    for (id_range,(byte_offset,byte_len,item_len)) in self.node_interval_offsets.iter() {
      if let (Included(low),Included(high)) = id_range {
        offset += varint::encode(*low as u64, &mut buf[offset..])?;
        offset += varint::encode(*high as u64, &mut buf[offset..])?;
        offset += varint::encode(*byte_offset, &mut buf[offset..])?;
        offset += varint::encode(*byte_len as u64, &mut buf[offset..])?;
        offset += varint::encode(*item_len as u64, &mut buf[offset..])?;
      }
    }
    offset += varint::encode(self.way_interval_offsets.len() as u64, &mut buf[offset..])?;
    for (id_range,(byte_offset,byte_len,item_len)) in self.way_interval_offsets.iter() {
      if let (Included(low),Included(high)) = id_range {
        offset += varint::encode(*low as u64, &mut buf[offset..])?;
        offset += varint::encode(*high as u64, &mut buf[offset..])?;
        offset += varint::encode(*byte_offset, &mut buf[offset..])?;
        offset += varint::encode(*byte_len as u64, &mut buf[offset..])?;
        offset += varint::encode(*item_len as u64, &mut buf[offset..])?;
      }
    }
    offset += varint::encode(self.relation_interval_offsets.len() as u64, &mut buf[offset..])?;
    for (id_range,(byte_offset,byte_len,item_len)) in self.relation_interval_offsets.iter() {
      if let (Included(low),Included(high)) = id_range {
        offset += varint::encode(*low as u64, &mut buf[offset..])?;
        offset += varint::encode(*high as u64, &mut buf[offset..])?;
        offset += varint::encode(*byte_offset, &mut buf[offset..])?;
        offset += varint::encode(*byte_len as u64, &mut buf[offset..])?;
        offset += varint::encode(*item_len as u64, &mut buf[offset..])?;
      }
    }
    Ok(offset)
  }
}

impl CountBytes for ScanTable {
  fn count_bytes(&self) -> usize {
    let mut size = 0;
    size += varint::length(self.node_interval_offsets.len() as u64);
    for (id_range,(byte_offset,byte_len,item_len)) in self.node_interval_offsets.iter() {
      if let (Included(low),Included(high)) = id_range {
        size += varint::length(*low as u64);
        size += varint::length(*high as u64);
        size += varint::length(*byte_offset);
        size += varint::length(*byte_len as u64);
        size += varint::length(*item_len as u64);
      }
    }
    size += varint::length(self.way_interval_offsets.len() as u64);
    for (id_range,(byte_offset,byte_len,item_len)) in self.way_interval_offsets.iter() {
      if let (Included(low),Included(high)) = id_range {
        size += varint::length(*low as u64);
        size += varint::length(*high as u64);
        size += varint::length(*byte_offset);
        size += varint::length(*byte_len as u64);
        size += varint::length(*item_len as u64);
      }
    }
    size += varint::length(self.relation_interval_offsets.len() as u64);
    for (id_range,(byte_offset,byte_len,item_len)) in self.relation_interval_offsets.iter() {
      if let (Included(low),Included(high)) = id_range {
        size += varint::length(*low as u64);
        size += varint::length(*high as u64);
        size += varint::length(*byte_offset);
        size += varint::length(*byte_len as u64);
        size += varint::length(*item_len as u64);
      }
    }
    size
  }
  fn count_from_bytes(_buf: &[u8]) -> Result<usize,Error> {
    unimplemented![]
  }
}

impl FromBytes for ScanTable {
  fn from_bytes(buf: &[u8]) -> Result<(usize,Self),Error> {
    let mut table = ScanTable::default();
    let mut offset = 0;
    let (s,node_len) = varint::decode(&buf[offset..])?;
    offset += s;
    for _ in 0..node_len {
      let (s,low) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,high) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,byte_offset) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,byte_len) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,item_len) = varint::decode(&buf[offset..])?;
      offset += s;
      let id_range = (Included(low as i64),Included(high as i64));
      table.node_interval_offsets.insert(
        id_range.clone(),
        (byte_offset, byte_len as usize, item_len as usize)
      );
      table.nodes.insert(id_range);
    }
    let (s,way_len) = varint::decode(&buf[offset..])?;
    offset += s;
    for _ in 0..way_len {
      let (s,low) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,high) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,byte_offset) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,byte_len) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,item_len) = varint::decode(&buf[offset..])?;
      offset += s;
      let id_range = (Included(low as i64),Included(high as i64));
      table.way_interval_offsets.insert(
        id_range.clone(),
        (byte_offset, byte_len as usize, item_len as usize)
      );
      table.ways.insert(id_range);
    }
    let (s,relation_len) = varint::decode(&buf[offset..])?;
    offset += s;
    for _ in 0..relation_len {
      let (s,low) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,high) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,byte_offset) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,byte_len) = varint::decode(&buf[offset..])?;
      offset += s;
      let (s,item_len) = varint::decode(&buf[offset..])?;
      offset += s;
      let id_range = (Included(low as i64),Included(high as i64));
      table.relation_interval_offsets.insert(
        id_range.clone(),
        (byte_offset, byte_len as usize, item_len as usize)
      );
      table.relations.insert(id_range);
    }
    Ok((offset,table))
  }
}
