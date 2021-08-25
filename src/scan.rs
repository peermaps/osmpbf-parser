use std::collections::HashMap;
use crate::{OsmPbfDenormalize,element,Element,Error};
use unbounded_interval_tree::IntervalTree;
use std::ops::{Bound::Included,Bound};
use std::io::{Read,Seek};

pub struct Scan<'a, F: Read+Seek> {
  parser: &'a mut OsmPbfDenormalize<F>,
  nodes: IntervalTree<i64>,
  ways: IntervalTree<i64>,
  relations: IntervalTree<i64>,
  interval_offsets: HashMap<(Bound<i64>,Bound<i64>),(u64,usize)>,
}

impl<'a,F> Scan<'a,F> where F: Read+Seek {
  pub fn scan(parser: &'a mut OsmPbfDenormalize<F>, start: u64, end: u64) -> Result<Self,Error> {
    let mut result = Self {
      parser,
      nodes: IntervalTree::default(),
      ways: IntervalTree::default(),
      relations: IntervalTree::default(),
      interval_offsets: HashMap::new(),
    };
    let mut offset = start;
    while offset < end {
      let (blob_header_len,blob_header) = result.parser.read_blob_header(offset)?;
      let blob = result.parser.read_blob(offset + blob_header_len, blob_header.datasize as usize)?;

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
        result.interval_offsets.insert(iv.clone(), (blob_offset,blob_len));
        match etype {
          element::MemberType::Node => {
            result.nodes.insert(iv);
          },
          element::MemberType::Way => {
            result.ways.insert(iv);
          },
          element::MemberType::Relation => {
            result.relations.insert(iv);
          },
        }
      }
      offset += len;
    }
    Ok(result)
  }
  pub fn get_node(&mut self, id: i64) -> Result<Option<element::Node>,Error> {
    let q = (Included(id),Included(id));
    for iv in self.nodes.get_interval_overlaps(&q).iter() {
      if let Some((offset,len)) = self.interval_offsets.get(&iv) {
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
    for iv in self.ways.get_interval_overlaps(&q).iter() {
      if let Some((offset,len)) = self.interval_offsets.get(&iv) {
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
    for iv in self.relations.get_interval_overlaps(&q).iter() {
      if let Some((offset,len)) = self.interval_offsets.get(&iv) {
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

impl<F> OsmPbfDenormalize<F> where F: Read+Seek {
  pub fn scan<'a>(&'a mut self, start: u64, end: u64) -> Result<Scan<'a,F>,Error> {
    Scan::scan(self, start, end)
  }
}
