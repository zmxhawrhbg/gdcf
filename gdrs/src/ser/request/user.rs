use gdcf::api::request::{user::UserRequest, BaseRequest};
use super::BaseRequestRem;

#[derive(Serialize)]
#[serde(remote = "UserRequest")]
pub struct UserRequestRem {
    #[serde(flatten, with = "BaseRequestRem")]
    base: BaseRequest,

    #[serde(rename = "targetAccountID")]
    user: u64,
}