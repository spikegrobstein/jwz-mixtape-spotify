extern crate rspotify;

use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use rspotify::senum::Country;
use rspotify::util::get_token;
use rspotify::oauth2::TokenInfo;

use std::fs::File;
use std::io::prelude::*;

use regex::Regex;
use rss::Channel;

use anyhow::{Result, anyhow};

#[tokio::main]
async fn main() {
    // configure the spotify client
    let spotify = get_spotify_client().await.unwrap();
    // get the rss feed
    let channel = Channel::from_url("https://www.dnalounge.com/webcast/mixtapes/mixtapes.rss").unwrap();

    let item = channel.items().get(0).unwrap();
    let title = item.title().unwrap();
    let text = item.description().unwrap();

    eprintln!("Title: {}", title);

    eprintln!("parsin: {}", text);
    let queries  = parse_tracks(&text);
    eprintln!("queries: {:?}", queries);

    for query in queries {
        eprintln!("  Searching: {}", query);
        let result = spotify
            .search_track(&query, 10, 0, Some(Country::UnitedStates))
            .await;

        if let Ok(result) = result {

            let tracks = result.tracks;
            match tracks.items.get(0) {
                Some(track) => {
                    let result_uri = track.id.as_ref().unwrap();
                    println!("  search result:{:?}", result_uri);
                },
                None => {
                    eprintln!("Found nothin'");
                }
            }
        }
    }

}

async fn get_spotify_client() -> Result<Spotify> {
    // TODO change these before we opensource
    let mut oauth = SpotifyOAuth::default()
        .client_id("4abb24ee71384d518e0bb9e3d54b8241")
        .client_secret("0d5269c16cec49c3b441bfc227fba85c")
        .redirect_uri("http://localhost:8888/callback")
        .scope("user-read-private")
        .scope("playlist-modify-public")
        .scope("playlist-modify-private")
        .build();

    match get_token(&mut oauth).await {
        Some(token_info) => {
            let client_credential = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();

            Ok(Spotify::default()
                .client_credentials_manager(client_credential)
                .build())
        },
        None => {
            eprintln!("error.");
            std::process::exit(1);
        }
    }
}

fn parse_tracks(string: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^(\d+)\s+(.+) -- (.+)\s+\(\d{4}\)$").unwrap();
    let mut result = vec![];

    for cap in re.captures_iter(string).into_iter() {
        result.push(format!("{} {}", &cap[2], &cap[3]));
    }

    result
}

// either read data from file or oauthify it
// async fn load_token() -> TokenInfo {
    // match read_token_from_file() {

    // }
// }

// fn read_token_from_file() -> Result<TokenInfo> {
    // let mut file = File::open("token.info")?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;



// }
