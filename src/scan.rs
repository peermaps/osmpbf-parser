use std::collections::HashMap;
use crate::{Parser,element,Element,Error};
use unbounded_interval_tree::IntervalTree;
use std::ops::{Bound::Included,Bound};
use std::io::{Read,Seek};

pub struct Scan<F: Read+Seek> {
  pub parser: Parser<F>,
  pub table: ScanTable,
}

#[derive(Debug,Clone)]
pub struct ScanTable {
  nodes: IntervalTree<i64>,
  ways: IntervalTree<i64>,
  relations: IntervalTree<i64>,
  interval_offsets: HashMap<(Bound<i64>,Bound<i64>),(u64,usize)>,
}
impl Default for ScanTable {
  fn default() -> Self {
    Self {
      nodes: IntervalTree::default(),
      ways: IntervalTree::default(),
      relations: IntervalTree::default(),
      interval_offsets: HashMap::new(),
    }
  }
}

impl<F> Scan<F> where F: Read+Seek {
  pub fn new(parser: Parser<F>) -> Self {
    Self {
      parser,
      table: ScanTable::default(),
    }
  }
  pub fn scan(&mut self, start: u64, end: u64) -> Result<(),Error> {
    let mut offset = start;
    while offset < end {
      let (blob_header_len,blob_header) = self.parser.read_blob_header(offset)?;
      let blob = self.parser.read_blob(offset + blob_header_len, blob_header.datasize as usize)?;

      let blob_offset = offset + blob_header_len;
      let blob_len = blob_header.datasize as usize;
      let len = blob_header_len + blob_len as u64;

      let items = blob.decode_primitive()?.decode();

      let mut etype = element::MemberType::Node;
      let mut min_id = i64::MAX;
      let mut max_id = i64::MIN;
      for item in items.iter() {
        match item {
          Element::Node(node) => {
            min_id = node.id.min(min_id);
            max_id = node.id.max(max_id);
          },
          Element::Way(way) => {
            etype = element::MemberType::Way;
            min_id = way.id.min(min_id);
            max_id = way.id.max(max_id);
          },
          Element::Relation(relation) => {
            etype = element::MemberType::Relation;
            min_id = relation.id.min(min_id);
            max_id = relation.id.max(max_id);
          },
        }
      }
      if !items.is_empty() {
        let iv = (Included(min_id),Included(max_id));
        self.table.interval_offsets.insert(iv.clone(), (blob_offset,blob_len));
        match etype {
          element::MemberType::Node => {
            self.table.nodes.insert(iv);
          },
          element::MemberType::Way => {
            self.table.ways.insert(iv);
          },
          element::MemberType::Relation => {
            self.table.relations.insert(iv);
          },
        }
      }
      offset += len;
    }
    Ok(())
  }
  pub fn get_node(&mut self, id: i64) -> Result<Option<element::Node>,Error> {
    let q = (Included(id),Included(id));
    for iv in self.table.nodes.get_interval_overlaps(&q).iter() {
      if let Some((offset,len)) = self.table.interval_offsets.get(&iv) {
        let blob = self.parser.read_blob(*offset,*len)?;
        let items = blob.decode_primitive()?.decode();
        for item in items {
          match item {
            Element::Node(node) => {
              if node.id == id { return Ok(Some(node)) }
            },
            _ => { break }
          }
        }
      }
    }
    Ok(None)
  }
  pub fn get_way(&mut self, id: i64) -> Result<Option<element::Way>,Error> {
    let q = (Included(id),Included(id));
    for iv in self.table.ways.get_interval_overlaps(&q).iter() {
      if let Some((offset,len)) = self.table.interval_offsets.get(&iv) {
        let blob = self.parser.read_blob(*offset,*len)?;
        let items = blob.decode_primitive()?.decode();
        for item in items {
          match item {
            Element::Way(way) => {
              if way.id == id { return Ok(Some(way)) }
            },
            _ => { break }
          }
        }
      }
    }
    Ok(None)
  }
  pub fn get_relation(&mut self, id: i64) -> Result<Option<element::Relation>,Error> {
    let q = (Included(id),Included(id));
    for iv in self.table.relations.get_interval_overlaps(&q).iter() {
      if let Some((offset,len)) = self.table.interval_offsets.get(&iv) {
        let blob = self.parser.read_blob(*offset,*len)?;
        let items = blob.decode_primitive()?.decode();
        for item in items {
          match item {
            Element::Relation(relation) => {
              if relation.id == id { return Ok(Some(relation)) }
            },
            _ => { break }
          }
        }
      }
    }
    Ok(None)
  }
}
