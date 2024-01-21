use std::fs;
use std::path::PathBuf;
use std::process::Command;

const TESTS_DIR: &str = "./tests";
const TESTS_TEMPORARY_TARGET_DIR: &str = "./tests/target/";
const TCC_DIR: &str = "./target/debug/tcc";

#[test]
fn test_invalid_programs() {
    // create the tcc executable
    Command::new("cargo")
        .arg("build")
        .args(["--target-dir", TESTS_TEMPORARY_TARGET_DIR])
        .status()
        .expect("could not compile the tcc executable");

    std::env::set_current_dir(TESTS_DIR)
        .expect("could not change directory to the temporary directory");

    let mut test_programs_dir = std::env::current_dir().unwrap();
    test_programs_dir.push("programs_invalid");
    test_programs(test_programs_dir);
}

fn test_programs(dir: PathBuf) {
    let dir_entries = fs::read_dir(dir).unwrap();

    for dir_entry in dir_entries {
        let path = dir_entry.unwrap().path();
        if path.is_dir() {
            test_programs(path);
            continue;
        }

        let input_file_dir = &path.into_os_string().into_string().unwrap();
        println!("Checking failure for the file {:?}", input_file_dir);

        // compile source code with tcc
        let tcc_exit_status = Command::new(TCC_DIR)
            .arg(input_file_dir)
            .arg("-n")
            .status()
            .unwrap_or_else(|_| panic!("tcc could not compile {}", input_file_dir));
        if tcc_exit_status.success() {
            panic!("tcc succeeded for file {:?}", input_file_dir)
        }
    }
}
