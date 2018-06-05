//! # Tag Manager API
//! Here are the public functions for getting, setting and deleting tags on files given.
//! The tags are stored in an extended attribute called "user.tags" and separated by comma.

use std::fs;

extern crate xattr;
extern crate clap;

const ATTR_NAME : &str = "user.tags";
const SEPARATOR : u8 = ',' as u8;

enum Operation { Set, Delete }
use Operation::*;

/// Return the tags (if there is at least one) associated with the file given. Print error 
/// on stderr if there is an error.
pub fn get_tags(file: &str) -> Option<Vec<String>> {
    match check_existent_tags(file) {
        Ok(res) => res,
        Err(_) => None 
    }
}

/// Set given tags to given file. If a tag is already present, he's not added. Preserve existent
/// tags. The recursion in subtree is activated with `recursive` to true.
/// Print to stdout the new tags added to file.
pub fn set_tags(file: &str, new_tags: &Vec<String>, recursive: bool) {
    recursion(file, recursive, Set, new_tags);
    match check_existent_tags(file) {
        Ok(res) => match res {
            Some(mut tags) => {
                let mut full_tags = new_tags.clone();
                full_tags.append(&mut tags);
                // Check duplicata with sort() and dedup()
                full_tags.sort();
                full_tags.dedup();
                xattr::set(file, ATTR_NAME, &vec_string_to_vec_u8(&full_tags))
                    .expect("Error when setting tag(s)");
            },
            None => xattr::set(file, ATTR_NAME, &vec_string_to_vec_u8(new_tags))
                .expect("Error when setting tag(s)")
        },
        Err(err) => {
            eprintln!("Error for file \"{}\" : {}", file, err);
            return;
        }
    }
    println!("Tag(s) {:?} for file {:?} have been setted", new_tags, file);
}

/// Delete given tags of given file. Preserve other existent tags.
/// The recursion in subtree is activated with `recursive` to true.
/// Print to stdout the deleted tags.
pub fn del_tags(file: &str, tags_to_del: &Vec<String>, recursive: bool) {
    recursion(file, recursive, Delete, tags_to_del);
    match check_existent_tags(file) {
        Ok(res) => match res {
            Some(mut tags) => {
                // Delete only the given tags
                for tag in tags_to_del { tags.retain(|ref e| e != &tag); }
                // To avoid to let an empty array of tags
                if tags.is_empty() { match xattr::remove(file, ATTR_NAME) { _ => () } }
                else {
                    xattr::set(file, ATTR_NAME, &vec_string_to_vec_u8(&tags))
                        .expect("Error when (re)setting tag(s)");
                }
            }, _ => ()
        },
        Err(err) => {
            eprintln!("Error for file \"{}\" : {}", file, err);
            return;
        }
    }
    println!("Tag(s) {:?} for file {:?} have been deleted", tags_to_del, file);
}

fn recursion(file: &str, recursive: bool, operation: Operation, tags: &Vec<String>) {
    if fs::metadata(file).unwrap().file_type().is_dir() && recursive {
        for entry in fs::read_dir(file).unwrap() {
            let sub_file = entry.unwrap().path().to_str().unwrap().to_string();
            match operation {
                Set => set_tags(&sub_file, tags, recursive),
                Delete => del_tags(&sub_file, tags, recursive)
            }
        }
    }
}

fn check_existent_tags(file: &str) -> Result<Option<Vec<String>>, std::io::Error> {
    match xattr::get(file, ATTR_NAME) {
        Ok(res) => match res {
            Some(tags) => Ok(Some(vec_u8_to_vec_string(tags))),
            None => Ok(None)
        },
        Err(err) => Err(err)
    }
}

fn vec_string_to_vec_u8(tags_string: &Vec<String>) -> Vec<u8> {
    let mut tags_u8 : Vec<u8> = Vec::new();
    if !tags_string.is_empty() {
        for tag in tags_string {
            for u in tag.bytes() { tags_u8.push(u); }
            tags_u8.push(SEPARATOR);
        }
        // remove last separator
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

// -------------------------------------------- TESTS --------------------------------------------

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
    #[should_panic]
    fn check_existent_tags_no_file() {
        let path = "/tmp/check_existent_tags_no_tags";

        // Test with file inexistent
        let result = super::check_existent_tags(&path);
        panic!(result);
    }

    #[test]
    fn check_existent_tags_no_tags() {
        let path = "/tmp/check_existent_tags_no_tags";
        File::create(path).expect("Error when creating file");

        // Test with file with no tags
        let option = super::check_existent_tags(&path).unwrap();
        assert_eq!(option, None);

        fs::remove_file(path).expect("Error when removing file");
    }

    #[test]
    fn check_existent_tags_tags() {
        let path = "/tmp/check_existent_tags_tags";
        File::create(path).expect("Error when creating file");

        // Test with file with tags
        let vec_u8 = &[65, 67, 68, 67, super::SEPARATOR, 66, 79, 66];
        let tags = vec!["ACDC", "BOB"];
        super::xattr::set(path, super::ATTR_NAME, vec_u8).unwrap();
        let option = super::check_existent_tags(&path).unwrap().unwrap();
        assert_eq!(option, tags);

        fs::remove_file(path).expect("Error when removing file");
    }

    #[test]
    fn set_tags() {
        let path = "/tmp/set_tags";
        File::create(path).expect("Error when creating file");

        let tags : Vec<String> = vec!["bob".to_string(), "max".to_string()];
        let tags_u8 = vec![98, 111, 98, super::SEPARATOR, 109, 97, 120];
        super::set_tags(path, &tags, false);
        if let Ok(res) = super::xattr::get(path, super::ATTR_NAME) {
            if let Some(tags) = res {
                assert_eq!(tags, tags_u8);
            }
        }

        // Reset the same tags to see if dedup work correctly
        super::set_tags(path, &tags, false);
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

        super::set_tags(path, &vec!["bob".to_string(), "max".to_string()], false);

        // Delete "bob"
        super::del_tags(path, &vec!["bob".to_string()], false);
        let tags_u8 = vec![109, 97, 120];
        if let Ok(res) = super::xattr::get(path, super::ATTR_NAME) {
            if let Some(tags) = res {
                assert_eq!(tags, tags_u8);
            }
        }

        fs::remove_file(path).expect("Error when removing file");
    }
}
