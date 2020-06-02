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
use std::error::Error;

use anyhow::{Result, anyhow};

#[tokio::main]
async fn main() {
    // configure the spotify client
    let spotify = get_spotify_client().await.unwrap();
    // get the rss feed
    let channel = Channel::from_url("https://www.dnalounge.com/webcast/mixtapes/mixtapes.rss").unwrap();

    let me = spotify.me().await.unwrap();
    let user_id = me.id;

    for item in channel.items() {
        let title = item.title().unwrap();
        let text = item.description().unwrap();

        eprintln!("Title: {}", title);

        // try to get the playlist
        // this will create the playlist if it doesn't already exist
        // then if the playlist is public, we will skip it
        // otherwise, if it's private, then we have to ensure that tracks are added
        let playlist = match get_playlist_for(&spotify, &user_id, &title).await {
            Ok(playlist) => playlist,
            Err(error) => {
                eprintln!("Playlist exists and is public, so yeah: {}", error);
                continue
            },
        };

        let queries  = parse_tracks(&text);
        // eprintln!("queries: {:?}", queries);

        let mut track_uris: Vec<String> = vec![];

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
                        track_uris.push(result_uri.to_owned());
                    },
                    None => {
                        eprintln!("Found nothin'");
                    }
                }
            }
        }

        eprintln!("adding tracks to playlist...");
        // now let's add tracks to playlist
        spotify.user_playlist_add_tracks(&user_id, &playlist, &track_uris[..track_uris.len()], None).await.unwrap();
        // make it public
        spotify.user_playlist_change_detail(&user_id, &playlist, None, Some(true), None, None).await.unwrap();
    }
}

async fn get_playlist_for(spotify: &Spotify, user_id: &str, title: &str) -> Result<String> {
    let playlists = spotify.current_user_playlists(50, 0).await.unwrap();

    for p in playlists.items {
        if p.name == title {
            if Some(true) == p.public {
                return Err(anyhow!("Playlist is plublic so already created."));
            } else {
                return Ok(p.id.to_owned());
            }
        }
    }

    eprintln!("Creating playlist...");

    let playlist = spotify.user_playlist_create(user_id, title, Some(false), None).await.unwrap();

    Ok(playlist.id.to_owned())
}

async fn get_spotify_client() -> Result<Spotify> {
    let mut oauth = SpotifyOAuth::default()
        .client_id("4abb24ee71384d518e0bb9e3d54b8241")
        .client_secret("XXXXXXXXXXXXXXXXXXXXXXXXX") // this has been reset and has to be populated
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
