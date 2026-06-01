fn main() {
    let (openapi_docs, _) = cellnoor::api::router();
    println!("{}", serde_json::to_string(&openapi_docs).unwrap());
}
