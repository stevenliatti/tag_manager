extern crate clap;
use clap::App;

fn main() {
  App::new("tag_manager").version("0.1.0").about("Manage your tags").get_matches();
}
