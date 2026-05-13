mod graphql_request;
mod kodik_api_response;
mod shiki_api_animes;

pub use graphql_request::{
    Anime, BasicAnime, FetchAnimesResponse, FetchAnimesVars, GraphQLRequest, Related, Relation, RelationKind,
    UserRate as GraphQLUserRate,
};
pub use kodik_api_response::*;
pub use shiki_api_animes::*;
