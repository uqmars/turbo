use reqwest::Response;
use rusqlite::Connection;

const DB_FILE_NAME: &str = "aoc.db3";

pub fn create_contestants_db() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS contestants (
            unique_id   INTEGER PRIMARY KEY,
            aoc_id      INTEGER,
            aoc_name    TEXT NOT NULL,
            disc_id     INTEGER
        ) STRICT;",
        (),
    )?;

    Ok(())
}

struct Contestant {
    unique_id: u64,
    aoc_id: u64,
    aoc_name: String,
    disc_id: u64,
}

pub fn get_aoc_from_discord(id: u64) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    let response = conn.query_row(
        "SELECT aoc_name FROM contestants WHERE disc_id = ?1",
        (id,),
        |row| Ok(row.get(0)?),
    );
    let aoc_name = match response {
        Ok(x) => Some(x),
        Err(_) => None,
    };
    Ok(aoc_name)
}

fn get_discord_from_aoc(username: String) -> Result<u64, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    let disc_id = conn
        .query_row(
            "SELECT disc_id FROM contestants WHERE aoc_name = ?1",
            (username,),
            |row| Ok(row.get(0)?),
        )
        .unwrap();
    Ok(disc_id)
}

pub fn insert_registration_db(
    disc_id: u64,
    aoc_name: &str,
    aoc_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    conn.execute(
        "INSERT INTO contestants (disc_id, aoc_name, aoc_id)
            VALUES (?1, ?2, ?3);",
        (disc_id, aoc_name, aoc_id),
    )?;

    Ok(())
}

pub fn remove_registration_db(disc_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    conn.execute(
        "DELETE FROM contestants
            WHERE disc_id=?1;",
        (disc_id,),
    )?;

    Ok(())
}

fn get_all_registrations_db() -> Result<Vec<Contestant>, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    let select_sql = "SELECT unique_id, disc_id, aoc_name, aoc_id
        FROM    contestants";
    let mut stmt = conn.prepare(select_sql)?;
    let mut rules: Vec<Contestant> = Vec::new();
    let rule_iter = stmt.query_map((), |row| {
        Ok(Contestant {
            unique_id: row.get(0)?,
            disc_id: row.get(1)?,
            aoc_name: row.get(2)?,
            aoc_id: row.get(3)?,
        })
    })?;

    for rule in rule_iter {
        rules.push(rule.unwrap());
    }

    Ok(rules)
}
