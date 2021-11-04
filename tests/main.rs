use std::process::Command;
use std::str;

#[test]
fn test_help() {
    let output1 = if cfg!(target_os = "windows") {
        Command::new("target\\debug\\freight_chess.exe")
            .arg("-h")
            .output()
            .expect("Failed to execute process")
    } else {
        Command::new("./target/debug/freight_chess")
            .arg("-h")
            .output()
            .expect("Failed to execute process")
    };

    let output2 = if cfg!(target_os = "windows") {
        Command::new("target\\debug\\freight_chess.exe")
            .output()
            .expect("Failed to execute process")
    } else {
        Command::new("./target/debug/freight_chess")
            .output()
            .expect("Failed to execute process")
    };

    let output1 = match str::from_utf8(output1.stdout.as_slice()) {
        Ok(t) => t,
        Err(e) => panic!("Invalid utf-8 sequence: {}", e),
    };

    let output2 = match str::from_utf8(output2.stdout.as_slice()) {
        Ok(t) => t,
        Err(e) => panic!("Invalid utf-8 sequence: {}", e),
    };

    assert_eq!(output1, output2);
}

#[test]
fn test_version() {
    let output = if cfg!(target_os = "windows") {
        Command::new("target\\debug\\freight_chess.exe -h")
            .arg("-V")
            .output()
            .expect("Failed to execute process")
    } else {
        Command::new("./target/debug/freight_chess")
            .arg("-V")
            .output()
            .expect("Failed to execute process")
    };

    let output = match str::from_utf8(output.stdout.as_slice()) {
        Ok(t) => t,
        Err(e) => panic!("Invalid utf-8 sequence: {}", e),
    };

    println!("{}", output);
    assert_eq!(output, "Chess Engine 0.1.0\n");
}
