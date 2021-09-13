use std::io::{Error, BufRead, BufReader};
use std::fs::File;
use std::path::Path;

use async_std::task;

use sqlx::Connection;
use sqlx::any::AnyConnection;
use sqlx::any::AnyQueryResult;

const CONFIG: &str = "/root/confixx/confixx_main.conf";

pub fn is_confixx() -> bool {
    if Path::new(CONFIG).exists() {
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
    task::block_on(async {
        set_login(&username, &password).await.unwrap();
    });
    println!("Temporary password: {}\n", &password);
    super::wait();
    task::block_on(async {
        reset_login(&username, &password_hash).await.unwrap();
    });
    println!("Passwort has been reset!");
}

fn get_mysql_credentials() -> Result<(String, String, String, String), Error> {
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
                if vec[0].contains("dbUser") {
                    username = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                } else if vec[0].contains("dbPw") {
                    password = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                } else if vec[0].contains("dbServer") {
                    hostname = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
                } else if vec[0].contains("dbDB") {
                    database = String::from(vec[1].replace(&['\'', ';'][..], "")).trim().to_string();
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
    let row: (String, String) = sqlx::query_as("SELECT login, longpw FROM admin")
        .fetch_one(&mut conn).await?;
    Ok(row)
}

async fn set_login(username: &str, password: &str) -> Result<AnyQueryResult, sqlx::Error> {
    let (db_username, db_password, hostname, database) = get_mysql_credentials()?;
    let mut conn = AnyConnection::
        connect(&format!("mysql://{}:{}@{}/{}", db_username, db_password, hostname, database)).await?;
    sqlx::query("UPDATE admin SET longpw = ENCRYPT(?) WHERE login = ?")
        .bind(password)
        .bind(username)
        .execute(&mut conn)
        .await
}

async fn reset_login(username: &str, password_hash: &str) -> Result<AnyQueryResult, sqlx::Error> {
    let (db_username, db_password, hostname, database) = get_mysql_credentials()?;
    let mut conn = AnyConnection::
        connect(&format!("mysql://{}:{}@{}/{}", db_username, db_password, hostname, database)).await?;
    sqlx::query("UPDATE admin SET longpw = ? WHERE login = ?")
        .bind(password_hash)
        .bind(username)
        .execute(&mut conn)
        .await
}