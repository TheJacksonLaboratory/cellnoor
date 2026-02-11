use cellnoor_api::api::routes::router;

fn main() {
    let (_, openapi_docs) = router();
    println!("{}", serde_json::to_string(&openapi_docs).unwrap());
}
