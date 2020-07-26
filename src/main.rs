use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use mongodb::bson::doc;
use mongodb::{Client, Collection};

#[derive(Clone)]
struct CountService {
    coll: Collection,
}

impl CountService {
    fn new(coll: Collection) -> CountService {
        CountService { coll }
    }

    async fn update_count(&self, username: &str) -> i32 {
        let result = self
            .coll
            .find_one_and_update(doc! {"name":username}, doc! {"$inc": {"count":1}}, None)
            .await
            .expect("update");
        match result {
            None => {
                self.coll
                    .insert_one(doc! {"name": username, "count":1}, None)
                    .await
                    .expect("insert");
                1
            }
            Some(x) => x.get_i32("count").expect("value errror"),
        }
    }
}

async fn get_count(req: HttpRequest, data: web::Data<CountService>) -> HttpResponse {
    let mut count: i32 = 0;
    let name = req.match_info().get("name");
    count = match name {
        Some(x) => data.update_count(x).await,
        None => -1,
    };

    let svg = format!("<?xml version=\"1.0\" standalone=\"no\"?><svg height=\"30\" width=\"200\" version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink= \"http://www.w3.org/1999/xlink\"><text x=\"0\" y=\"15\" fill=\"black\">visits so far: {}</text></svg>", count);
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .header("Cache-Control", "no-cache,max-age=0,no-store,s-maxage=0,proxy-revalidate")
        .header("Pragma", "no-cache")
        .header("Expires", "-1")
        .body(svg)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let client: Client = Client::with_uri_str("mongodb+srv://<user>:<pass>@cluster0-k1eqt.azure.mongodb.net/<db>?retryWrites=true&w=majority")
        .await
        .expect("connect");
    let coll: Collection = client.database("gh").collection("visit");
    let svc = CountService::new(coll.clone());
    HttpServer::new(move || {
        App::new()
            .data(svc.clone())
            .route("/{name}", web::get().to(get_count))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
