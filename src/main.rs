//! # Tag Manager
//! Little CLI tool for getting, setting and deleting tags for files and folders.
//! The tags are stored in an extended attribute called "user.tags" and separated by comma.
//! Run `tag_manager -h` to see help.

extern crate tag_manager;
extern crate clap;
use clap::{App, Arg, ArgGroup};

fn main() {
    let matches = App::new("tag_manager")
        .help("\
            tag_manager v0.1.0 - Manage your tags\n\n\
            Usage:\n    tag_manager [Options] <files> [[--set|--del] <tags>]\n\n\
            Options:\n    \
            -h    Display this message\n    \
            -r    Recursive option. Get, set or delete tags for each folder and file in folder subtree\n\n\
            Arguments:\n    \
            -s, --set <tags>       Set the given tags\n    \
            -d, --del <tags>       Delete the given tags\n\n\
            Examples:\n    \
            tag_manager myfile => Show the actual tags of file \"myfile\"\n    \
            tag_manager myfile -s work => Set the tag \"work\" to the file \"myfile\"\n    \
            tag_manager myfile -d work => Delete the tag \"work\" to the file \"myfile\"\n    \
            tag_manager myfolder -r -s geneva => Set the tag \"geneva\" to the folder \"myfolder\" and his subtree")
        .group(ArgGroup::with_name("ops").args(&["set", "del"]))
        .arg(Arg::with_name("set").short("s").long("set").takes_value(true).multiple(true))
        .arg(Arg::with_name("del").short("d").long("del").takes_value(true).multiple(true))
        .arg(Arg::with_name("files").takes_value(true).multiple(true).required(true))
        .arg(Arg::with_name("recursive").short("-r").long("--recursive"))
        .get_matches();

    let files: Vec<&str> = matches.values_of("files").unwrap().collect();
    let recursive = matches.is_present("recursive");
    
    if !matches.is_present("set") && !matches.is_present("del") {
        tag_manager::get_tags(&vec_str_to_vec_string(&files), recursive);
    }

    if let Some(tags) = matches.values_of("set") {
        tag_manager::set_tags(&vec_str_to_vec_string(&files), &tags.collect(), recursive);
    }

    if let Some(tags) = matches.values_of("del") {
        tag_manager::del_tags(&vec_str_to_vec_string(&files), &tags.collect(), recursive);
    }
}

fn vec_str_to_vec_string(files: &Vec<&str>) -> Vec<String> {
    let mut new_files : Vec<String> = Vec::new();
    for f in files {
        new_files.push(f.to_string());
    }
    new_files
}
