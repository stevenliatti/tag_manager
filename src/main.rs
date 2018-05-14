extern crate clap;
use clap::{App, Arg, ArgGroup};

extern crate xattr;

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
        // TODO: format when multiple tags (push in vector)
        for file in &files {
            if let Some(tags) = xattr::get(file, "user.tag").expect("Unable to get attributes") {
                println!("tags {:?} for file {}", String::from_utf8(tags).unwrap(), file);
            }
        }
    }
        
    // Set case
    if let Some(tags) = matches.values_of("set") {
        let tags: Vec<&str> = tags.collect();
        // TODO: check if existent tag
        for file in &files {
            for tag in &tags {
                xattr::set(file, "user.tag", tag.as_bytes())
                    .expect("An error occurred when setting a tag");
            }
        }
        println!("set tag(s) {:?} for file(s) {:?}", tags, files);
    }

    // Delete case
    if let Some(tags) = matches.values_of("del") {
        let tags: Vec<&str> = tags.collect();
        // TODO: remove only given tags, not the entire extended attribute
        for file in &files {
            xattr::remove(file, "user.tag").expect("An error occurred when deleting a tag");
        }
        println!("del tag(s) {:?} for file(s) {:?}", tags, files);
    }
}
