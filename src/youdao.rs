use config::Config;
use log::{error, trace};
use reqwest::{self, Response};
use rocket::handler::{Handler, Outcome};
use rocket::{http::Method, http::Status, Data, Request, Route};
use serde::Deserialize;
use sled::Db;
use url::Url;

use raloword::Upstream;

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

impl Handler for Youdao {
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
            trace!("query Youdao: {}", &word);
            let mut resp = reqwest::get(&url).unwrap();
            let ctnt = resp.text().unwrap();
            if Youdao::valid_response(&mut resp) {
              trace!("attempt to cache: {}", &word);
              if let Err(e) = self.cache.insert(word.as_bytes(), ctnt.as_bytes()) {
                error!("failed to insert Youdao cache: {}, {}, {}", &word, &ctnt, e);
              }
            }
            Outcome::from(req, ctnt)
          }
          Err(e) => panic!(e),
        }
      }
      None => Outcome::Failure(Status::BadRequest),
    }
  }
}

impl Into<Vec<Route>> for Youdao {
  fn into(self) -> Vec<Route> {
    vec![Route::new(Method::Get, "/query/<word>", self)]
  }
}
