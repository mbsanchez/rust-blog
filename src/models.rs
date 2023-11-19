use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub body: String
}

#[derive(Queryable, Debug, Deserialize, Serialize)]
pub struct PostThumbnail {
    pub title: String,
    pub body: String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewPostHandler {
    pub title: String,
    pub body: String
}

use diesel::prelude::*;
use super::schema::posts;
use super::schema::posts::dsl::*;

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub slug: &'a str,
}

impl Post {
    pub fn slugify(ttle: &String) -> String {
        return ttle.replace(" ", "-").to_lowercase();
    }

    pub fn create_post<'a>(conn: &mut PgConnection, post: &NewPostHandler) -> Result<Post, diesel::result::Error> {
        let slg = Post::slugify(&post.title.clone());

        let new_post = NewPost {
            title: &post.title,
            slug: &slg,
            body: &post.body
        };

        return diesel::insert_into(posts).values(new_post).get_result::<Post>(conn);
    }
}