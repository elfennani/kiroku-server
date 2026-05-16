use crate::domain::models::User;
use crate::prelude::*;

pub trait SessionRepository: Sync + Send {
    fn get_access_token(&self) -> Result<Option<String>>;
    fn save_access_token(&self, access_token: String) -> Result<()>;
}

pub trait UserRepository: Sync + Send{
    fn get_user_by_id(&self, id: i32) -> Result<Option<User>>;
    fn get_viewer_user(&self) -> Result<Option<User>>;
    fn save_user(&self, user: &User, is_viewer: bool) -> Result<()>;
}
