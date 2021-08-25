pub type Tags = Vec<(String,String)>;

#[derive(Debug,Clone)]
pub enum Element {
  Node(Node),
  Way(Way),
  Relation(Relation),
}

#[derive(Debug,Clone)]
pub struct Info {
  pub version: i32,
  pub timestamp: Option<i64>,
  pub changeset: Option<i64>,
  pub uid: Option<i32>,
  pub user: Option<String>,
  pub visible: Option<bool>,
}

#[derive(Debug,Clone)]
pub struct Node {
  pub id: i64,
  pub tags: Tags,
  pub info: Option<Info>,
  pub lon: f64,
  pub lat: f64,
}

#[derive(Debug,Clone)]
pub struct Way {
  pub id: i64,
  pub tags: Tags,
  pub info: Option<Info>,
  pub refs: Vec<i64>,
}

#[derive(Debug,Clone)]
pub struct Relation {
  pub id: i64,
  pub tags: Tags,
  pub info: Option<Info>,
  pub members: Vec<Member>,
}

#[derive(Debug,Clone)]
pub struct Member {
  pub id: i64,
  pub role: String,
  pub member_type: MemberType,
}

#[derive(Debug,Clone)]
pub enum MemberType {
  Node,
  Way,
  Relation,
}
