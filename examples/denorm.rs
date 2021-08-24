use std::fs::File;
use osmpbf_denormalize::OsmPbfDenormalize;

type Error = Box<dyn std::error::Error+Send+Sync+'static>;

fn main() -> Result<(),Error> {
  let args = std::env::args().collect::<Vec<String>>();
  let h = File::open(&args[1])?;
  let mut opd = OsmPbfDenormalize::open(Box::new(h));
  //println!["{:?}", opd.read_header()];
  //println!["{:?}", opd.read_header()];
  opd.scan()?;
  Ok(())
}
