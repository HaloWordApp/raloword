use std::sync::{Arc, Mutex};

use config::Config;
use reqwest::{self, Response};
use rocket::handler::{Handler, Outcome};
use rocket::{http::Method, http::Status, Data, Request, Route};
use rocksdb::DB;
use serde::Deserialize;
use url::Url;

use raloword::Upstream;

#[derive(Deserialize, Debug)]
struct YoudaoResponse {
    #[serde(rename(deserialize = "errorCode"))]
    error_code: i32,
}

#[derive(Clone)]
pub struct Youdao {
    base_url: Url,
    keyfrom: String,
    key: String,
    cache: Arc<Mutex<DB>>,
}

impl Youdao {
    pub fn from_config(conf: &Config) -> Self {
        let db = DB::open_default("youdao.cache").unwrap();
        Youdao {
            base_url: Url::parse(&conf.get_str("youdao.base_url").unwrap()).unwrap(),
            keyfrom: conf.get_str("youdao.keyfrom").unwrap(),
            key: conf.get_str("youdao.key").unwrap(),
            cache: Arc::new(Mutex::new(db)),
        }
    }
}

impl Upstream for Youdao {
    fn query_url(&self, word: &str) -> Url {
        let mut url = self.base_url.clone();
        url.query_pairs_mut()
            .append_pair("keyfrom", &self.keyfrom)
            .append_pair("key", &self.key)
            .append_pair("q", word);

        url
    }

    fn valid_response(_: Response) -> bool {
        true
    }
}

impl Handler for Youdao {
    fn handle<'r>(&self, req: &'r Request, _: Data) -> Outcome<'r> {
        match req.get_param(1) {
            Some(rword) => {
                let word: String = rword.unwrap();

                let db = self.cache.lock().unwrap();
                match (*db).get(&word) {
                    Ok(Some(cached_resp)) => {
                        println!("reading cache");
                        let resp = cached_resp.to_utf8().unwrap().to_string();
                        Outcome::from(req, resp)
                    }
                    Ok(None) => {
                        let url = self.query_url(&word).into_string();
                        let resp = reqwest::get(&url).unwrap().text().unwrap();
                        (*db).put(&word, &resp).unwrap();
                        Outcome::from(req, resp)
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

// fn fetch_youdao() -> Option<YoudaoResponse> {

// }

// #[get("/youdao/query/<word>")]
// fn youdao(word: String) -> String {
//     match fetch_youdao() {
//         Some(resp) => format!("body = {:?}", resp.error_code),
//         _ => "test".to_string(),
//     }
// }
