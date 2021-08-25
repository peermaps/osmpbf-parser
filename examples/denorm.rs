#![feature(generator_trait)]

use std::fs::File;
use osmpbf_denormalize::OsmPbfDenormalize;

type Error = Box<dyn std::error::Error+Send+Sync+'static>;

fn main() -> Result<(),Error> {
  let args = std::env::args().collect::<Vec<String>>();
  let h = File::open(&args[1])?;
  let file_len = h.metadata()?.len();
  let mut opd = OsmPbfDenormalize::open(Box::new(h));
  let mut offset = 0;
  while offset < file_len {
    let (len,items) = opd.read(offset)?;
    println!["{:?}", items];
    offset += len;
  }
  Ok(())
}
