use tide::http::headers::HeaderValue;
use tide::Request;
use tide::prelude::*;
use rusqlite::{params, Connection, Result};
use tide::Response;
use tide::StatusCode;
use tide::security::{CorsMiddleware, Origin};



const DATABASE_URL: &str = "bookmark.sqlite";

#[derive(Debug, Deserialize)]
struct BookMarkFile{
    pub name: String,
    pub data: String,
    pub tag: Vec<String>,
}


#[derive(Debug, Serialize)]
struct BookMarkNameTags{
    pub name: String,
    pub tags: Vec<String>,
}

struct BookMarksTable{
    pub id: i64,
    pub name: String,
    pub data: String,
}

struct TagsTable{
    pub id: i64,
    pub name: String,
}

struct BookmarkTagsTable{
    pub bookmark_id: i64,
    pub tag_id: i64,
}
#[async_std::main]
async fn main() -> tide::Result<()> {
    init_db()?;
    let mut app = tide::new();
        app.with(CorsMiddleware::new()
        .allow_origin(Origin::from("*"))
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_headers("Content-Type, Authorization".parse::<HeaderValue>().unwrap())
        .allow_credentials(false));
    app.at("/bookmarkfile").post(bookmark);
    app.at("/bookmarkfile").get(get_all_bookmark_name_tags);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn init_db() -> Result<()> {
    let conn = Connection::open(DATABASE_URL)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            data TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bookmark_tags (
            bookmark_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (bookmark_id, tag_id)
        )",
        [],
    )?;
    Ok(())
}
// read a json body from a front server
async fn bookmark(mut req: Request<()>) -> tide::Result {
    let result = req.body_json().await;
    
    match result {
        Ok(BookMarkFile { name, data, tag }) => {
            println!("Received JSON: name={}, data={}, tags={:?}", name, data, tag);
            create_bookmark(name, data, tag)
        }
        Err(e) => {
            println!("Failed to parse JSON: {}", e);
            Ok(Response::new(StatusCode::BadRequest))
        }
    }
}

fn create_bookmark(name: String, data: String, tags: Vec<String>) -> tide::Result<tide::Response> {
    let conn = Connection::open(DATABASE_URL).map_err(|e| {
        tide::Error::new(StatusCode::InternalServerError, e)
    })?;
    
    let mut tag_ids = Vec::<i64>::new();
    for tag in tags {
        let tag_id = get_or_create_tag_id(&conn, &tag).map_err(|e| {
            tide::Error::new(StatusCode::InternalServerError, e)
        })?;
        tag_ids.push(tag_id);
    }

    conn.execute("INSERT INTO bookmarks (name, data) VALUES (?1, ?2)", params![name, data])
        .map_err(|e| {
            tide::Error::new(StatusCode::InternalServerError, e)
        })?;
    
    let bookmark_id = conn.last_insert_rowid();
    for tag_id in tag_ids {
        conn.execute("INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES (?1, ?2)", params![bookmark_id, tag_id])
            .map_err(|e| {
                tide::Error::new(StatusCode::InternalServerError, e)
            })?;
    }

    Ok(Response::new(StatusCode::Ok))
}
fn get_or_create_tag_id(conn: &Connection, tag_name: &str) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT id FROM tags WHERE name = ?1")?;
    let tag_id_result: Result<i64> = stmt.query_row(params![tag_name], |row| row.get(0));

    match tag_id_result {
        Ok(tag_id) => Ok(tag_id), 
        Err(_) => {
            conn.execute("INSERT INTO tags (name) VALUES (?1)", params![tag_name])?;
            Ok(conn.last_insert_rowid()) 
        }
    }
}

async fn get_all_bookmark_name_tags(_req: Request<()>) -> tide::Result {
    
    let conn = Connection::open(DATABASE_URL).map_err(|e| {
        tide::Error::new(StatusCode::InternalServerError, e)
    })?;
    
    let bookmarks = get_book_marks_table()?;
    
    let mut response_bookmark_name_tags: Vec<BookMarkNameTags> = Vec::new();
    for bookmark in bookmarks {
        let tags = get_tags_by_bookmark_id(bookmark.id);
        response_bookmark_name_tags.push(BookMarkNameTags {
            name: bookmark.name,
            tags: tags.unwrap(),
        });
    }
    
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(tide::Body::from_json(&response_bookmark_name_tags)?);
    Ok(res)
    
}

fn get_book_marks_table() -> Result<Vec<BookMarksTable>, tide::Error> {
    
    let conn = Connection::open(DATABASE_URL).map_err(|e| {
        tide::Error::new(StatusCode::InternalServerError, e)
    })?;
    let mut stmt = conn.prepare("SELECT * FROM bookmarks")?;
    
    let bookmark_iter = stmt.query_map([], |row| {
        Ok(BookMarksTable {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
        })
    }).map_err(|e| {
        tide::Error::new(StatusCode::InternalServerError, e)
    })?;
    
    let mut bookmarks: Vec<BookMarksTable> = Vec::new();
    for bookmark_result in bookmark_iter {
        let book_marks_table = bookmark_result.map_err(|e| {
            
            tide::Error::new(StatusCode::InternalServerError, e)
        })?;
        
        bookmarks.push(book_marks_table);
    }
    Ok(bookmarks)
}

fn get_tags_by_bookmark_id(bookmark_id: i64) -> tide::Result<Vec<String>> {
    
    let conn = Connection::open(DATABASE_URL).map_err(|e| {
        tide::Error::new(StatusCode::InternalServerError, e)
    })?;
    
    let mut stmt = conn.prepare("SELECT tag_id FROM bookmark_tags WHERE bookmark_id = ?1")?;
    let tag_ids = stmt.query_map(params![bookmark_id], |row| {
        Ok(row.get(0)?)
    }).map_err(|e| {
        tide::Error::new(StatusCode::InternalServerError, e)
    })?;
    
    let mut tags: Vec<String> = Vec::new();
    for tag_id_result in tag_ids {
        let tag_id = tag_id_result.map_err(|e| {
            tide::Error::new(StatusCode::InternalServerError, e)
        })?;
        
        let tag = get_tag_by_id(&conn, tag_id).map_err(|e| {
            tide::Error::new(StatusCode::InternalServerError, e)
        })?;
        tags.push(tag);
    }

    Ok(tags)
}

fn get_tag_by_id(conn: &Connection, tag_id: i64) -> Result<String> {
    let mut stmt = conn.prepare("SELECT name FROM tags WHERE id = ?1")?;
    let tag_name: Result<String> = stmt.query_row(params![tag_id], |row| row.get(0));
    tag_name
}