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
  pub nodes: IntervalTree<i64>,
  pub ways: IntervalTree<i64>,
  pub relations: IntervalTree<i64>,
  pub node_interval_offsets: HashMap<(Bound<i64>,Bound<i64>),(u64,usize,usize)>,
  pub way_interval_offsets: HashMap<(Bound<i64>,Bound<i64>),(u64,usize,usize)>,
  pub relation_interval_offsets: HashMap<(Bound<i64>,Bound<i64>),(u64,usize,usize)>,
}
impl Default for ScanTable {
  fn default() -> Self {
    Self {
      nodes: IntervalTree::default(),
      ways: IntervalTree::default(),
      relations: IntervalTree::default(),
      node_interval_offsets: HashMap::new(),
      way_interval_offsets: HashMap::new(),
      relation_interval_offsets: HashMap::new(),
    }
  }
}

impl ScanTable {
  pub fn extend(&mut self, other: &ScanTable) {
    for range in other.nodes.iter() {
      self.nodes.insert(range.clone());
    }
    for range in other.ways.iter() {
      self.ways.insert(range.clone());
    }
    for range in other.relations.iter() {
      self.relations.insert(range.clone());
    }
    self.node_interval_offsets.extend(other.node_interval_offsets.iter());
    self.way_interval_offsets.extend(other.way_interval_offsets.iter());
    self.relation_interval_offsets.extend(other.relation_interval_offsets.iter());
  }
  pub fn get_node_blob_offsets(&self) -> impl Iterator<Item=(u64,usize,usize)>+'_ {
    self.node_interval_offsets.values().cloned()
  }
  pub fn get_node_blob_offsets_for_id(&self, id: i64) -> Vec<(u64,usize,usize)> {
    let q = (Included(id),Included(id));
    self.nodes.get_interval_overlaps(&q).iter()
      .map(|iv| self.node_interval_offsets.get(iv))
      .filter(|o_pair| o_pair.is_some())
      .map(|o_pair| o_pair.unwrap())
      .cloned()
      .collect()
  }
  pub fn get_way_blob_offsets(&self) -> impl Iterator<Item=(u64,usize,usize)>+'_ {
    self.way_interval_offsets.values().cloned()
  }
  pub fn get_way_blob_offsets_for_id(&self, id: i64) -> Vec<(u64,usize,usize)> {
    let q = (Included(id),Included(id));
    self.ways.get_interval_overlaps(&q).iter()
      .map(|iv| self.way_interval_offsets.get(iv))
      .filter(|o_pair| o_pair.is_some())
      .map(|o_pair| o_pair.unwrap())
      .cloned()
      .collect()
  }
  pub fn get_relation_blob_offsets(&self) -> impl Iterator<Item=(u64,usize,usize)>+'_ {
    self.relation_interval_offsets.values().cloned()
  }
  pub fn get_relation_blob_offsets_for_id(&self, id: i64) -> Vec<(u64,usize,usize)> {
    let q = (Included(id),Included(id));
    self.relations.get_interval_overlaps(&q).iter()
      .map(|iv| self.relation_interval_offsets.get(iv))
      .filter(|o_pair| o_pair.is_some())
      .map(|o_pair| o_pair.unwrap())
      .cloned()
      .collect()
  }
}

impl<F> Scan<F> where F: Read+Seek {
  pub fn new(parser: Parser<F>) -> Self {
    Self {
      parser,
      table: ScanTable::default(),
    }
  }
  pub fn from_table(parser: Parser<F>, table: ScanTable) -> Self {
    Self { parser, table }
  }
  pub fn scan(&mut self, start: u64, end: u64) -> Result<(),Error> {
    let mut offset = start;
    while offset < end {
      let (blob_header_len,blob_header) = self.parser.read_blob_header(offset)?;
      let blob_offset = offset + blob_header_len;
      let blob_len = blob_header.datasize as usize;
      let blob = self.parser.read_blob(blob_offset, blob_len)?;
      let len = blob_header_len + blob_len as u64;
      if offset == 0 { // skip header
        offset += len;
        continue;
      }
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
        match etype {
          element::MemberType::Node => {
            self.table.node_interval_offsets.insert(
              iv.clone(),
              (blob_offset,blob_len,items.len())
            );
            self.table.nodes.insert(iv);
          },
          element::MemberType::Way => {
            self.table.way_interval_offsets.insert(
              iv.clone(),
              (blob_offset,blob_len,items.len())
            );
            self.table.ways.insert(iv);
          },
          element::MemberType::Relation => {
            self.table.relation_interval_offsets.insert(
              iv.clone(),
              (blob_offset,blob_len,items.len())
            );
            self.table.relations.insert(iv);
          },
        }
      }
      offset += len;
    }
    Ok(())
  }
  pub fn get_node_blob_offsets(&self) -> impl Iterator<Item=(u64,usize,usize)>+'_ {
    self.table.get_node_blob_offsets()
  }
  pub fn get_node_blob_offsets_for_id(&mut self, id: i64) -> Vec<(u64,usize,usize)> {
    self.table.get_node_blob_offsets_for_id(id)
  }
  pub fn get_node(&mut self, id: i64) -> Result<Option<element::Node>,Error> {
    for (offset,byte_len,_len) in self.get_node_blob_offsets_for_id(id) {
      let blob = self.parser.read_blob(offset,byte_len)?;
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
    Ok(None)
  }
  pub fn get_way_blob_offsets(&mut self) -> impl Iterator<Item=(u64,usize,usize)>+'_ {
    self.table.get_way_blob_offsets()
  }
  pub fn get_way_blob_offsets_for_id(&mut self, id: i64) -> Vec<(u64,usize,usize)> {
    self.table.get_way_blob_offsets_for_id(id)
  }
  pub fn get_way(&mut self, id: i64) -> Result<Option<element::Way>,Error> {
    for (offset,byte_len,_len) in self.get_way_blob_offsets_for_id(id) {
      let blob = self.parser.read_blob(offset,byte_len)?;
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
    Ok(None)
  }
  pub fn get_relation_blob_offsets(&mut self) -> impl Iterator<Item=(u64,usize,usize)>+'_ {
    self.table.get_relation_blob_offsets()
  }
  pub fn get_relation_blob_offsets_for_id(&mut self, id: i64) -> Vec<(u64,usize,usize)> {
    self.table.get_relation_blob_offsets_for_id(id)
  }
  pub fn get_relation(&mut self, id: i64) -> Result<Option<element::Relation>,Error> {
    for (offset,byte_len,_len) in self.get_relation_blob_offsets_for_id(id) {
      let blob = self.parser.read_blob(offset,byte_len)?;
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
    Ok(None)
  }
}
