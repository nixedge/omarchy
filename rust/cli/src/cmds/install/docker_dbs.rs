use crate::desktop;
use std::process::Command;

const DB_OPTIONS: &[&str] = &["MySQL", "PostgreSQL", "Redis", "MongoDB", "MariaDB", "MSSQL"];

fn install_db(db: &str) {
    println!("Installing {db}...");
    let status = match db {
        "MySQL" => Command::new("sudo")
            .args(["docker", "run", "-d", "--restart", "unless-stopped",
                   "-p", "127.0.0.1:3306:3306", "--name=mysql8",
                   "-e", "MYSQL_ROOT_PASSWORD=",
                   "-e", "MYSQL_ALLOW_EMPTY_PASSWORD=true",
                   "mysql:8.4"])
            .status(),
        "PostgreSQL" => Command::new("sudo")
            .args(["docker", "run", "-d", "--restart", "unless-stopped",
                   "-p", "127.0.0.1:5432:5432", "--name=postgres18",
                   "-e", "POSTGRES_HOST_AUTH_METHOD=trust",
                   "postgres:18"])
            .status(),
        "MariaDB" => Command::new("sudo")
            .args(["docker", "run", "-d", "--restart", "unless-stopped",
                   "-p", "127.0.0.1:3306:3306", "--name=mariadb11",
                   "-e", "MARIADB_ROOT_PASSWORD=",
                   "-e", "MARIADB_ALLOW_EMPTY_ROOT_PASSWORD=true",
                   "mariadb:11.8"])
            .status(),
        "Redis" => Command::new("sudo")
            .args(["docker", "run", "-d", "--restart", "unless-stopped",
                   "-p", "127.0.0.1:6379:6379", "--name=redis",
                   "redis:7"])
            .status(),
        "MongoDB" => Command::new("sudo")
            .args(["docker", "run", "-d", "--restart", "unless-stopped",
                   "-p", "127.0.0.1:27017:27017", "--name=mongodb",
                   "-e", "MONGO_INITDB_ROOT_USERNAME=admin",
                   "-e", "MONGO_INITDB_ROOT_PASSWORD=admin123",
                   "mongo:noble"])
            .status(),
        "MSSQL" => Command::new("sudo")
            .args(["docker", "run", "-d", "--restart", "unless-stopped",
                   "-p", "127.0.0.1:1433:1433", "--name=mssql",
                   "-e", "MSSQL_PID=Developer",
                   "-e", "ACCEPT_EULA=Y",
                   "-e", "MSSQL_SA_PASSWORD=@dmin123",
                   "mcr.microsoft.com/mssql/server:2022-CU12-ubuntu-22.04"])
            .status(),
        other => {
            eprintln!("Unknown database: {other}");
            return;
        }
    };
    if let Err(e) = status {
        eprintln!("Failed to start {db}: {e}");
    }
}

pub fn run(dbs: &[String]) -> i32 {
    let choices: Vec<String> = if dbs.is_empty() {
        match desktop::gum_choose(DB_OPTIONS, "Select database (return to install, esc to cancel)") {
            Some(v) => v,
            None => {
                println!("No databases selected for installation.");
                return 0;
            }
        }
    } else {
        dbs.to_vec()
    };

    for db in &choices {
        install_db(db.trim());
    }
    0
}
