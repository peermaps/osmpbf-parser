use std::fs::File;
use osmpbf_parser::{Parser,Element};

type Error = Box<dyn std::error::Error+Send+Sync+'static>;

fn main() -> Result<(),Error> {
  let args = std::env::args().collect::<Vec<String>>();
  let h = File::open(&args[1])?;
  let file_len = h.metadata()?.len();
  let mut opd = Parser::new(Box::new(h));
  let mut offset = 0;
  while offset < file_len {
    let (byte_len,items) = opd.read(offset)?;
    let mut etype = "";
    let mut min_id = i64::MAX;
    let mut max_id = i64::MIN;
    for item in items.iter() {
      match item {
        Element::Node(node) => {
          etype = "node";
          min_id = node.id.min(min_id);
          max_id = node.id.max(max_id);
        },
        Element::Way(way) => {
          etype = "way";
          min_id = way.id.min(min_id);
          max_id = way.id.max(max_id);
        },
        Element::Relation(relation) => {
          etype = "relation";
          min_id = relation.id.min(min_id);
          max_id = relation.id.max(max_id);
        },
      }
    }
    if !items.is_empty() {
      println!["{}: offset={} byte_len={} items.len()={} id range {}..{}",
        etype, offset, byte_len, items.len(), min_id, max_id];
    }
    offset += byte_len;
  }
  Ok(())
}
