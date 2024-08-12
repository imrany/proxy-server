use std::{
    io::{BufReader,BufRead, Result},
    fs,
};

pub fn read_file_lines_to_vec(filename: &str) -> Vec<String>{
 let file_in = fs::File::open(filename).unwrap(); 
 let file_reader = BufReader::new(file_in); 
 file_reader.lines().filter_map(Result::ok).collect()
}

pub fn check_address_block(address_to_check: &str) -> bool {
 let addresses_blocked: Vec<String> = read_file_lines_to_vec(&"./blacklist.txt");
 let address_in = addresses_blocked.contains(&address_to_check.to_string());
 return address_in
}
