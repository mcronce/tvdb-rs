extern crate xmltree;
extern crate hyper;


use hyper::Client;
use hyper::header::Connection;
use std::io::Read;

use hyper::Url;


/// Turns "123" into 123
pub fn intify(instr: &str) -> u32{
    // TODO: Better error handling
    instr.to_owned().parse::<u32>().unwrap()
}


/// Errors in contacting TheTVDB
#[derive(Debug)]
pub enum TvdbError {
    SeriesNotFound,
    CommunicationError{reason: String},
    DataError{reason: String},
    Cancelled,
}


#[derive(Debug,Clone)]
pub struct SeriesSearchResult{
    pub seriesid: u32, // seriesid is preferred over id according to TVDB wiki
    pub seriesname: String,
    pub language: String,
    pub overview: Option<String>,
    pub banner: Option<String>,
    pub imdb_id: Option<String>,
    //pub firstaired: Date,
    pub network: Option<String>,
    pub zap2it_id: Option<String>,
}

pub struct Tvdb{
    key: String,
}

impl Tvdb{
    fn new(key: String) -> Tvdb{
        Tvdb{key: key}
    }

    /// Searches for a given series name.
    fn search(&self, seriesname: String, lang: String) -> Result<Vec<SeriesSearchResult>, TvdbError>{
        let client = Client::new();

        let formatted_url = format!("http://thetvdb.com/api/GetSeries.php?seriesname={}", seriesname);
        let url = Url::parse(&formatted_url).ok().expect("invalid URL");
        println!("Getting {}", url);

        let res = client.get(url)
            .header(Connection::close())
            .send();

        let mut res = match res {
            Err(e) => return Err(TvdbError::CommunicationError{reason: "Error contacting TVDB".to_owned()}), // FIXME: http://stackoverflow.com/questions/28911833/error-handling-best-practices
            Ok(r) => r
        };

        // Read the Response.
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        // Parse XML
        let tree = xmltree::Element::parse(body.as_bytes());

        // Convert XML into structs
        let mut results : Vec<SeriesSearchResult> = vec![];

        for child in tree.children.iter(){

            fn get_text(child: &xmltree::Element, x: &str) -> Option<String>{
                child.get_child(x).and_then(|id_child| id_child.text.clone())
            }

            let r = SeriesSearchResult{
                seriesid:   intify(&get_text(child, "seriesid").expect("Search result XML missing 'seriesid' element")),
                seriesname: get_text(child, "SeriesName").expect("Search result XML Missing 'SeriesName' element"),
                language:   get_text(child, "language").expect("Search result XML missing 'language' element"),
                overview:   get_text(child, "Overview"),
                banner:     get_text(child, "banner"),
                imdb_id:    get_text(child, "IMDB_ID"),
                //firstaired: Date,
                network:    get_text(child, "Network"),
                zap2it_id:  get_text(child, "zap2it_id"),
            };

            results.push(r);
        }

        if results.is_empty(){
            return Err(TvdbError::SeriesNotFound);
        }

        return Ok(results);
    }
}

struct EpisodeInfo{
    pub episodename: String,
}

impl SeriesSearchResult{
    fn episode(&self, season: u32, episode: u32) -> EpisodeInfo{
        println!("{} s{}e{}", self.seriesname, season, episode);

        EpisodeInfo{episodename: "My First Day".to_owned()} // FIXME
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn search() {
        let api = Tvdb::new("APIKEY".to_owned());
        let sr = api.search("scrubs".to_owned(), "en".to_owned());
        assert!(sr.ok().unwrap()[0].seriesname == "Scrubs");
    }

    #[test]
    fn nonexist() {
        let api = Tvdb::new("APIKEY".to_owned());
        let sr = api.search("ladlkgdklfgsdfglk".to_owned(), "en".to_owned());
        assert!(sr.is_err());
    }

    #[test]
    fn epinfo_default(){
        let api = Tvdb::new("APIKEY".to_owned());
        let sr = api.search("scrubs".to_owned(), "en".to_owned()).ok().unwrap();
        let ep = sr[0].episode(1, 2);
        assert!(ep.episodename == "My First Day");
    }

    #[test]
    fn epinfo_dvd(){
        let api = Tvdb::new("APIKEY".to_owned());
        let sr = api.search("scrubs".to_owned(), "en".to_owned()).ok().unwrap();
        let ep = sr[0].episode(1, 2);
        assert!(ep.episodename == "My First Day");
    }

}
