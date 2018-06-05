//! # Tag Manager
//! Little CLI tool for getting, setting and deleting tags for files and folders.
//! The tags are stored in an extended attribute called "user.tags" and separated by comma.
//! Run `tag_manager -h` to see help.

extern crate tag_manager;
extern crate clap;
use clap::{App, Arg, ArgGroup};
use std::fs;

fn main() {
    let matches = App::new("tag_manager")
        .help("\
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
            tag_manager myfolder -r -s geneva   => Set the tag \"geneva\" to the folder \"myfolder\" and his subtree")
        .group(ArgGroup::with_name("ops").args(&["set", "del"]))
        .arg(Arg::with_name("set").short("s").long("set").takes_value(true).multiple(true))
        .arg(Arg::with_name("del").short("d").long("del").takes_value(true).multiple(true))
        .arg(Arg::with_name("files").takes_value(true).multiple(true).required(true))
        .arg(Arg::with_name("recursive").short("-r").long("--recursive"))
        .get_matches();

    let files: Vec<&str> = matches.values_of("files").unwrap().collect();
    let recursive = matches.is_present("recursive");

    if !matches.is_present("set") && !matches.is_present("del") {
        for file in &files { show_tags(file, recursive); }
    }

    if let Some(tags) = matches.values_of("set") {
        let tags : Vec<&str> = tags.collect();
        let tags = &vec_str_to_vec_string(&tags);
        for file in &files {
            tag_manager::set_tags(file, tags, recursive);
        }
    }

    if let Some(tags) = matches.values_of("del") {
        let tags : Vec<&str> = tags.collect();
        let tags = &vec_str_to_vec_string(&tags);
        for file in &files {
            tag_manager::del_tags(file, tags, recursive);
        }
    }
}

fn show_tags(file: &str, recursive: bool) {
    match tag_manager::get_tags(file) {
        Some(tags) => println!("Tag(s) {:?} for file \"{}\"", tags, file),
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

fn vec_str_to_vec_string(files: &Vec<&str>) -> Vec<String> {
    let mut new_files : Vec<String> = Vec::new();
    for f in files { new_files.push(f.to_string()); }
    new_files
}
