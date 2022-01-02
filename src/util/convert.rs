use async_trait::async_trait;

#[async_trait]
pub(crate) trait AsyncTryInto<T>: Sized {
    type Error;

    async fn async_try_into(self) -> Result<T, Self::Error>;
}

#[async_trait]
pub(crate) trait AsyncTryFrom<T>: Sized {
    type Error;

    async fn async_try_from(v: T) -> Result<Self, Self::Error>;
}

#[async_trait]
impl<T, U> AsyncTryInto<U> for T
where
    T: Send,
    U: AsyncTryFrom<T>,
{
    type Error = U::Error;

    async fn async_try_into(self) -> Result<U, U::Error> {
        U::async_try_from(self).await
    }
}

#[async_trait]
pub(crate) trait TryFromRef<T>: Sized {
    type Error;

    async fn try_from_ref(v: &T) -> Result<Self, Self::Error>;
}

#[async_trait]
pub(crate) trait TryRefInto<T>: Sized {
    type Error;

    async fn try_ref_into(&self) -> Result<T, Self::Error>;
}

#[async_trait]
impl<T, U> TryRefInto<U> for T
where
    T: Send + Sync,
    U: TryFromRef<T>,
{
    type Error = U::Error;

    async fn try_ref_into(&self) -> Result<U, U::Error> {
        U::try_from_ref(self).await
    }
}

#[async_trait]
pub(crate) trait AsyncFromStr: Sized {
    type Error;

    async fn async_from_str(s: &str) -> Result<Self, Self::Error>;
}
