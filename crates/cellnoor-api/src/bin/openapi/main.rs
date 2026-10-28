fn main() {
    let (openapi_docs, _) = cellnoor_api::api::router();
    println!("{}", serde_json::to_string(&openapi_docs).unwrap());
}
