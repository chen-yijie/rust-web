use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::Mutex;

use actix_web::middleware::Logger;

#[derive( Debug, Clone, Serialize, Deserialize )]
pub struct Course {
    pub teacher_id: u32,
    pub id: Option<u32>,
    pub name: String,
    pub time: Option<NaiveDateTime>,
}

impl From<web::Json<Course>> for Course {
    fn from( course: web::Json<Course> ) -> Self {
        Course {
            teacher_id: course.teacher_id,
            id: course.id,
            name: course.name.clone(),
            time: course.time,
        }
    }
}

pub struct AppState {
    pub health_check_response: String,
    pub visit_count: Mutex<u32>,
    pub courses: Mutex<Vec<Course>>,
}

pub async fn health_check_handler( app_state: web::Data<AppState> ) -> HttpResponse {
    let health_check_response = &app_state.health_check_response;
    let mut visit_count = app_state.visit_count.lock().unwrap();

    let response = format!( "{} {} times", health_check_response, visit_count );
    *visit_count += 1;
    HttpResponse::Ok().json( &response )
}

pub async fn new_course(
    new_course: web::Json<Course>,
    app_state: web::Data<AppState>,
 ) -> HttpResponse {
    println!( "Received new course" );
    let course_count = app_state
        .courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .filter( |course| course.teacher_id == new_course.teacher_id )
        .collect::<Vec<Course>>()
        .len();
    let new_course = Course {
        teacher_id: new_course.teacher_id,
        id: Some( course_count as u32 + 1 ),
        name: new_course.name.clone(),
        time: Some( Utc::now().naive_utc() ),
    };

    app_state.courses.lock().unwrap().push( new_course );
    HttpResponse::Ok().json( "Course added" )
}

pub async fn get_courses_for_teacher(
    app_state: web::Data<AppState>,
    params: web::Path<( u32 )>,
 ) -> HttpResponse {
    let teacher_id = params.into_inner();

    let filtered_courses = app_state
        .courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .filter( |course| course.teacher_id == teacher_id )
        .collect::<Vec<Course>>();

    if filtered_courses.len() > 0 {
        HttpResponse::Ok().json( filtered_courses )
    } else {
        HttpResponse::Ok().json( "No courses found for teacher".to_string() )
    }
}

pub async fn get_course_detail( 
    app_state: web::Data<AppState>,
    params: web::Path<(u32, u32)>,
) -> HttpResponse {

    let ( teacher_id, course_id ) = params.into_inner();

    let selected_course = app_state
        .courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .find( | x | x.teacher_id == teacher_id && x.id == Some( course_id ))
        .ok_or( "Course not found" );

    if let Ok( course ) = selected_course {
        HttpResponse::Ok().json( course )
    } else {
        HttpResponse::Ok().json( "Course not found".to_string() )
    }
}

// 配置route
pub fn general_routes( cfg: &mut web::ServiceConfig ) {
    cfg.route( "/health", web::get().to(health_check_handler ) );
}

// 配置范围route
pub fn course_routes( cfg: &mut web::ServiceConfig ) {
    cfg.service(
        web::scope( "/courses" )
            .route( "/", web::post().to(new_course ) )
            .route( "/{user_id}", web::get().to(get_courses_for_teacher ) )
            .route( "/{user_id}/{course_id}", web::get().to( get_course_detail ) ),
    );
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env( env_logger::Env::new().default_filter_or( "info" ) );

    let shared_data = web::Data::new( AppState {
        health_check_response: "I'm OK.".to_string(),
        visit_count: Mutex::new( 0 ),
        courses: Mutex::new( vec![] ),
    } );

    let app = move || {
        App::new()
            .app_data( shared_data.clone() )
            .configure( general_routes )
            .configure( course_routes )
            .wrap( Logger::default() )
        // .wrap(Logger::new("%a %{User-Agent}i"));
    };

    HttpServer::new( app).bind( "127.0.0.1:3000" )?.run().await
}

#[cfg( test )]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[actix_rt::test]
    async fn post_course_test() {
        let course = web::Json( Course {
            teacher_id: 1,
            name: "Test course".into(),
            id: None,
            time: None,
        } );

        let app_state: web::Data<AppState> = web::Data::new( AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new( 0 ),
            courses: Mutex::new( vec![] ),
        } );

        let resp = new_course( course, app_state ).await;
        assert_eq!( resp.status(), StatusCode::OK );
    }

    #[actix_rt::test]
    async fn get_all_courses_success() {
        let app_state: web::Data<AppState> = web::Data::new( AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new( 0 ),
            courses: Mutex::new( vec![] ),
        } );

        let teacher_id: web::Path<( u32 )> = web::Path::from( ( 1 ) );
        let resp = get_courses_for_teacher( app_state, teacher_id ).await;

        assert_eq!( resp.status(), StatusCode::OK );
    }

    #[actix_rt::test]
    async fn get_one_course_success() {
        let app_state: web::Data<AppState> = web::Data::new( AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new( 0 ),
            courses: Mutex::new( vec![] ),
        });

        let params: web::Path<(u32, u32)> = web::Path::from( (1, 1));
        let resp = get_course_detail( app_state, params ).await;
        assert_eq!( resp.status(), StatusCode::OK );
    }
}
