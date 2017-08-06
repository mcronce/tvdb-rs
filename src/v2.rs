use std::io::Read;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use url;
use reqwest;
use serde_json;

use super::error::{TvdbError, TvdbResult};
use super::api::RequestClient;
use super::api::DefaultHttpClient;

/// Main interface
#[derive(Debug, Clone)]
pub struct Tvdb<'a> {
    /// Your API key from TheTVDB.com
    pub key: String,
    http_client: Option<&'a RequestClient>,
    jwt_token: RefCell<Option<String>>,
    default_client: Rc<RequestClient>,

}


/// https://api.thetvdb.com/swagger#/Authentication
#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: String,
}

/// List of `SeriesSearchData`, returned from a search
#[derive(Deserialize, Debug)]
pub struct SeriesSearchResult {
    pub data: Vec<SeriesSearchData>,
}

/// Info for a single series, as returned from search query
#[derive(Deserialize, Debug)]
pub struct SeriesSearchData {
    pub aliases: Option<Vec<String>>,
    pub banner: Option<String>,
    pub firstAired: Option<String>,
    pub id: Option<i64>,
    pub network: Option<String>,
    pub overview: Option<String>,
    pub seriesName: Option<String>,
    pub status: Option<String>,
}


impl<'a> Tvdb<'a> {
    /// Initalise API with the given API key. A key can be acquired via
    /// the [API Key Registration page](http://thetvdb.com/?tab=apiregister)
    pub fn new<S>(key: S) -> Tvdb<'a>
        where S: Into<String>,
    {
        Tvdb {
            key: key.into(),
            http_client: None,
            jwt_token: RefCell::new(None),
            default_client: Rc::new(DefaultHttpClient {}),
        }
    }

    /// Set the JWT session token
    fn set_token(&self, token: String) {
        let mut j = self.jwt_token.borrow_mut();
        *j = Some(token);
    }
    fn get_token(&self) -> Option<String> {
        let j = self.jwt_token.borrow();
        match *j {
            None => None,
            Some(ref t) => Some(format!("{}", *t)),
        }
    }

    /// Authenticate with TheTVDB, storing the JWT token internally for use by
    /// other methods.
    pub fn login(&self) -> TvdbResult<bool> {
        let mut map = HashMap::new();
        map.insert("apikey", &self.key);

        let c = reqwest::Client::new().unwrap();
        let mut resp = c.post("https://api.thetvdb.com/login")
            .json(&map)
            .send()
            .map_err(|x| {
                TvdbError::CommunicationError { reason: format!("{}", x) }
            })?;
        let mut result = String::new();
        resp.read_to_string(&mut result).map_err(|x| {
            TvdbError::CommunicationError { reason: format!("Error reading response: {}", x) }
        })?;

        let deserialized: serde_json::Value = serde_json::from_str(&result).unwrap();
        let tok: String = deserialized["token"].as_str().unwrap().into();
        self.set_token(tok);

        Ok(true)
    }

    /// Sets a custom client (implementation of `RequestClient`) used to
    /// perform HTTP requests
    pub fn set_http_client(&mut self, client: &'a RequestClient) {
        self.http_client = Some::<&'a RequestClient>(client);
    }

    pub fn search(&self, name: &str) -> TvdbResult<SeriesSearchResult> {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("name", name);

        let data = self.get_search_data(&map)?;

        // Parse result
        let result: SeriesSearchResult = serde_json::from_str(&data).unwrap();

        Ok(result)
    }

    pub fn search_imdb(&self, imdb_id: &str) -> TvdbResult<SeriesSearchResult> {
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("imdbId", imdb_id);

        let data = self.get_search_data(&params)?;

        // Parse result
        let result: SeriesSearchResult = serde_json::from_str(&data).unwrap();

        Ok(result)
    }

    fn get_search_data(&self, params: &HashMap<&str, &str>) -> TvdbResult<String> {
        let search_url = "https://api.thetvdb.com/search/series";
        let url: String = url::Url::parse_with_params(search_url, params)
            .unwrap()
            .as_str()
            .into();
        let dc = self.default_client.as_ref();
        let c = self.http_client.unwrap_or(dc);
        // Query URL
        let data = c.get_url(&url, self.get_token())?;

        return Ok(data);
    }
}
