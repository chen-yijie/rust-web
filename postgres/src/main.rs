use chrono::NaiveDateTime;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Debug)]
pub struct Course {
    pub id: i32,
    pub teacher_id: i32,
    pub name: String,
    pub time: Option<NaiveDateTime>,
}

#[actix_rt::main]
async fn main() -> Result<(), sqlx::Error> {
    // 读取所有的环境变量
    dotenv().ok();

    // for (key, value) in env::vars() {
    //     println!("环境变量内容：{}: {}", key, value);
    // }

    let database_url = env::var("DATABASE_URL")
        .expect("数据库连接字符串获取失败，请检查env文件是否已配置数据库连接字符串");

    println!("数据库连接字符串是：{}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        // .connect("postgres://cml:123456@192.168.1.239:5432/rust_sqlx")
        .connect(&database_url)
        .await?;

    println!("db_pool is : {:?}", pool);

    //查询所有
    let list = sqlx::query!("SELECT * FROM course")
        .fetch_all(&pool)
        .await?;
    let mut vec = vec![];
    for row in list {
        vec.push(Course {
            id: row.id,
            teacher_id: row.teacher_id,
            name: row.name,
            time: Some(chrono::NaiveDateTime::from(row.time.unwrap())),
        })
    }

    println!("数据库中的所有数据：{:#?}", vec);

    //查询单个
    let list2 = sqlx::query!(r#"select * from course where id = $1"#, 1)
        .fetch_all(&pool)
        .await?;
    let mut vec2 = vec![];
    for row in list2 {
        vec2.push(Course {
            id: row.id,
            teacher_id: row.teacher_id,
            name: row.name,
            time: Some(chrono::NaiveDateTime::from(row.time.unwrap())),
        })
    }
    println!("查询单个{:#?}", vec2);

    //增加
    // let insert = sqlx::query!(
    //     r#"INSERT INTO course VALUES ($1, $2, $3)"#,
    //     100000,
    //     11,
    //     "gg"
    // )
    // .fetch_all(&pool)
    // .await?;
    //更新

    let _update = sqlx::query!(r#"update course set name=$1"#, "ogg")
        .fetch_all(&pool)
        .await?;
    Ok(())
}
