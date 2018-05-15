extern crate xattr;
extern crate clap;
use clap::{App, Arg, ArgGroup};

const ATTR_NAME : &str = "user.tags";
const COMMA_ASCII_VALUE : u8 = ',' as u8;

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
        let mut tags: Vec<&str> = tags.collect();
        let mut new_tags_u8 : Vec<u8> = Vec::new();

        // Convert str tags to bytes
        for tag in &tags {
            for u in tag.bytes() {
                new_tags_u8.push(u);
            }
            new_tags_u8.push(COMMA_ASCII_VALUE);
        }

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

            let mut s = String::new();
            let mut full_tags_str : Vec<String> = Vec::new();
            for u in update_tags_u8.clone() {
                if u == COMMA_ASCII_VALUE {
                    full_tags_str.push(s.to_string());
                    s = String::new();
                }
                else {
                    s.push(u as char);
                }
            }
            full_tags_str.push(s.to_string());
            full_tags_str.sort();
            full_tags_str.dedup();

            let mut final_tags : Vec<u8> = Vec::new();
            // Convert str tags to bytes
            for tag in &full_tags_str {
                for u in tag.bytes() {
                    final_tags.push(u);
                }
                final_tags.push(COMMA_ASCII_VALUE);
            }
            final_tags.pop();
            xattr::set(file, ATTR_NAME, &final_tags).expect("Error when setting tag(s)");
        }
        println!("Add tag(s) {:?} for file(s) {:?}", tags, files);
    }

    // Delete case
    if let Some(tags) = matches.values_of("del") {
        let tags_to_del: Vec<&str> = tags.collect();
        let mut new_tags_u8 : Vec<u8> = Vec::new();

        // Convert str tags to bytes
        for tag in &tags_to_del {
            for u in tag.bytes() {
                new_tags_u8.push(u);
            }
            new_tags_u8.push(COMMA_ASCII_VALUE);
        }

        for &file in &files {
            let mut existent_tags = Vec::new();
            match xattr::get(file, ATTR_NAME) {
                Ok(res) => match res {
                    Some(get_tags) =>
                        for tag in get_tags {
                            existent_tags.push(tag);
                        },
                    None => println!("File \"{}\" has no tags", file)
                },
                Err(err) => {
                    println!("Error for file \"{}\" : {}", file, err);
                    return;
                }
            }

            // Compute a Vec<String> to check existent tags
            let mut s = String::new();
            let mut existent_tags_str : Vec<String> = Vec::new();
            for u in existent_tags {
                if u == COMMA_ASCII_VALUE {
                    existent_tags_str.push(s.to_string());
                    s = String::new();
                }
                else {
                    s.push(u as char);
                }
            }
            existent_tags_str.push(s.to_string());

            // Retain tags that must be effectively deleted
            for tag in &tags_to_del {
                existent_tags_str.retain(|ref e| e != &&tag.to_string());
            }

            // Delete the tags
            xattr::remove(file, ATTR_NAME).expect("Error when deleting tag(s)");

            // Readd the tags that are not in tags_to_del
            let mut update_tags_u8 : Vec<u8> = Vec::new();
            for tag in &existent_tags_str {
                for u in tag.bytes() {
                    update_tags_u8.push(u);
                }
                update_tags_u8.push(COMMA_ASCII_VALUE);
            }
            // remove last comma
            update_tags_u8.pop();
            xattr::set(file, ATTR_NAME, &update_tags_u8).expect("Error when (re)setting tag(s)");
        }
        println!("Delete tag(s) {:?} for file(s) {:?}", tags_to_del, files);
    }
}
