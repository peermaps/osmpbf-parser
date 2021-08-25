use crate::proto;
use crate::proto::osmformat::{HeaderBlock,PrimitiveBlock};
use quick_protobuf::{MessageRead,Reader};
use flate2::read::ZlibDecoder;
use std::io::Read;
use crate::{element,Blob};

type Error = Box<dyn std::error::Error+Send+Sync+'static>;

impl Blob {
  pub fn decode_header(&self) -> Result<HeaderBlock,Error> {
    let data = self.get_data()?;
    let header_block = Reader::from_bytes(data)
      .read(HeaderBlock::from_reader)?;
    Ok(header_block)
  }
  pub fn decode_primitive(&self) -> Result<PrimitiveBlock,Error> {
    let data = self.get_data()?;
    let primitive_block = Reader::from_bytes(data)
      .read(PrimitiveBlock::from_reader)?;
    Ok(primitive_block)
  }
  pub fn get_data(&self) -> Result<Vec<u8>,Error> {
    //if let Some(data) = &self.raw {
    //  Reader::from_bytes(data.clone())
    //} else if let Some(input) = &self.zlib_data {
    if let Some(input) = &self.zlib_data {
      let mut z = ZlibDecoder::new(&input[..]);
      let mut data = vec![];
      z.read_to_end(&mut data)?;
      Ok(data)
    } else {
      panic!["unsupported compression type"];
    }
  }
}

impl PrimitiveBlock {
  pub fn decode(&self) -> Vec<element::Element> {
    let mut elements = vec![];
    for g in self.primitivegroup.iter() {
      for node in g.nodes.iter() {
        elements.push(element::Element::Node(element::Node {
          id: node.id,
          tags: self.tags(&node.keys, &node.vals),
          info: node.info.as_ref().map(|info| self.info(info)),
          lon: (self.lon_offset + (self.granularity as i64 * node.lon)) as f64 * 1e-9,
          lat: (self.lat_offset + (self.granularity as i64 * node.lat)) as f64 * 1e-9,
        }));
      }
      if let Some(dense) = &g.dense {
        let mut prev_id = 0;
        let mut prev_lon = 0;
        let mut prev_lat = 0;
        let mut prev_timestamp = 0;
        let mut prev_changeset = 0;
        let mut prev_uid = 0;
        let mut prev_user_sid = 0;
        let mut tag_i = 0;
        let mut info_i = 0;
        let z = dense.id.iter().zip(dense.lon.iter().zip(dense.lat.iter()));
        for (d_id,(d_lon,d_lat)) in z {
          let id = *d_id + prev_id;
          let lon = *d_lon + prev_lon;
          let lat = *d_lat + prev_lat;

          let mut tags = vec![];
          while tag_i+1 < dense.keys_vals.len() && dense.keys_vals[tag_i] != 0 {
            tags.push((
              self.get_string(dense.keys_vals[tag_i+0] as usize),
              self.get_string(dense.keys_vals[tag_i+1] as usize),
            ));
            tag_i += 2;
          }
          tag_i += 1;
          let info = dense.denseinfo.as_ref().map(|info| {
            let timestamp = info.timestamp.get(info_i).map(|x| prev_timestamp+*x);
            let changeset = info.changeset.get(info_i).map(|x| prev_changeset+*x);
            let uid = info.uid.get(info_i).map(|x| prev_uid+*x);
            let user_sid = info.user_sid.get(info_i).map(|x| prev_user_sid+*x);
            let einfo = element::Info {
              version: info.version.get(info_i).cloned().unwrap_or(0),
              timestamp,
              changeset,
              uid,
              user: user_sid.map(|i| self.get_string(i as usize)),
              visible: info.visible.get(info_i).cloned(),
            };
            info_i += 1;
            prev_timestamp = timestamp.unwrap_or(0);
            prev_changeset = changeset.unwrap_or(0);
            prev_uid = uid.unwrap_or(0);
            prev_user_sid = user_sid.unwrap_or(0);
            einfo
          });
          elements.push(element::Element::Node(element::Node {
            id: id,
            tags,
            info,
            lon: (self.lon_offset + (self.granularity as i64 * lon)) as f64 * 1e-9,
            lat: (self.lat_offset + (self.granularity as i64 * lat)) as f64 * 1e-9,
          }));
          prev_id = id;
          prev_lon = lon;
          prev_lat = lat;
        }
      }
      for way in g.ways.iter() {
        let mut refs = vec![];
        let mut prev_ref = 0;
        for r in way.refs.iter() {
          //refs.push(r + prev_ref);
          refs.push(prev_ref + *r);
          prev_ref += r;
        }
        elements.push(element::Element::Way(element::Way {
          id: way.id,
          tags: self.tags(&way.keys, &way.vals),
          info: way.info.as_ref().map(|info| self.info(info)),
          refs,
        }));
      }
      for relation in g.relations.iter() {
        let mut members = vec![];
        let mut prev_mem_id = 0;
        let z = relation.memids.iter().zip(relation.roles_sid.iter()).zip(relation.types.iter());
        for ((mem_id,role_sid),mem_type) in z {
          members.push(element::Member {
            id: mem_id + prev_mem_id,
            role: self.get_string(*role_sid as usize),
            member_type: match mem_type {
              proto::osmformat::mod_Relation::MemberType::NODE => element::MemberType::Node,
              proto::osmformat::mod_Relation::MemberType::WAY => element::MemberType::Way,
              proto::osmformat::mod_Relation::MemberType::RELATION => element::MemberType::Relation,
            },
          });
          prev_mem_id = mem_id + prev_mem_id;
        }
        elements.push(element::Element::Relation(element::Relation {
          id: relation.id,
          tags: self.tags(&relation.keys, &relation.vals),
          info: relation.info.as_ref().map(|info| self.info(info)),
          members,
        }));
      }
    }
    elements
  }
  fn tags<'a>(&self, keys: &[u32], values: &[u32]) -> element::Tags {
    keys.iter().zip(values.iter()).map(|(ki,vi)| {
      let key = self.get_string(*ki as usize);
      let value = self.get_string(*vi as usize);
      (key, value)
    }).collect()
  }
  fn info(&self, info: &proto::osmformat::Info) -> element::Info {
    element::Info {
      version: info.version,
      timestamp: info.timestamp,
      changeset: info.changeset,
      uid: info.uid,
      user: info.user_sid.map(|i| self.get_string(i as usize)),
      visible: info.visible,
    }
  }
  pub fn get_string(&self, i: usize) -> String {
    let s = &self.stringtable.s[i];
    String::from_utf8(s.to_vec()).unwrap()
  }
}
