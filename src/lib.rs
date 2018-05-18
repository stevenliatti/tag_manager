extern crate xattr;
extern crate clap;

const ATTR_NAME : &str = "user.tags";
const SEPARATOR : u8 = ',' as u8;

enum Operation { Get, Set, Delete }
use Operation::*;

pub fn get_tags(files: &Vec<&str>) {
    let mut empty : Vec<u8> = Vec::new();
    for &file in files { check_existent_tags(file, &Get, &mut empty); }
}

pub fn set_tags(files: &Vec<&str>, tags: clap::Values) {
    let tags: Vec<&str> = tags.collect();
    let mut new_tags_u8 : Vec<u8> = Vec::new();
    // Convert str tags to bytes
    for tag in &tags {
        for u in tag.bytes() { new_tags_u8.push(u); }
        new_tags_u8.push(SEPARATOR);
    }

    for &file in files {
        let mut existent_tags = new_tags_u8.clone();
        check_existent_tags(file, &Set, &mut existent_tags);
        let mut full_tags_str = vec_u8_to_vec_string(existent_tags);
        // Compute a Vec<String> of tags to check duplicata with dedup()
        full_tags_str.sort();
        full_tags_str.dedup();
        xattr::set(file, ATTR_NAME, &vec_string_to_vec_u8(&full_tags_str))
            .expect("Error when setting tag(s)");
    }
    println!("Tag(s) {:?} for file(s) {:?} have been setted", tags, files);
}

pub fn del_tags(files: &Vec<&str>, tags: clap::Values) {
    let tags_to_del: Vec<&str> = tags.collect();
    for &file in files {
        let mut existent_tags = Vec::new();
        check_existent_tags(file, &Delete, &mut existent_tags);
        let mut existent_tags_str = vec_u8_to_vec_string(existent_tags);
        // Retain tags that must be effectively deleted
        for tag in &tags_to_del { existent_tags_str.retain(|ref e| e != &&tag.to_string()); }
        // Delete the tags
        xattr::remove(file, ATTR_NAME).expect("Error when deleting tag(s)");
        // Adding again the tags that are not in tags_to_del
        let final_tags = vec_string_to_vec_u8(&existent_tags_str);
        xattr::set(file, ATTR_NAME, &final_tags).expect("Error when (re)setting tag(s)");
    }
    println!("Tag(s) {:?} for file(s) {:?} have been deleted", tags_to_del, files);
}

fn check_existent_tags(file: &str, operation: &Operation, existent_tags: &mut Vec<u8>) {
    match xattr::get(file, ATTR_NAME) {
        Ok(res) => match res {
            Some(tags) => {
                match operation {
                    &Get => println!("Tags {:?} for file \"{}\"", vec_u8_to_vec_string(tags), file),
                    _ => for tag in tags { existent_tags.push(tag); }
                }
            },
            None => {
                match operation {
                    &Set => { existent_tags.pop(); },
                    _ => { println!("File \"{}\" has no tags", file) }
                }
            }
        },
        Err(err) => { 
            println!("Error for file \"{}\" : {}", file, err);
            return;
        }
    }
}

fn vec_string_to_vec_u8(tags_string: &Vec<String>) -> Vec<u8> {
    let mut tags_u8 : Vec<u8> = Vec::new();
    for tag in tags_string {
        for u in tag.bytes() { tags_u8.push(u); }
        tags_u8.push(SEPARATOR);
    }
    // remove last comma
    tags_u8.pop();
    tags_u8
}

fn vec_u8_to_vec_string(tags_u8: Vec<u8>) -> Vec<String> {
    let mut s = String::new();
    let mut tags_str : Vec<String> = Vec::new();
    for u in tags_u8 {
        if u == SEPARATOR {
            tags_str.push(s.to_string());
            s = String::new();
        }
        else { s.push(u as char); }
    }
    tags_str.push(s.to_string());
    tags_str
}
