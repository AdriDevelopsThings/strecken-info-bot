use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

use chrono::{Datelike, Utc};
use chrono_tz::Europe;
use log::error;
use models::Infrastructure;
use tokio::{sync::RwLock, time::interval};

use crate::normalize_spaces;

mod models;

const TRASSENFINDER_INFRASTRUCTURES: &str = "https://trassenfinder.de/api/web/infrastrukturen";
const TRASSENFINDER_RELOAD_DURATION: Duration = Duration::from_secs(60 * 60 * 24 * 2);

#[derive(Clone)]
pub struct TrassenfinderApi {
    infrastructure_ids: Arc<RwLock<Vec<i32>>>,
    pub stations: Arc<RwLock<HashMap<String, (f64, f64)>>>,
}

impl TrassenfinderApi {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let s = Self {
            infrastructure_ids: Arc::new(RwLock::new(Vec::new())),
            stations: Arc::new(RwLock::new(HashMap::new())),
        };
        s.reload_infrastructure_ids().await?;
        s.reload_stations().await?;

        Ok(s)
    }

    pub async fn start_reloading(&self) {
        let s = self.clone();
        tokio::spawn(async move {
            let mut i = interval(TRASSENFINDER_RELOAD_DURATION);
            loop {
                i.tick().await;

                if let Err(e) = s.reload_infrastructure_ids().await {
                    error!("Error while reloading trassenfinder infrastructure id: {e:?}");
                }

                if let Err(e) = s.reload_stations().await {
                    error!("Error while reloading trassenfinder stations: {e:?}");
                }
            }
        });
    }

    async fn reload_stations(&self) -> Result<(), Box<dyn Error>> {
        let infrastructure_ids = &*self.infrastructure_ids.read().await.clone();
        let mut stations: HashMap<String, (f64, f64)> = HashMap::new();
        for infrastructure_id in infrastructure_ids {
            let response = reqwest::Client::new()
                .get(format!(
                    "{TRASSENFINDER_INFRASTRUCTURES}/{infrastructure_id}"
                ))
                .send()
                .await?;
            response.error_for_status_ref()?;

            let response: Infrastructure = response.json().await?;
            debug_assert_eq!(response.id, *infrastructure_id);
            let data = response
                .data
                .ok_or::<String>("Data not sent with infrastructure response".into())?;

            for station in data.stations {
                let coords = match station.coordinates {
                    Some(c) => c,
                    None => continue,
                };
                stations.insert(normalize_spaces(&station.ds100), (coords.lon, coords.lat));
            }
        }

        let mut stations_write = self.stations.write().await;
        *stations_write = stations;

        Ok(())
    }

    async fn reload_infrastructure_ids(&self) -> Result<(), Box<dyn Error>> {
        let response = reqwest::Client::new()
            .get(TRASSENFINDER_INFRASTRUCTURES)
            .send()
            .await?;
        response.error_for_status_ref()?;

        let response: Vec<Infrastructure> = response.json().await?;
        let current_year = Utc::now().with_timezone(&Europe::Berlin).year();
        let current_infrastructures = response
            .into_iter()
            .filter(|i| i.year == current_year)
            .collect::<Vec<_>>();

        let mut infrastructure_ids_write = self.infrastructure_ids.write().await;
        *infrastructure_ids_write = current_infrastructures.iter().map(|i| i.id).collect();
        Ok(())
    }
}
