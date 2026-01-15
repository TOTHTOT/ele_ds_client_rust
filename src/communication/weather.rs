use crate::communication::http_client::EleDsHttpClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// get 天气数据返回的内容
pub struct WeatherResponse {
    pub code: String,
    pub update_time: String,
    pub fx_link: String,
    pub daily: Vec<DailyWeather>,
    pub refer: Refer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// 每日天气
pub struct DailyWeather {
    pub fx_date: String,
    pub sunrise: String,
    pub sunset: String,
    pub moonrise: String,
    pub moonset: String,
    pub moon_phase: String,
    pub moon_phase_icon: String,
    pub temp_max: String,
    pub temp_min: String,
    pub icon_day: String,
    pub text_day: String,
    pub icon_night: String,
    pub text_night: String,
    pub wind360_day: String,
    pub wind_dir_day: String,
    pub wind_scale_day: String,
    pub wind_speed_day: String,
    pub wind360_night: String,
    pub wind_dir_night: String,
    pub wind_scale_night: String,
    pub wind_speed_night: String,
    pub humidity: String,
    pub precip: String,
    pub pressure: String,
    pub vis: String,
    pub cloud: String,
    pub uv_index: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refer {
    pub sources: Vec<String>,
    pub license: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 获取城市id用的
pub struct GeoResponse {
    pub code: String,
    pub location: Vec<Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub name: String,
    pub id: String,
    pub lat: String,  // 纬度
    pub lon: String,  // 经度
    pub adm2: String, // 上级城市
    pub adm1: String, // 省份
    pub country: String,
    pub tz: String, // 时区
}
pub struct Weather {
    pub city: String,
    key: String,
}
impl Weather {
    pub fn new(city: &str, key: &str) -> Self {
        Self {
            city: city.to_string(),
            key: key.to_string(),
        }
    }

    fn get_city_id_from_name(&self, client: &mut EleDsHttpClient) -> anyhow::Result<String> {
        let url = format!(
            "https://geoapi.qweather.com/v2/city/lookup?key={}&location={}",
            self.key, self.city
        );

        let json_str = client.get_msg(&url)?;

        let geo_data: GeoResponse = serde_json::from_str(&json_str)?;

        if geo_data.code == "200" && !geo_data.location.is_empty() {
            let city_id = geo_data.location[0].id.clone();
            log::info!("city: {}, id: {}", self.city, city_id);
            Ok(city_id)
        } else {
            anyhow::bail!(
                "failed get city id, error code: {}, response: {json_str}",
                geo_data.code,
            );
        }
    }
    pub fn get_weather_hefeng(&self) -> anyhow::Result<WeatherResponse> {
        let mut client = EleDsHttpClient::new("https://devapi.qweather.com")?;
        let city_id = self.get_city_id_from_name(&mut client)?;
        let url = format!(
            "https://devapi.qweather.com/v7/weather/7d?key={}&location={city_id}&lang=en",
            self.key
        );
        let json_str = client.get_msg(&url)?;
        let weather: WeatherResponse = serde_json::from_str(&json_str)?;
        if weather.code != "200" {
            anyhow::bail!("QWeather API error code: {}", weather.code);
        }
        Ok(weather)
    }
}

#[test]
pub fn test() -> anyhow::Result<()> {
    let weather = Weather::new("fuzhou", "e7d95a70480a4d6c9140378d9d100d42");
    log::info!(
        "{} weather is: \n{:?}",
        weather.city,
        weather.get_weather_hefeng()?
    );
    Ok(())
}
