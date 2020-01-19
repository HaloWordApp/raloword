use reqwest::Response;
use url::Url;

pub trait Upstream {
  fn query_url(&self, _: &str) -> Url;
  fn valid_response(_: &mut Response) -> bool {
    true
  }
}
