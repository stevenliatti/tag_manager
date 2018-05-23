//! # Tag Manager API
//! Here are the public functions for getting, setting and deleting tags on files given.
//! The tags are stored in an extended attribute called "user.tags" and separated by comma.

use std::fs;

extern crate xattr;
extern crate clap;

const ATTR_NAME : &str = "user.tags";
const SEPARATOR : u8 = ',' as u8;

enum Operation { Get, Set, Delete }
use Operation::*;

/// Print to stdout the tags (if there is at least one) associated with the files given.
/// It print one line by file. The recursion in subtree is activated with `recursive` to true.
/// The function print an error if the file is not found.
pub fn get_tags(files: &Vec<String>, recursive: bool) {
    let mut empty : Vec<u8> = Vec::new();
    for file in files {
        recursion(file, recursive, Get, &vec![]);
        match check_existent_tags(&file, &Get, &mut empty) { _ => continue }
    }
}

/// Set given tags to given files. If a tag is already present, he's not added. Preserve existent
/// tags. The recursion in subtree is activated with `recursive` to true.
/// Print to stdout the new tags added to files.
pub fn set_tags(files: &Vec<String>, tags: &Vec<&str>, recursive: bool) {
    let mut new_tags_u8 : Vec<u8> = Vec::new();
    // Convert str tags to bytes
    for tag in tags {
        for u in tag.bytes() { new_tags_u8.push(u); }
        new_tags_u8.push(SEPARATOR);
    }
    for file in files {
        recursion(file, recursive, Set, tags);
        let mut existent_tags = new_tags_u8.clone();
        // If needed, add existent tags to existent_tags in check_existent_tags()
        match check_existent_tags(&file, &Set, &mut existent_tags) {
            Ok(_) => (),
            Err(_) => continue
        }
        let mut full_tags_str = vec_u8_to_vec_string(existent_tags);
        // Compute a Vec<String> of tags to check duplicata with dedup()
        full_tags_str.sort();
        full_tags_str.dedup();
        xattr::set(file, ATTR_NAME, &vec_string_to_vec_u8(&full_tags_str))
            .expect("Error when setting tag(s)");
    }
    println!("Tag(s) {:?} for file(s) {:?} have been setted", tags, files);
}

/// Delete given tags of given files. Preserve other existent tags.
/// The recursion in subtree is activated with `recursive` to true.
/// Print to stdout the deleted tags.
pub fn del_tags(files: &Vec<String>, tags_to_del: &Vec<&str>, recursive: bool) {
    for file in files {
        recursion(file, recursive, Delete, tags_to_del);
        let mut existent_tags = Vec::new();
        // If needed, add existent tags to existent_tags in check_existent_tags()
        match check_existent_tags(&file, &Delete, &mut existent_tags) {
            Ok(_) => (),
            Err(_) => continue
        }
        let mut existent_tags_str = vec_u8_to_vec_string(existent_tags);
        // Retain tags that must be effectively deleted
        for tag in tags_to_del { existent_tags_str.retain(|ref e| e != &&tag.to_string()); }
        if existent_tags_str.is_empty() { match xattr::remove(file, ATTR_NAME) { _ => () } }
        else {
            // Adding again the tags that are not in tags_to_del
            xattr::set(file, ATTR_NAME, &vec_string_to_vec_u8(&existent_tags_str))
                .expect("Error when (re)setting tag(s)");
        }
    }
    println!("Tag(s) {:?} for file(s) {:?} have been deleted", tags_to_del, files);
}

fn recursion(file: &String, recursive: bool, operation: Operation, tags: &Vec<&str>) {
    if fs::metadata(file).unwrap().file_type().is_dir() && recursive {
        let mut sub_files : Vec<String> = Vec::new();
        for entry in fs::read_dir(&file).unwrap() {
            let path = entry.unwrap().path();
            sub_files.push(path.to_str().unwrap().to_string());
        }
        match operation {
            Get => get_tags(&sub_files, recursive),
            Set => set_tags(&sub_files, tags, recursive),
            Delete => del_tags(&sub_files, tags, recursive)
        }
    }
}

fn check_existent_tags(file: &str, operation: &Operation, existent_tags: &mut Vec<u8>)
-> Result<(), std::io::Error> {
    match xattr::get(file, ATTR_NAME) {
        Ok(res) => match res {
            Some(tags) => {
                match operation {
                    &Get => {
                        println!("Tag(s) {:?} for file \"{}\"", vec_u8_to_vec_string(tags), file);
                        Ok(())
                    },
                    _ => {
                        for tag in tags { existent_tags.push(tag); }
                        Ok(())
                    }
                }
            },
            None => {
                match operation {
                    &Set => { existent_tags.pop(); Ok(()) },
                    _ => { println!("File \"{}\" has no tags", file); Ok(()) }
                }
            }
        },
        Err(err) => {
            println!("Error for file \"{}\" : {}", file, err);
            Err(err)
        }
    }
}

fn vec_string_to_vec_u8(tags_string: &Vec<String>) -> Vec<u8> {
    let mut tags_u8 : Vec<u8> = Vec::new();
    if !tags_string.is_empty() {
        for tag in tags_string {
            for u in tag.bytes() { tags_u8.push(u); }
            tags_u8.push(SEPARATOR);
        }
        // remove last comma
        tags_u8.pop();
    }
    tags_u8
}

fn vec_u8_to_vec_string(tags_u8: Vec<u8>) -> Vec<String> {
    let mut s = String::new();
    let mut tags_str : Vec<String> = Vec::new();
    if !tags_u8.is_empty() {
        for u in tags_u8 {
            if u == SEPARATOR {
                tags_str.push(s.to_string());
                s = String::new();
            }
            else { s.push(u as char); }
        }
        tags_str.push(s.to_string());
    }
    tags_str
}

// ---------------------------------------- TESTS ----------------------------------------

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::fs;

    #[test]
    fn vec_u8_to_vec_string_empty() {
        let empty_u8 : Vec<u8> = Vec::new();
        let empty_string : Vec<String> = Vec::new();
        assert_eq!(empty_string, super::vec_u8_to_vec_string(empty_u8));
    }

    #[test]
    fn vec_u8_to_vec_string_one() {
        // ["ACDC"]
        let vec_u8 : Vec<u8> = vec![65, 67, 68, 67];
        let vec_string : Vec<String> = vec!["ACDC".to_string()];
        assert_eq!(vec_string, super::vec_u8_to_vec_string(vec_u8));
    }

    #[test]
    fn vec_u8_to_vec_string_two() {
        // ["ACDC", "BOB"]
        let vec_u8 : Vec<u8> = vec![65, 67, 68, 67, super::SEPARATOR, 66, 79, 66];
        let vec_string : Vec<String> = vec!["ACDC".to_string(), "BOB".to_string()];
        assert_eq!(vec_string, super::vec_u8_to_vec_string(vec_u8));
    }

    #[test]
    fn vec_string_to_vec_u8_empty() {
        let empty_u8 : Vec<u8> = Vec::new();
        let empty_string : Vec<String> = Vec::new();
        assert_eq!(empty_u8, super::vec_string_to_vec_u8(&empty_string));
    }

    #[test]
    fn vec_string_to_vec_u8_one() {
        // ["ACDC"]
        let vec_u8 : Vec<u8> = vec![65, 67, 68, 67];
        let vec_string : Vec<String> = vec!["ACDC".to_string()];
        assert_eq!(vec_u8, super::vec_string_to_vec_u8(&vec_string));
    }

    #[test]
    fn vec_string_to_vec_u8_two() {
        // ["ACDC", "BOB"]
        let vec_u8 : Vec<u8> = vec![65, 67, 68, 67, super::SEPARATOR, 66, 79, 66];
        let vec_string : Vec<String> = vec!["ACDC".to_string(), "BOB".to_string()];
        assert_eq!(vec_u8, super::vec_string_to_vec_u8(&vec_string));
    }

    #[test]
    fn check_existent_tags_no_tags_get() {
        let path = "/tmp/check_existent_tags_no_tags_get";
        File::create(path).expect("Error when creating file");

        // Test with file with no tags and get
        let mut vec_u8 : Vec<u8> = vec![65, 67, 68, 67, super::SEPARATOR, 66, 79, 66];
        let clone_vec_u8 = vec_u8.clone();
        super::check_existent_tags(&path, &super::Operation::Get, &mut vec_u8).unwrap();
        assert_eq!(vec_u8, clone_vec_u8);

        fs::remove_file(path).expect("Error when removing file");
    }

    #[test]
    fn check_existent_tags_no_tags_set() {
        let path = "/tmp/check_existent_tags_no_tags_set";
        File::create(path).expect("Error when creating file");

        // Test with file with no tags and set
        let mut vec_u8 : Vec<u8> = vec![65, 67, 68, 67, super::SEPARATOR, 66, 79, 66, super::SEPARATOR];
        let mut clone_vec_u8 = vec_u8.clone();
        assert_eq!(vec_u8, clone_vec_u8);
        clone_vec_u8.pop();
        super::check_existent_tags(&path, &super::Operation::Set, &mut vec_u8).unwrap();
        assert_eq!(vec_u8, clone_vec_u8);

        fs::remove_file(path).expect("Error when removing file");
    }

    #[test]
    fn check_existent_tags_tags_set() {
        let path = "/tmp/check_existent_tags_tags_set";
        File::create(path).expect("Error when creating file");

        // Test with file with tag and set
        let bob = &[66, 79, 66];
        super::xattr::set(path, super::ATTR_NAME, bob).unwrap();
        let mut vec_u8 : Vec<u8> = vec![65, 67, 68, 67, super::SEPARATOR];
        let mut clone_vec_u8 = vec_u8.clone();
        assert_eq!(vec_u8, clone_vec_u8);
        super::check_existent_tags(&path, &super::Operation::Set, &mut vec_u8).unwrap();
        clone_vec_u8.push(bob[0]);
        clone_vec_u8.push(bob[1]);
        clone_vec_u8.push(bob[2]);
        assert_eq!(vec_u8, clone_vec_u8);

        fs::remove_file(path).expect("Error when removing file");
    }

    #[test]
    fn set_tags() {
        let path = "/tmp/set_tags";
        File::create(path).expect("Error when creating file");

        let tags = vec!["bob", "max"];
        let tags_u8 = vec![98, 111, 98, super::SEPARATOR, 109, 97, 120];
        super::set_tags(&vec![path.to_string()], &tags, false);
        if let Ok(res) = super::xattr::get(path, super::ATTR_NAME) {
            if let Some(tags) = res {
                assert_eq!(tags, tags_u8);
            }
        }

        // Reset the same tags to see if dedup work correctly
        super::set_tags(&vec![path.to_string()], &tags, false);
        if let Ok(res) = super::xattr::get(path, super::ATTR_NAME) {
            if let Some(tags) = res {
                assert_eq!(tags, tags_u8);
            }
        }

        fs::remove_file(path).expect("Error when removing file");
    }

    #[test]
    fn del_tags() {
        let path = "/tmp/del_tags";
        File::create(path).expect("Error when creating file");
        let files : Vec<String> = vec![path.to_string()];

        super::set_tags(&files, &vec!["bob", "max"], false);

        // Delete "bob"
        super::del_tags(&files, &vec!["bob"], false);
        let tags_u8 = vec![109, 97, 120];
        if let Ok(res) = super::xattr::get(path, super::ATTR_NAME) {
            if let Some(tags) = res {
                assert_eq!(tags, tags_u8);
            }
        }

        fs::remove_file(path).expect("Error when removing file");
    }
}
