use reqwest::Response;
use url::Url;

pub trait Upstream {
  fn query_url(&self, _: &str) -> Url;
  fn valid_response(_: &mut Response) -> bool {
    true
  }
}

#[macro_export]
macro_rules! impl_rocket {
  ( $typ:ident ) => {
    impl rocket::handler::Handler for $typ {
      fn handle<'r>(&self, req: &'r rocket::Request, _: rocket::Data) -> rocket::handler::Outcome<'r> {
        use log::{error, trace};
        use reqwest::Response;
        use rocket::handler::{Handler, Outcome};
        use rocket::{http::Status, Data, Request, Route};

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
                trace!("query: {}, {}", &word, &url);
                let mut resp = reqwest::get(&url).unwrap();
                let ctnt = resp.text().unwrap();
                if $typ::valid_response(&mut resp) {
                  trace!("attempt to cache: {}", &word);
                  if let Err(e) = self.cache.insert(word.as_bytes(), ctnt.as_bytes()) {
                    error!("failed to cache: {}, {}, {}", &word, &ctnt, e);
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

    impl Into<Vec<rocket::Route>> for $typ {
      fn into(self) -> Vec<rocket::Route> {
        vec![rocket::Route::new(rocket::http::Method::Get, "/query/<word>", self)]
      }
    }
  };
}
