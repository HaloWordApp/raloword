use config::Config;
use rand::seq::SliceRandom;
use sled::Db;
use url::Url;

use raloword::{impl_rocket, Upstream};

#[derive(Clone)]
pub struct Webster {
  api: Url,
  keys: Vec<String>,
  cache: Db,
}

impl Webster {
  pub fn from_config(conf: &Config) -> Self {
    let keys = conf
      .get_array("webster.keys")
      .unwrap()
      .into_iter()
      .map(|v| v.into_str().unwrap())
      .collect();

    Webster {
      api: Url::parse(&conf.get_str("webster.api").unwrap()).unwrap(),
      keys,
      cache: sled::open("webster.cache").unwrap(),
    }
  }
}

impl Upstream for Webster {
  fn query_url(&self, word: &str) -> Url {
    let mut rng = rand::thread_rng();
    let mut url = self.api.clone();
    url.path_segments_mut().unwrap().pop_if_empty().push(word);
    url
      .query_pairs_mut()
      .append_pair("key", self.keys.choose(&mut rng).unwrap());

    url
  }
}

impl_rocket!(Webster);
