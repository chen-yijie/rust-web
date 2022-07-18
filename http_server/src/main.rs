use http_server::httprequest::HttpRequest;
use http_server::httprequest::Method;
use http_server::httprequest::Resource;
use http_server::httpresponse::HttpResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpListener;
use std::{env, fs};

pub struct Server<'a> {
    socket_addr: &'a str,
}

pub trait Handler {
    fn handle( req: &HttpRequest ) -> HttpResponse;
    fn load_file( file_name: &str ) -> Option<String> {
        let default_path = format!( "{}/public", env!( "CARGO_MANIFEST_DIR" ) );
        let public_path = env::var( "PUBLIC_PATH" ).unwrap_or( default_path );
        let full_path = format!( "{}/{}", public_path, file_name );

        let contents = fs::read_to_string( full_path );
        contents.ok()
    }
}

pub struct StaticPageHandler;
pub struct PageNotFoundHandler;
pub struct WebServiceHandler;

#[derive( Serialize, Deserialize )]
pub struct OrderStatus {
    order_id: i32,
    order_date: String,
    order_status: String,
}

impl Handler for PageNotFoundHandler {
    fn handle( _req: &HttpRequest ) -> HttpResponse {
        HttpResponse::new( "404", None, Self::load_file("404.html" ) )
    }
}

impl Handler for StaticPageHandler {
    fn handle( req: &HttpRequest ) -> HttpResponse {
        let Resource::Path( s ) = &req.resource;
        let route: Vec<&str> = s.split( "/" ).collect();
        match route[1] {
            "" => HttpResponse::new("200", None, Self::load_file("index.html" ) ),
            "health" => HttpResponse::new("200", None, Self::load_file("health.html" ) ),
            path => match Self::load_file( path ) {
                Some( contents ) => {
                    let mut map: HashMap<&str, &str> = HashMap::new();
                    if path.ends_with( "css" ) {
                        map.insert( "Content-Type", "text/css" );
                    } else if path.ends_with( "js" ) {
                        map.insert( "Content-Type", "text/javascript" );
                    } else {
                        map.insert( "Content-Type", "text/html" );
                    }

                    HttpResponse::new( "200", Some( map ), Some( contents ) )
                }
                None => HttpResponse::new( "404", None, Self::load_file("404.html" ) ),
            },
        }
    }
}

impl WebServiceHandler {
    fn load_json() -> Vec<OrderStatus> {
        let default_path = format!( "{}/data", env!("CARGO_MANIFEST_DIR" ) );
        let data_path = env::var( "DATA_PATH" ).unwrap_or(default_path );
        let full_path = format!( "{}/{}", data_path, "orders.json" );
        let json_contents = fs::read_to_string( full_path );
        let orders: Vec<OrderStatus> =
            serde_json::from_str( json_contents.unwrap().as_str() ).unwrap();
        orders
    }
}

impl Handler for WebServiceHandler {
    fn handle( req: &HttpRequest ) -> HttpResponse {
        let Resource::Path( s ) = &req.resource;
        let route: Vec<&str> = s.split( "/" ).collect();
        // localhost:3000/api/shipping/orders
        match route[2] {
            "shipping" if route.len() > 2 && route[3] == "orders" => {
                let body = Some( serde_json::to_string( &Self::load_json() ).unwrap() );
                let mut headers: HashMap<&str, &str> = HashMap::new();
                headers.insert( "Content-Type", "application/json" );
                HttpResponse::new( "200", Some( headers ), body )
            }
            _ => HttpResponse::new( "404", None, Self::load_file("404.html" ) ),
        }
    }
}

pub struct Router;

impl Router {
    pub fn route( req: HttpRequest, stream: &mut impl Write ) -> () {
        match req.method {
            Method::Get => match &req.resource {
                Resource::Path( s ) => {
                    let route: Vec<&str> = s.split( '/' ).collect();
                    match route[1] {
                        "api" => {
                            let resp: HttpResponse = WebServiceHandler::handle( &req );
                            let _ = resp.send_response( stream );
                        }
                        _ => {
                            let resp: HttpResponse = StaticPageHandler::handle( &req );
                            let _ = resp.send_response( stream );
                        }
                    }
                }
            },
            _ => {
                let resp: HttpResponse = PageNotFoundHandler::handle( &req );
                let _ = resp.send_response( stream );
            }
        }
    }
}

impl<'a> Server<'a> {
    pub fn new( socket_addr: &'a str ) -> Self {
        Server { socket_addr }
    }

    pub fn run( &self ) {
        let connection_listener = TcpListener::bind( self.socket_addr ).unwrap();

        println!( "Running on {}", self.socket_addr );

        for stream in connection_listener.incoming() {
            let mut stream = stream.unwrap();
            println!( "Connection established" );

            let mut read_buffer = [0; 200];
            stream.read( &mut read_buffer ).unwrap();
            let req: HttpRequest = String::from_utf8( read_buffer.to_vec() ).unwrap().into();

            Router::route( req, &mut stream );
        }
    }
}

fn main() {
    let server = Server::new( "localhost:3000" );
    server.run();
}
