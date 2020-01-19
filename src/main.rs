use config as app_config;
use rocket::config::{Config, Environment};

mod webster;
mod youdao;

fn main() {
  env_logger::init();

  let mut conf = app_config::Config::default();
  conf.merge(app_config::File::with_name("config")).unwrap();

  let rc = Config::build(Environment::Production)
    .address("localhost")
    .port(8123)
    .finalize()
    .unwrap();

  rocket::custom(rc)
    .mount("/youdao", youdao::Youdao::from_config(&conf))
    .mount("/webster", webster::Webster::from_config(&conf))
    .launch();
}
