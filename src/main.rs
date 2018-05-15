extern crate xattr;
extern crate clap;
use clap::{App, Arg, ArgGroup};

const ATTR_NAME : &str = "user.tags";
const COMMA_ASCII_VALUE : u8 = 44;

// TODO: where put type annotations in println ?
fn str_to_vec(string: &str) -> Vec<&str> {
    string.split(",").collect()
}

fn main() {
    // TODO: improve help usage
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
        for &file in &files {
            match xattr::get(file, ATTR_NAME) {
                Ok(res) => match res {
                    Some(tags) => println!("Tags {:?} for file \"{}\"",
                        str_to_vec(&String::from_utf8(tags).unwrap()), file),
                    None => println!("File \"{}\" has no tags", file)
                },
                Err(err) => println!("Error for file \"{}\" : {}", file, err)
            }
        }
    }

    // Set case
    if let Some(tags) = matches.values_of("set") {
        let mut tags_args: Vec<&str> = tags.collect();

        let mut new_tags_u8 : Vec<u8> = Vec::new();
        for tag in &tags_args {
            for u in tag.bytes() {
                new_tags_u8.push(u);
            }
            new_tags_u8.push(COMMA_ASCII_VALUE);
        }

        // TODO: if existent tag, do not duplicate
        for &file in &files {
            let mut update_tags_u8 = new_tags_u8.clone();
            match xattr::get(file, ATTR_NAME) {
                Ok(res) => match res {
                    Some(get_tags) => 
                        for tag in get_tags {
                            update_tags_u8.push(tag);
                        },
                    None => { update_tags_u8.pop(); }
                },
                Err(err) => println!("Error for file \"{}\" : {}", file, err)
            }

            xattr::set(file, ATTR_NAME, &update_tags_u8).expect("Error when setting tags");
            println!("Add/update tag(s) {:?} for file(s) {:?}", tags_args, files);
        }
    }

    // Delete case
    if let Some(tags) = matches.values_of("del") {
        let tags: Vec<&str> = tags.collect();
        // TODO: remove only given tags, not the entire extended attribute
        for &file in &files {
            xattr::remove(file, ATTR_NAME).expect("An error occurred when deleting a tag");
        }
        println!("del tag(s) {:?} for file(s) {:?}", tags, files);
    }
}
