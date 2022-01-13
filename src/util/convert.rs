use async_trait::async_trait;

#[async_trait]
pub(crate) trait TryFromValue<T>: Sized {
    type Error;

    async fn try_from_value(v: T) -> Result<Self, Self::Error>;
}

#[async_trait]
pub(crate) trait TryValueInto<T>: Sized {
    type Error;

    async fn try_value_into(self) -> Result<T, Self::Error>;
}

#[async_trait]
impl<T, U> TryValueInto<U> for T
where
    T: Send,
    U: TryFromValue<T>,
{
    type Error = U::Error;

    async fn try_value_into(self) -> Result<U, U::Error> {
        U::try_from_value(self).await
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
pub(crate) trait FromStrRef: Sized {
    type Error;

    async fn from_str_ref(s: &str) -> Result<Self, Self::Error>;
}
