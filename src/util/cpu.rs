use std::time::Duration;

use futures::stream::TryStreamExt as _;
use tokio::sync::OnceCell;
use tokio::time::sleep;

use crate::Result;

pub(crate) async fn ids() -> &'static Vec<u64> {
    static IDS: OnceCell<Vec<u64>> = OnceCell::const_new();
    async fn ids() -> Vec<u64> {
        let mut v: Vec<_> = syx::cpu::ids().try_collect().await.unwrap_or_default();
        v.sort_unstable();
        v
    }
    IDS.get_or_init(ids).await
}

pub(crate) async fn wait_for_onoff() {
    let d = Duration::from_millis(300);
    sleep(d).await
}

pub(crate) async fn wait_for_policy() {
    let d = Duration::from_millis(100);
    sleep(d).await
}

pub(crate) async fn set_online(cpu_ids: Vec<u64>) -> Result<Vec<u64>> {
    let mut onlined = vec![];
    if !cpu_ids.is_empty() {
        let offline: Vec<_> = syx::cpu::offline_ids().try_collect().await?;
        for cpu_id in cpu_ids {
            if offline.contains(&cpu_id) {
                syx::cpu::set_online(cpu_id, true).await?;
                onlined.push(cpu_id);
            }
        }
        if !onlined.is_empty() {
            wait_for_onoff().await;
        }
    }
    Ok(onlined)
}

pub(crate) async fn set_offline(cpu_ids: Vec<u64>) -> Result<Vec<u64>> {
    let mut offlined = vec![];
    if !cpu_ids.is_empty() {
        let online: Vec<_> = syx::cpu::online_ids().try_collect().await?;
        for cpu_id in cpu_ids {
            if online.contains(&cpu_id) {
                syx::cpu::set_online(cpu_id, false).await?;
                offlined.push(cpu_id);
            }
        }
        if !offlined.is_empty() {
            wait_for_onoff().await;
        }
    }
    Ok(offlined)
}
