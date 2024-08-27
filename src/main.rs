use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use std::env;
use std::sync::Arc;
use dotenv::dotenv;
use std::collections::HashMap;
use chrono::Duration;

// Structs for JSON serialization/deserialization
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject (user id)
    exp: usize, // expiration time (as UTC timestamp)
}

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: String,
    name: String,
}

// Mock database

// Function to start server
async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut users = HashMap::new();
    users.insert("1".to_string(), User { id: "1".to_string(), name: "Alice".to_string() });
    let addr = ([0, 0, 0, 0], 3000).into();
    
    let service = make_service_fn(|_conn| {
        let users = Arc::new(users.clone());
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let arc = Arc::clone(&users);
                handle_request(req, arc)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}

// Request handler
async fn handle_request(
    req: Request<Body>,
    users: Arc<HashMap<String, User>>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/authenticate") => {
            // Here would be logic to authenticate user, for simplicity we'll just generate a token
            let token = create_jwt("1").unwrap();
            Ok(Response::new(Body::from(token)))
        },
        (&Method::GET, "/user") => {
            if let Some(auth_header) = req.headers().get("Authorization") {
                let auth_header = auth_header.to_str().unwrap();
                if let Ok(token_data) = validate_jwt(auth_header.replace("Bearer ", "").as_str()) {
                    if let Some(user) = users.get(&token_data.claims.sub) {
                        let user_json = serde_json::to_string(user).unwrap();
                        return Ok(Response::new(Body::from(user_json)));
                    }
                }
            }
            Ok(Response::builder()
               .status(StatusCode::UNAUTHORIZED)
               .body(Body::from("Unauthorized"))
               .unwrap())
        },
        _ => Ok(Response::builder()
           .status(StatusCode::NOT_FOUND)
           .body(Body::empty())
           .unwrap()),
    }
}

fn create_jwt(uid: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now() + Duration::days(1);
    let claims = Claims { sub: uid.to_string(), exp: expiration.timestamp() as usize };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref()))?;
    Ok(token)
}

fn validate_jwt(token: &str) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, &DecodingKey::from_secret("secret".as_ref()), &Validation::default())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    start_server().await
}
