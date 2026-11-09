use bytes::Bytes;
use freya::query::{Query, QueryCapability, UseQuery, use_query};
use oneclient_core::LauncherError;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalSkinQuery;

impl QueryCapability for LocalSkinQuery {
    type Keys = Uuid;
    type Ok = Option<(Bytes, bool)>;
    type Err = LauncherError;
    async fn run(&self, account: &Uuid) -> Result<Self::Ok, Self::Err> {
        Ok(oneclient_core::local_skin::load(*account)
            .await?
            .map(|(bytes, slim)| (bytes.into(), slim)))
    }
}

pub fn use_local_skin(account: Option<Uuid>) -> UseQuery<LocalSkinQuery> {
    use_query(Query::new(account, LocalSkinQuery))
}
