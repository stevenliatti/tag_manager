//! # Tag Manager API
//! Here are the public functions for getting, setting and deleting tags on files given.
//! The tags are stored in an extended attribute called "user.tags" and separated by comma.

use std::fs;
use std::collections::HashSet;

extern crate xattr;
extern crate clap;

const ATTR_NAME : &str = "user.tags";
const SEPARATOR : u8 = ',' as u8;

enum Operation { Set, Delete }
use Operation::*;

/// Return the tags (if there is at least one) associated with the file given. Print error
/// on stderr if there is an error.
pub fn get_tags(file: &str) -> Option<HashSet<String>> {
    match check_existent_tags(file) {
        Ok(res) => res,
        Err(_) => None
    }
}

/// Set given tags to given file. If a tag is already present, he's not added. Preserve existent
/// tags. The recursion in subtree is activated with `recursive` to true.
/// Print to stdout the new tags added to file.
pub fn set_tags(file: &str, new_tags: &HashSet<String>, recursive: bool) {
    recursion(file, recursive, Set, new_tags);
    match check_existent_tags(file) {
        Ok(res) => match res {
            Some(mut tags) => {
                for tag in new_tags { tags.insert(tag.clone()); }
                xattr::set(file, ATTR_NAME, &hash_set_to_vec_u8(&tags))
                    .expect("Error when setting tag(s)");
            },
            None => xattr::set(file, ATTR_NAME, &hash_set_to_vec_u8(new_tags))
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
pub fn del_tags(file: &str, tags_to_del: &HashSet<String>, recursive: bool) {
    recursion(file, recursive, Delete, tags_to_del);
    match check_existent_tags(file) {
        Ok(res) => match res {
            Some(mut tags) => {
                // Delete only the given tags
                for tag in tags_to_del {
                    tags.retain(|ref e| e != &tag);
                }
                // To avoid to let an empty array of tags
                if tags.is_empty() {
                    match xattr::remove(file, ATTR_NAME) { _ => () }
                }
                else {
                    xattr::set(file, ATTR_NAME,
                        &hash_set_to_vec_u8(&tags))
                        .expect("Error when (re)setting tag(s)");
                }
            }, _ => ()
        },
        Err(err) => {
            eprintln!("Error for file \"{}\" : {}", file, err);
            return;
        }
    }
    println!("Tag(s) {:?} for file {:?} have been deleted",
        tags_to_del, file);
}
// TODO: doc
pub fn rename_tag(file: &str, old : String, new : String) {
    match check_existent_tags(file) {
        Ok(res) => match res {
            Some(mut tags) => {
                if tags.remove(&old) {
                    tags.insert(new.clone());
                    xattr::set(file, ATTR_NAME, &hash_set_to_vec_u8(&tags))
                        .expect("Error when setting tag(s)");
                }
            },
            None => ()
        },
        Err(err) => {
            eprintln!("Error for file \"{}\" : {}", file, err);
            return;
        }
    }
}

fn recursion(file: &str, recursive: bool, operation: Operation, tags: &HashSet<String>) {
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

fn check_existent_tags(file: &str) -> Result<Option<HashSet<String>>, std::io::Error> {
    match xattr::get(file, ATTR_NAME) {
        Ok(res) => match res {
            Some(tags) => Ok(Some(vec_u8_to_hash_set(tags))),
            None => Ok(None)
        },
        Err(err) => Err(err)
    }
}

fn hash_set_to_vec_u8(tags_set: &HashSet<String>) -> Vec<u8> {
    let mut tags_u8 : Vec<u8> = Vec::new();
    if !tags_set.is_empty() {
        for tag in tags_set {
            for u in tag.bytes() { tags_u8.push(u); }
            tags_u8.push(SEPARATOR);
        }
        // remove last separator
        tags_u8.pop();
    }
    tags_u8
}

fn vec_u8_to_hash_set(tags_u8: Vec<u8>) -> HashSet<String> {
    let mut s = String::new();
    let mut tags_set = HashSet::new();
    if !tags_u8.is_empty() {
        for u in tags_u8 {
            if u == SEPARATOR {
                tags_set.insert(s.to_string());
                s = String::new();
            }
            else { s.push(u as char); }
        }
        tags_set.insert(s.to_string());
    }
    tags_set
}

// -------------------------------------------- TESTS --------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::fs;
    use std::collections::HashSet;

    #[test]
    fn vec_u8_to_set_string_empty() {
        let empty_u8 : Vec<u8> = Vec::new();
        let empty_set : HashSet<String> = HashSet::new();
        assert_eq!(empty_set, super::vec_u8_to_hash_set(empty_u8));
    }

    #[test]
    fn vec_u8_to_set_string_one() {
        // ["ACDC"]
        let vec_u8 : Vec<u8> = vec![65, 67, 68, 67];
        let mut set_string = HashSet::new();
        set_string.insert("ACDC".to_string());
        assert_eq!(set_string, super::vec_u8_to_hash_set(vec_u8));
    }

    #[test]
    fn vec_u8_to_set_string_two() {
        // ["ACDC", "BOB"]
        let vec_u8 : Vec<u8> = vec![65, 67, 68, 67, super::SEPARATOR, 66, 79, 66];
        let mut set_string = HashSet::new();
        set_string.insert("ACDC".to_string());
        set_string.insert("BOB".to_string());
        assert_eq!(set_string, super::vec_u8_to_hash_set(vec_u8));
    }

    #[test]
    fn set_string_to_vec_u8_empty() {
        let empty_u8 : Vec<u8> = Vec::new();
        let empty_set : HashSet<String> = HashSet::new();
        assert_eq!(empty_u8, super::hash_set_to_vec_u8(&empty_set));
    }

    #[test]
    fn set_string_to_vec_u8_one() {
        // ["ACDC"]
        let vec_u8 : Vec<u8> = vec![65, 67, 68, 67];
        let mut set_string = HashSet::new();
        set_string.insert("ACDC".to_string());
        assert_eq!(vec_u8, super::hash_set_to_vec_u8(&set_string));
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
        let mut tags = HashSet::new();
        tags.insert("ACDC".to_string());
        tags.insert("BOB".to_string());
        super::xattr::set(path, super::ATTR_NAME, vec_u8).unwrap();
        let option = super::check_existent_tags(&path).unwrap().unwrap();
        assert_eq!(option, tags);

        fs::remove_file(path).expect("Error when removing file");
    }

    #[test]
    fn set_tag() {
        let path = "/tmp/set_tags";
        File::create(path).expect("Error when creating file");

        let mut tags = HashSet::new();
        tags.insert("bob".to_string());
        let tags_u8 = vec![98, 111, 98];
        super::set_tags(path, &tags, false);
        if let Ok(res) = super::xattr::get(path, super::ATTR_NAME) {
            if let Some(tags) = res {
                assert_eq!(tags, tags_u8);
            }
        }

        // Reset the same tag
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

        let mut tags = HashSet::new();
        tags.insert("bob".to_string());
        tags.insert("max".to_string());;
        super::set_tags(path, &tags, false);

        // Delete "bob"
        let mut bob = HashSet::new();
        bob.insert("bob".to_string());
        super::del_tags(path, &bob, false);
        let tags_u8 = vec![109, 97, 120];
        if let Ok(res) = super::xattr::get(path, super::ATTR_NAME) {
            if let Some(tags) = res {
                assert_eq!(tags, tags_u8);
            }
        }

        fs::remove_file(path).expect("Error when removing file");
    }
}
