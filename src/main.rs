extern crate clap;
use clap::{App, Arg, ArgGroup};

fn main() {
    let matches = App::new("tag_manager").version("0.1.0").about("Manage your tags")
        .group(ArgGroup::with_name("ops").args(&["get", "set", "del"]))
        .arg(Arg::with_name("get")
            .help("Get tag(s)")
            .short("g")
            .long("get"))
        .arg(Arg::with_name("set")
            .help("Set tag(s)")
            .short("s")
            .long("set")
            .takes_value(true)
            .multiple(true))
        .arg(Arg::with_name("del")
            .help("Delete tag(s)")
            .short("d")
            .long("del")
            .takes_value(true)
            .multiple(true))
        .arg(Arg::with_name("files")
            .help("File(s) name(s)")
            .short("f")
            .long("files")
            .takes_value(true)
            .required(true)
            .multiple(true))
        .get_matches();

    let files: Vec<&str> = matches.values_of("files").unwrap().collect();
    
    // Get case
    if matches.is_present("get") {
        println!("get tag(s) for file(s) {:?}", files);
    }
        
    // Set case
    if let Some(tags) = matches.values_of("set") {
        let tags: Vec<&str> = tags.collect();
        println!("set tag(s) {:?} for file(s) {:?}", tags, files);
    }

    // Delete case
    if let Some(tags) = matches.values_of("del") {
        let tags: Vec<&str> = tags.collect();
        println!("del tag(s) {:?} for file(s) {:?}", tags, files);
    }
}
