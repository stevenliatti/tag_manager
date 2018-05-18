extern crate tag_manager;
extern crate clap;
use clap::{App, Arg, ArgGroup};

fn main() {
    // TODO: add recursive option
    let matches = App::new("tag_manager")
        .help("\
            tag_manager v0.1.0 - Manage your tags\n\n\
            Usage:\n    tag_manager [Options] <files> [[--set|--del] <tags>]\n\n\
            Options:\n    -h    Display this message\n\n\
            Arguments:\n    \
            -s, --set <tags>       Set the given tags\n    \
            -d, --del <tags>       Delete the given tags\n\n\
            Examples:\n    \
            tag_manager myfile => Show the actual tags of file \"myfile\"\n    \
            tag_manager myfile -s work => Set the tag \"work\" to the file \"myfile\"\n    \
            tag_manager myfile -d work => Delete the tag \"work\" to the file \"myfile\"")
        .group(ArgGroup::with_name("ops").args(&["set", "del"]))
        .arg(Arg::with_name("set").help("Set tag(s)").short("s").long("set")
            .takes_value(true).multiple(true))
        .arg(Arg::with_name("del").help("Delete tag(s)").short("d").long("del")
            .takes_value(true).multiple(true))
        .arg(Arg::with_name("files").help("File(s) name(s)")
            .takes_value(true).multiple(true).required(true))
        .get_matches();

    let files: Vec<&str> = matches.values_of("files").unwrap().collect();
    if !matches.is_present("set") && !matches.is_present("del") {
        tag_manager::get_tags(&files);
    }
    if let Some(tags) = matches.values_of("set") { tag_manager::set_tags(&files, tags); }
    if let Some(tags) = matches.values_of("del") { tag_manager::del_tags(&files, tags); }
}
