use config::Config;
use log::{error, trace};
use rand::seq::SliceRandom;
use rocket::handler::{Handler, Outcome};
use rocket::{http::Method, http::Status, Data, Request, Route};
use sled::Db;
use url::Url;

use raloword::Upstream;

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

impl Handler for Webster {
  fn handle<'r>(&self, req: &'r Request, _: Data) -> Outcome<'r> {
    match req.get_param(1) {
      Some(rword) => {
        let word: String = rword.unwrap();

        match self.cache.get(&word) {
          Ok(Some(cached_resp)) => {
            trace!("found in cache: {}", &word);
            Outcome::from(req, std::str::from_utf8(&cached_resp).unwrap().to_string())
          }
          Ok(None) => {
            trace!("not found in cache: {}", &word);
            let url = self.query_url(&word).into_string();
            trace!("query Webster: {}, {}", &word, &url);
            let resp = reqwest::get(&url).unwrap().text().unwrap();
            trace!("attempt to cache: {}", &word);
            if let Err(e) = self.cache.insert(word.as_bytes(), resp.as_bytes()) {
              error!("failed to insert Webster cache: {}, {}, {}", &word, &resp, e);
            }
            Outcome::from(req, resp)
          }
          Err(e) => panic!(e),
        }
      }
      None => Outcome::Failure(Status::BadRequest),
    }
  }
}

impl Into<Vec<Route>> for Webster {
  fn into(self) -> Vec<Route> {
    vec![Route::new(Method::Get, "/query/<word>", self)]
  }
}
