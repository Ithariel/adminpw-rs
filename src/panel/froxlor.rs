use std::io::{Error, BufRead, BufReader};

use std::fs::File;
use std::path::Path;

use async_std::task;

use sqlx::Connection;
use sqlx::any::AnyConnection;
use sqlx::any::AnyQueryResult;

const FROXLOR_DIR: &str = "/var/www/froxlor";
const SYSCP_DIR: &str = "/var/www/syscp";
const USERDATA: &str = "/lib/userdata.inc.php";

pub fn is_froxlor() -> bool {
    if Path::new(FROXLOR_DIR).exists() {
        return true;
    } else if Path::new(SYSCP_DIR).exists() {
        return true;
    }
    false
}

pub fn run(password: &str) {
    let mut username = String::new();
    let mut password_hash = String::new();
    task::block_on(async {
        let (u, h) = get_login().await.unwrap();
        username = u;
        password_hash = h;
    });
    println!("Admin account:{:<50}\nPassword hash: {:<50}\n", &username, &password_hash);
    task::block_on(async {
        set_login(&username, &password).await.unwrap();
    });
    println!("Temporary password: {:<50}\n", &password);
    super::wait();
    task::block_on(async {
        reset_login(&username, &password_hash).await.unwrap();
    });
    println!("Passwort has been reset!");
}

fn get_mysql_credentials() -> Result<(String, String, String, String), Error> {
    let path;
    let mut username = String::new();
    let mut password = String::new();
    let mut hostname = String::new();
    let mut database = String::new();

    if Path::new(&[FROXLOR_DIR,USERDATA].join("")).is_file() {
        path = [FROXLOR_DIR, USERDATA].join("");        
    } else if Path::new(&[SYSCP_DIR,USERDATA].join("")).is_file() {
        path = [SYSCP_DIR, USERDATA].join("");
    } else {
        return Err(Error::new(std::io::ErrorKind::NotFound, "userdata.inc.php not found"));
    }

    let lines = BufReader::new(File::open(path)?).lines();
    for line in lines {
        if let Ok(line) = line {
            let split = line.split('=');
            let vec: Vec<&str> = split.collect();
            if vec.len() > 1 {
                if vec[0].contains("root") && vec[0].contains("user") {
                    username = String::from(vec[1].replace(&['\'', ';'][..], ""));
                } else if vec[0].contains("root") && vec[0].contains("password") {
                    password = String::from(vec[1].replace(&['\'', ';'][..], ""));
                } else if vec[0].contains("root") && vec[0].contains("host") {
                    hostname = String::from(vec[1].replace(&['\'', ';'][..], ""));
                } else if vec[0].contains("sql") && vec[0].contains("db") {
                    database = String::from(vec[1].replace(&['\'', ';'][..], ""));
                }
            }
        }
    }

    Ok((username, password, hostname, database))
}

async fn get_login() -> Result<(String, String), sqlx::Error> {
    let (username, password, hostname, database) = get_mysql_credentials()?;
    let mut conn = AnyConnection::
        connect(&format!("mysql://{}:{}@{}/{}", username, password, hostname, database)).await?;
    let row: (String, String) = sqlx::query_as("SELECT loginname, password FROM panel_admins")
        .fetch_one(&mut conn).await?;
    Ok(row)
}

async fn set_login(username: &str, password: &str) -> Result<AnyQueryResult, sqlx::Error> {
    let (db_username, db_password, hostname, database) = get_mysql_credentials()?;
    let mut conn = AnyConnection::
        connect(&format!("mysql://{}:{}@{}/{}", db_username, db_password, hostname, database)).await?;
    sqlx::query("UPDATE panel_admins SET password = MD5(?) WHERE loginname = ?")
        .bind(password)
        .bind(username)
        .execute(&mut conn)
        .await
}

async fn reset_login(username: &str, password_hash: &str) -> Result<AnyQueryResult, sqlx::Error> {
    let (db_username, db_password, hostname, database) = get_mysql_credentials()?;
    let mut conn = AnyConnection::
        connect(&format!("mysql://{}:{}@{}/{}", db_username, db_password, hostname, database)).await?;
    sqlx::query("UPDATE panel_admins SET password = ? WHERE loginname = ?")
        .bind(password_hash)
        .bind(username)
        .execute(&mut conn)
        .await
}