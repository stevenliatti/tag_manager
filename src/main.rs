extern crate tag_manager;
extern crate clap;
use clap::{App, Arg, ArgGroup};

fn main() {
    // TODO: improve help usage
    let matches = App::new("tag_manager").version("0.1.0").about("Manage your tags")
        .group(ArgGroup::with_name("ops").args(&["get", "set", "del"]))
        .arg(Arg::with_name("get").help("Get tag(s)").short("g").long("get"))
        .arg(Arg::with_name("set").help("Set tag(s)").short("s").long("set")
            .takes_value(true).multiple(true))
        .arg(Arg::with_name("del").help("Delete tag(s)").short("d").long("del")
            .takes_value(true).multiple(true))
        .arg(Arg::with_name("files").help("File(s) name(s)").short("f").long("files")
            .takes_value(true).multiple(true).required(true))
        .get_matches();

    let files: Vec<&str> = matches.values_of("files").unwrap().collect();
    if matches.is_present("get") { tag_manager::get_tags(&files); }
    if let Some(tags) = matches.values_of("set") { tag_manager::set_tags(&files, tags); }
    if let Some(tags) = matches.values_of("del") { tag_manager::del_tags(&files, tags); }
}
