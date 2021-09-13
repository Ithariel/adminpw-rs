use std::io::{BufRead, BufReader};
use std::fs::File;

use async_std::task;

use sqlx::Connection;
use sqlx::any::AnyConnection;
use sqlx::any::AnyQueryResult;

const CONFIG: &str = "/etc/liveconfig/liveconfig.conf";

pub fn is_liveconfig() -> bool {
    if std::path::Path::new(CONFIG).exists() {
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
    println!("Admin account: {}\nPassword hash: {}\n", &username, &password_hash);
    set_login(&password);
    println!("Temporary password: {}\n", &password);
    super::wait();
    task::block_on(async {
        reset_login(&username, &password_hash).await.unwrap();
    });
    println!("Passwort has been reset!");
}

async fn get_connection() -> Result<AnyConnection, sqlx::Error> {
    let mut driver = String::new();
    let mut username = String::new();
    let mut password = String::new();
    let mut hostname = String::new();
    let mut database = String::new();

    let lines = BufReader::new(File::open(CONFIG)?).lines();
    for line in lines {
        if let Ok(line) = line {
            let split = line.split('=');
            let vec: Vec<&str> = split.collect();
            if vec.len() > 1 {
                if vec[0].contains("db_driver") {
                    driver = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                } else if vec[0].contains("db_host") {
                    hostname = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                } else if vec[0].contains("db_name") {
                    database = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                } else if vec[0].contains("db_user") {
                    username = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                } else if vec[0].contains("db_password") {
                    password = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                }
            }
        }
    }

    if driver == "sqlite" {
        return Ok(AnyConnection::connect(&format!("sqlite://{}", database)).await?)
    }

    Ok(AnyConnection::connect(&format!("mysql://{}:{}@{}/{}", username, password, hostname, database)).await?)
}

async fn get_login() -> Result<(String, String), sqlx::Error> {
    let mut conn = get_connection().await?;
    let row: (String, String) = sqlx::query_as("SELECT U_LOGIN, U_PASSWORD FROM USERS")
        .fetch_one(&mut conn).await?;
    conn.close().await?;
    Ok(row)
}

fn set_login(password: &str) {
    std::process::Command::new("/usr/sbin/liveconfig")
                    .args(["--init"])
                    .env("LCINITPW", &password)
                    .status()
                    .unwrap();
}

async fn reset_login(username: &str, password_hash: &str) -> Result<AnyQueryResult, sqlx::Error> {
    let mut conn = get_connection().await?;
    let result = sqlx::query("UPDATE USERS SET U_PASSWORD = ? WHERE U_LOGIN = ?")
        .bind(password_hash)
        .bind(username)
        .execute(&mut conn)
        .await;
    conn.close().await?;
    result
}