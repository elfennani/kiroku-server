use crate::infrastructure::anilist::schema;

#[derive(cynic::QueryFragment)]
struct UserAvatar {
    large: Option<String>,
}

#[derive(cynic::QueryFragment)]
pub struct User {
    pub id: i32,
    name: String,
    avatar: Option<UserAvatar>,
    banner_image: Option<String>,
    about: Option<String>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct ViewerQuery {
    #[cynic(rename = "Viewer")]
    pub viewer: Option<User>,
}

// impl From<User> for crate::domain::models::User {
//     fn from(user: User) -> Self {
//         crate::domain::models::User {
//             id: user.id,
//             name: user.name,
//             avatar_url: user.avatar.and_then(|avatar| avatar.large),
//             banner_url: user.banner_image,
//             description: user.about,
//         }
//     }
// }
