use std::{
    io::{BufReader,BufRead, Result},
    fs,
};

pub fn read_file_lines_to_vec(filename: &str) -> Vec<String>{
 let file_in = fs::File::open(filename).unwrap(); 
 let file_reader = BufReader::new(file_in); 
 file_reader.lines().filter_map(Result::ok).collect()
}
