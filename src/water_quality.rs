use crate::{redis_utils, AppState, Spot, QUALITY_PATH};

use anyhow::anyhow;
use std::sync::Arc;

#[derive(serde::Serialize)]
pub struct WaterQuality {
    pub water_quality: String,
    pub water_quality_text: String,
}

impl WaterQuality {
    pub async fn try_get_string(spot: Arc<Spot>, state: Arc<AppState>) -> anyhow::Result<String> {
        if let Some(data) =
            redis_utils::get(&format!("water-quality-{}", spot.name), &state.redis_pool).await
        {
            tracing::info!("redis cache hit!");
            return Ok(data);
        }

        let data = Self::try_get(spot.clone(), state.quality_url).await?;
        let data = serde_json::to_string(&data)?;

        redis_utils::set(
            &format!("water-quality-{}", spot.name),
            &data,
            &state.redis_pool,
        )
        .await?;

        Ok(data)
    }

    pub async fn try_get(spot: Arc<Spot>, quality_url: &'static str) -> anyhow::Result<Self> {
        let (water_quality, water_quality_text) =
            Self::get_quality_data(spot.quality_query, spot.status_query, quality_url).await?;

        Ok(Self {
            water_quality,
            water_quality_text,
        })
    }

    async fn get_quality_data(
        quality_query: &str,
        status_query: &str,
        quality_url: &'static str,
    ) -> anyhow::Result<(String, String)> {
        let status = reqwest::get(format!("{quality_url}{QUALITY_PATH}{status_query}"))
            .await?
            .json::<serde_json::Value>()
            .await?;

        let response = reqwest::get(format!("{quality_url}{QUALITY_PATH}{quality_query}"))
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok((
            status
                .get("features")
                .ok_or(anyhow!("no features found."))?
                .as_array()
                .ok_or(anyhow!("features is not an array."))?
                .first()
                .ok_or(anyhow!("empty array of features."))?
                .get("attributes")
                .ok_or(anyhow!("no attributes found."))?
                .get("MAP_STATUS")
                .ok_or(anyhow!("no map status found."))?
                .as_str()
                .ok_or(anyhow!("map status not a string."))?
                .to_string(),
            response
                .get("features")
                .ok_or(anyhow!("no features found."))?
                .as_array()
                .ok_or(anyhow!("features is not an array."))?
                .first()
                .ok_or(anyhow!("empty array of features."))?
                .get("attributes")
                .ok_or(anyhow!("no attributes found."))?
                .get("STATUS")
                .ok_or(anyhow!("no status found."))?
                .as_str()
                .ok_or(anyhow!("status not a string."))?
                .to_string(),
        ))
    }
}
