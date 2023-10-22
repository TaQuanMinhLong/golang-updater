use std::process;

pub fn get_output(cmd: &str) -> process::Output {
    process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect(&format!("Failed to execute command \"{}\"", cmd))
}

pub fn get_stdout(cmd: &str) -> String {
    let stdout = get_output(cmd).stdout;
    String::from_utf8(stdout).expect("Failed to parse stdout")
}

pub fn exec_cmd(cmd: &str) {
    process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect(&format!("Failed to execute command \"{}\"", cmd));
}
