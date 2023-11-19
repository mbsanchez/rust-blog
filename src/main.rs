#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use tera::Tera;
use dotenv::dotenv;
use std::env;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel::r2d2::Pool;

use self::models::{Post, NewPostHandler};
use self::schema::posts::dsl::*;

// fn main() {
//     dotenv().ok();

//     let db_url = env::var("DATABASE_URL").expect("db url variable no encontrada");

//     let mut conn = PgConnection::establish(&db_url).expect("error de conexión a la base de datos");

//     use self::models::{Post, NewPost};
//     use self::schema::posts;
//     use self::schema::posts::dsl::*;

//     // let new_post = NewPost {
//     //     title: "Mi tercer blogpost",
//     //     body: "This is the post body",
//     //     slug: "tercer-post"
//     // };
//     // diesel::insert_into(posts::table).values(&new_post).get_result::<Post>(&mut conn).expect("error de creación de objeto");

//     // Select * from posts
//     let all_posts = posts.limit(5).load::<Post>(&mut conn).expect("error de ejecución de la query");

//     for post in all_posts {
//         println!("{:?}", post)
//     }
// }

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::Data;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// #[get("/tera_test")]
// async fn tera_test(template_manager: web::Data<tera::Tera>) -> impl Responder {

//     // Creamos un contexto para pasarle datos al template
//     let ctx = tera::Context::new();

//     // Enviamos el template que queremos localizándolo por su nombre
//     HttpResponse::Ok().content_type("text/html").body(
//         template_manager.render("index.html", &ctx).unwrap()
//     )
// }

#[get("/posts")]
async fn index_posts(pool: web::Data<DbPool>, template_manager: web::Data<tera::Tera>)-> impl Responder {
    let mut conn = pool.get().expect("DB not found");

    match web::block(move || {posts.load::<Post>(&mut conn)}).await {
        Ok(data)=> {
            let data = data.unwrap();
            let mut ctx = tera::Context::new();
            ctx.insert("posts", &data);

            HttpResponse::Ok().content_type("text/html").body(
                template_manager.render("index.html", &ctx).unwrap()
            )
        },
        Err(_err)=> HttpResponse::Ok().body("Error al recibir la data")
    }
}

#[get("/posts/{post_slug}")]
async fn show_post(
    pool: web::Data<DbPool>, 
    template_manager: web::Data<tera::Tera>,
    post_slug: web::Path<String>
)-> impl Responder {
    let mut conn = pool.get().expect("DB not found");
    let url_slug = post_slug.into_inner();

    match web::block(move || { posts.filter(slug.eq(url_slug)).load::<Post>(&mut conn) }).await {
        Ok(data)=> {
            let data = data.unwrap();

            if data.len() == 0 {
                return HttpResponse::NotFound().finish();
            }

            let post = &data[0];

            let mut ctx = tera::Context::new();
            ctx.insert("post", &post);

            HttpResponse::Ok().content_type("text/html").body(
                template_manager.render("post.html", &ctx).unwrap()
            )
        },
        Err(_err)=> HttpResponse::Ok().body("Error al recibir la data")
    }
}

#[post("/posts")]
async fn create_post(pool: web::Data<DbPool>, item: web::Json<NewPostHandler>)-> impl Responder {
    let mut conn = pool.get().expect("DB not found");

    match web::block(move || {Post::create_post(&mut conn, &item)}).await {
        Ok(data)=> {
            HttpResponse::Ok().body(format!("{:?}", data))
        },
        Err(_err)=> HttpResponse::Ok().body("Error al recibir la data")
    }
}

#[actix_web::main]
async fn main()-> std::io::Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("db url variable no encontrada");

    let connection = ConnectionManager::<PgConnection>::new(&db_url);
    let pool = Pool::builder().build(connection).expect("error de conexión a la base de datos");

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        
        App::new()
        .service(index_posts)
        .service(create_post)
        .service(show_post)
        //.service(tera_test)
        .app_data(Data::new(pool.clone()))
        .app_data(Data::new(tera.clone()))
    }).bind(("0.0.0.0", 9000)).unwrap().run().await
}
