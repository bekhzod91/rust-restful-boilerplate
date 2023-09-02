use uuid::Uuid;

async fn auth<T>(mut req: Request<T>, next: Next<T>) -> Result<Response, StatusCode> {
    let request_id = Uuid::new_v4().to_string();
    req.extensions_mut().insert();
    Ok(next.run(req).await)
}
