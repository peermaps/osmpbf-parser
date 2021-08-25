use std::fs::File;
use osmpbf_denormalize::OsmPbfDenormalize;

type Error = Box<dyn std::error::Error+Send+Sync+'static>;

fn main() -> Result<(),Error> {
  let args = std::env::args().collect::<Vec<String>>();
  let h = File::open(&args[1])?;
  let file_len = h.metadata()?.len();
  let mut opd = OsmPbfDenormalize::open(Box::new(h));
  let mut scan = opd.scan(0, file_len)?;
  let start = std::time::Instant::now();
  /*
  let node_id = args[2].parse().unwrap();
  if let Some(node) = scan.get_node(node_id)? {
    println!["node={:?}", &node];
  }
  */
  let way_id = args[2].parse().unwrap();
  if let Some(way) = scan.get_way(way_id)? {
    let elapsed = start.elapsed().as_secs_f64();
    let mut nodes = vec![];
    for r in way.refs.iter() {
      nodes.push(scan.get_node(*r)?.unwrap());
    }
    println!["{:.6} way={:?}", elapsed, &way];
    for node in nodes {
      println!["  node={:?}", &node];
    }
  }
  Ok(())
}
