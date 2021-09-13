pub fn is_plesk() -> bool {
    if std::path::Path::new("/opt/psa").exists() {
        return true;
    }
    false
}

pub fn run() {
    let output = std::process::Command::new("/opt/psa/bin/admin")
                    .args(["--show-password"])
                    .output()
                    .unwrap();
    println!("{}", std::str::from_utf8(&output.stdout).unwrap());
}