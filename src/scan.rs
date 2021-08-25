use std::collections::HashMap;
use crate::{OsmPbfDenormalize,element,Element,Error};
use unbounded_interval_tree::IntervalTree;
use std::ops::{Bound::Included,Bound};
use std::io::{Read,Seek};

pub struct Scan<'a, F: Read+Seek> {
  parser: &'a mut OsmPbfDenormalize<F>,
  nodes: IntervalTree<u64>,
  ways: IntervalTree<u64>,
  relations: IntervalTree<u64>,
  interval_offsets: HashMap<(Bound<u64>,Bound<u64>),u64>,
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
      let (len,items) = result.parser.read(offset)?;
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
        let iv = (Included(min_id as u64),Included(max_id as u64));
        result.interval_offsets.insert(iv.clone(), offset);
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
  pub fn get_node(&mut self, id: u64) -> Result<Option<element::Node>,Error> {
    let q = (Included(id),Included(id));
    for iv in self.nodes.get_interval_overlaps(&q).iter() {
      if let Some(offset) = self.interval_offsets.get(&iv) {
        let (_len,items) = self.parser.read(*offset)?;
        for item in items {
          match item {
            Element::Node(node) => {
              if node.id as u64 == id { return Ok(Some(node)) }
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
