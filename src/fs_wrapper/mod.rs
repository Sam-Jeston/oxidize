use std::fs::File;

/// Return a Result<file buffer> or Error(String) to the request handler based on the existence of
/// the file
pub fn file_match (path: String) -> Result<File, String> {
    match File::open(path) {
        Ok(mut file) => {
            Result::Ok(file)
        }
        _ => {
            println!("File does not exist");
            Result::Err("File does not exist".to_string())
        }
    }
}
