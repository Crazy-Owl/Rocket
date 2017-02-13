use rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, ContentType};
use rocket::Response;

use std::collections::HashMap;
use std::sync::Mutex;

macro_rules! run_test {
    ($req:expr, $test_fn:expr, $rocket:expr) => ({
        let mut req = $req;
        $test_fn(req.dispatch_with($rocket));
    })
}

#[test]
fn bad_get_put() {
    let rocket = rocket::ignite()
        .mount("/message", routes![super::new, super::update, super::get])
        .manage(Mutex::new(HashMap::<super::ID, String>::new()))
        .catch(errors![super::not_found]);

    // Try to get a message with an ID that doesn't exist.
    let req = MockRequest::new(Get, "/message/99").header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::NotFound);

        let body = response.body().unwrap().into_string().unwrap();
        assert!(body.contains("error"));
        assert!(body.contains("Resource was not found."));
    }, &rocket);

    // Try to get a message with an invalid ID.
    let req = MockRequest::new(Get, "/message/hi").header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::NotFound);
        let body = response.body().unwrap().into_string().unwrap();
        assert!(body.contains("error"));
    }, &rocket);

    // Try to put a message without a proper body.
    let req = MockRequest::new(Put, "/message/80").header(ContentType::JSON);
    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::BadRequest);
    }, &rocket);

    // Try to put a message for an ID that doesn't exist.
    let req = MockRequest::new(Put, "/message/80")
        .header(ContentType::JSON)
        .body(r#"{ "contents": "Bye bye, world!" }"#);

    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::NotFound);
    }, &rocket);
}

#[test]
fn post_get_put_get() {
    let rocket = rocket::ignite()
        .mount("/message", routes![super::new, super::update, super::get])
        .manage(Mutex::new(HashMap::<super::ID, String>::new()))
        .catch(errors![super::not_found]);
    // Check that a message with ID 1 doesn't exist.
    let req = MockRequest::new(Get, "/message/1").header(ContentType::JSON);
    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::NotFound);
    }, &rocket);

    // Add a new message with ID 1.
    let req = MockRequest::new(Post, "/message/1")
        .header(ContentType::JSON)
        .body(r#"{ "contents": "Hello, world!" }"#);

    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::Ok);
    }, &rocket);

    // Check that the message exists with the correct contents.
    let req = MockRequest::new(Get, "/message/1") .header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::Ok);
        let body = response.body().unwrap().into_string().unwrap();
        assert!(body.contains("Hello, world!"));
    }, &rocket);

    // Change the message contents.
    let req = MockRequest::new(Put, "/message/1")
        .header(ContentType::JSON)
        .body(r#"{ "contents": "Bye bye, world!" }"#);

    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::Ok);
    }, &rocket);

    // Check that the message exists with the updated contents.
    let req = MockRequest::new(Get, "/message/1") .header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::Ok);
        let body = response.body().unwrap().into_string().unwrap();
        assert!(!body.contains("Hello, world!"));
        assert!(body.contains("Bye bye, world!"));
    }, &rocket);
}
