//! # Tag Manager
//! Little CLI tool for getting, setting and deleting tags for files and folders.
//! The tags are stored in an extended attribute called "user.tags" and separated by comma.
//! Run `tag_manager -h` to see help.

extern crate tag_manager;
extern crate clap;
use clap::{App, Arg, ArgGroup};
use std::fs;
use std::collections::HashSet;

use std::os::unix::net::UnixStream;
use std::io::prelude::*;

const SOCKET_ADDRESS : &str = "/tmp/tag_engine";
const CODE_ENTRIES : &str = "0x0";
const CODE_TAGS : &str = "0x1";
// const CODE_RENAME_TAG : &str = "0x2";

fn main() {
    // TODO: update help
    let help = "\
        tag_manager v0.1.0\nManage your tags\nBy default, this tool store your tags \
        in the extended attribute\n\"user.tags\" and separe them by a comma (\",\").\n\n\
        Usage:\n    tag_manager [Options] <files> [[--set|--del] <tags>]\n\n\
        Options:\n    \
        -h    Display this message\n    \
        -r    Recursive option. Get, set or delete tags for each folder and file in folder subtree\n\n\
        Arguments:\n    \
        -s, --set <tags>       Set the given tags\n    \
        -d, --del <tags>       Delete the given tags\n\n\
        Examples:\n    \
        tag_manager myfile                  => Show the actual tags of file \"myfile\"\n    \
        tag_manager myfile -s work          => Set the tag \"work\" to the file \"myfile\"\n    \
        tag_manager myfile -d work          => Delete the tag \"work\" to the file \"myfile\"\n    \
        tag_manager myfolder -r -s geneva   => Set the tag \"geneva\" to the folder \"myfolder\" and his subtree";
    let matches = App::new("tag_manager")
        .help(help)
        .group(ArgGroup::with_name("ops").args(&["set", "del"]))
        .arg(Arg::with_name("set").short("s").long("set").takes_value(true).multiple(true))
        .arg(Arg::with_name("del").short("d").long("del").takes_value(true).multiple(true))
        .arg(Arg::with_name("files").takes_value(true).multiple(true).required(false))
        .arg(Arg::with_name("recursive").short("-r").long("--recursive"))
        .group(ArgGroup::with_name("queries").args(&["list", "query"]))
        .arg(Arg::with_name("query").short("-q").long("--query").takes_value(true))
        .arg(Arg::with_name("list").short("-l").long("--list"))
        .get_matches();

    if matches.is_present("files") {
        let files: Vec<&str> = matches.values_of("files").unwrap().collect();
        let recursive = matches.is_present("recursive");

        if !matches.is_present("set") && !matches.is_present("del") {
            for file in &files { show_tags(file, recursive); }
        }

        if matches.is_present("set") && matches.is_present("files") {
            let tags: HashSet<&str> = matches.values_of("set").unwrap().collect();
            let tags = &hash_set_str_to_hash_set_string(&tags);
            for file in &files {
                tag_manager::set_tags(file, tags, recursive);
            }
        }

        if matches.is_present("del") && matches.is_present("files") {
            let tags : HashSet<&str> = matches.values_of("del").unwrap().collect();
            let tags = &hash_set_str_to_hash_set_string(&tags);
            for file in &files {
                tag_manager::del_tags(file, tags, recursive);
            }
        }
    }
    else if matches.is_present("list") || matches.is_present("query") {
        let mut code = String::new();
        if matches.is_present("query") {
            // TODO: improve queries, boolean logic, etc.
            let query : Vec<&str> = matches.values_of("query").unwrap().collect();
            code = String::from(CODE_ENTRIES);
            code.push_str(query[0]);
        }
        if matches.is_present("list") {
            code = String::from(CODE_TAGS);
        }

        let mut stream = UnixStream::connect(SOCKET_ADDRESS).unwrap();
        stream.write_all(code.as_str().as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        print!("{}", response);
    }
    else {
        println!("{}", help);
    }
}

fn show_tags(file: &str, recursive: bool) {
    match tag_manager::get_tags(file) {
        Some(tags) => {
            let mut tags : Vec<String> = tags.into_iter().collect();
            tags.sort();
            println!("Tag(s) {:?} for file \"{}\"", tags, file);
        },
        None => println!("File \"{}\" has no tags", file)
    }
    match fs::metadata(file) {
        Ok(result) => {
            if result.file_type().is_dir() && recursive {
                for entry in fs::read_dir(file).unwrap() {
                    let sub_file = entry.unwrap().path().to_str().unwrap().to_string();
                    show_tags(&sub_file, recursive);
                }
            }
        },
        Err(err) => eprintln!("Error for file \"{}\" : {}", file, err)
    }
}

fn hash_set_str_to_hash_set_string(files: &HashSet<&str>) -> HashSet<String> {
    let mut new_files : HashSet<String> = HashSet::new();
    for f in files { new_files.insert(f.to_string()); }
    new_files
}
