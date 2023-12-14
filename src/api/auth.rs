use std::sync::Arc;
use serde::Serialize;
use serde::Deserialize;
use axum::extract::Extension;
use axum::response::IntoResponse;
use axum::Json;
use axum::http::StatusCode;
use tower_http::request_id::RequestId;
use bson::oid::ObjectId;

use bson::doc;
use super::code;
use super::state::State;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json;
use redis::AsyncCommands;


pub fn token_generate() -> String {
    rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(32)
    .map(char::from)
    .collect()
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    #[serde(rename(deserialize = "_id", serialize = "id"))]
    id: ObjectId,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInRequest {
    username: String,
    password: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SignInResponseData {
    token: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SignInResponse {
    request_id: String,
    code: String,
    data: Option<SignInResponseData>
}

pub async fn sign_in(
    Json(payload): Json<SignInRequest>,
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let request_id = _request_id.header_value().to_str().unwrap().to_string();

    let collection = _state.db.collection("account");
    
    let filter = doc! {
        "username": payload.username.trim()
    };

    let result: Option<Account> = collection.find_one(filter, None).await;
    
    if result.is_none() {
        let response = SignInResponse {
            request_id, 
            code: code::INVALID_CREDENTIALS.to_owned(),
            data: None
        };
    
        return (StatusCode::BAD_REQUEST, Json(response))
    }

    let account = result.unwrap();
    if account.password != payload.password {
        let response = SignInResponse {
            request_id, 
            code: code::INVALID_CREDENTIALS.to_owned(),
            data: None
        };

        return (StatusCode::BAD_REQUEST, Json(response))
    }

    let token = token_generate();
    let key = format!("auth:{}", token);
    let value = serde_json::to_string(&account).unwrap();

    let mut con = _state.redis_client.get_async_connection().await.unwrap();
    let _: () = con.set(key, value).await.unwrap();

    let response = SignInResponse {
        request_id, 
        code: code::SUCCESSED.to_owned(),
        data: Some(SignInResponseData {
            token,
        })
    };

    (StatusCode::OK, Json(response))
}



EXPLAIN ANALYZE
SELECT count(*)
FROM "order" 
 
WHERE 
    "order".is_deleted = false 
    AND "order".company_id IN (2021, 1987, 1926, 1778, 1203, 1773, 1589, 1397, 1517, 1513, 1413, 1361) 
    AND (
        "order".warehouse_id IN (3074, 3072, 3044, 2928, 2803, 2798, 2783, 2779, 2729, 2715, 2692, 2632, 2630, 2597, 1790) OR 
        "order".warehouse_id IS NULL
    )
ORDER BY "order".id DESC  
LIMIT 100  
;


EXPLAIN ANALYZE SELECT *
FROM "order" 
JOIN integrations ON integrations.id = "order".channel_id 
JOIN companies ON companies.id = "order".company_id 
WHERE 
    "order".is_deleted = false 
    AND "order".company_id = 1360
    ORDER BY "order".id DESC

    {2050, 2054, 2569, 1555, 1556, 2069, 1045, 1058, 2595, 1062, 2602, 2612, 2616, 2107, 2625, 2113, 2629, 2119, 2634, 2123, 2125, 2640, 1637, 2662, 615, 2675, 2676, 2678, 2168, 2174, 2177, 2178, 643, 2690, 2190, 1691, 1182, 2719, 2720, 2721, 2722, 2723, 2725, 689, 1718, 2742, 1734, 2758, 711, 1230, 1747, 2778, 2779, 2790, 742, 1766, 2281, 1779, 2292, 2293, 2295, 2299, 2300, 1277, 2301, 2826, 2315, 2830, 2833, 1811, 2836, 2325, 2326, 2839, 2840, 2841, 2842, 2844, 2845, 1821, 2847, 2848, 2850, 2851, 2852, 2853, 2860, 2862, 2863, 2357, 2870, 2359, 2360, 2362, 2363, 2876, 2877, 2366, 2367, 2878, 2369, 2881, 2882, 2885, 2886, 2888, 2376, 2890, 2379, 2891, 2889, 1871, 2399, 2403, 876, 2429, 383, 1930, 2442, 2445, 2446, 2467, 2468, 421, 2474, 1450, 2476, 1966, 2481, 2482, 2483, 2485, 2486, 2487, 2489, 2490, 2491, 2492, 1981, 2493, 2494, 2497, 1987, 2501, 2504, 1997, 2510, 2525, 2014, 2015, 1503, 2534, 2535, 2539, 2552, 2557}