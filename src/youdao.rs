use config::Config;
use reqwest::Response;
use serde::Deserialize;
use sled::Db;
use url::Url;

use raloword::{impl_upstream, Upstream};

#[derive(Deserialize, Debug)]
struct YoudaoResponse {
  #[serde(rename(deserialize = "errorCode"))]
  error_code: i32,
}

#[derive(Clone)]
pub struct Youdao {
  api: Url,
  keyfrom: String,
  key: String,
  cache: Db,
}

impl Youdao {
  pub fn from_config(conf: &Config) -> Self {
    Youdao {
      api: Url::parse(&conf.get_str("youdao.api").unwrap()).unwrap(),
      keyfrom: conf.get_str("youdao.keyfrom").unwrap(),
      key: conf.get_str("youdao.key").unwrap(),
      cache: sled::open("youdao.cache").unwrap(),
    }
  }
}

impl Upstream for Youdao {
  fn query_url(&self, word: &str) -> Url {
    let mut url = self.api.clone();
    url
      .query_pairs_mut()
      .append_pair("keyfrom", &self.keyfrom)
      .append_pair("key", &self.key)
      .append_pair("q", word);

    url
  }

  fn valid_response(resp: &mut Response) -> bool {
    // check for embedded error
    if let Ok(ydr) = resp.json::<YoudaoResponse>() {
      ydr.error_code == 0
    } else {
      false
    }
  }
}

impl_upstream!(Youdao);
