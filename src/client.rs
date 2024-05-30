use reqwest;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_yaml;

use std::io::Cursor;

#[derive(Debug)]
pub struct Client {
    base_url: String,
    http_client: reqwest::Client,
    pub access_token: String,
    user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(rename = "Username")]
    username: String,
    #[serde(rename = "Pw")]
    password: String,
}

impl Client {
    pub async fn new(base_url: &str) -> Self {
        let f = std::fs::File::open("config.yaml").unwrap();
        let d: Value = serde_yaml::from_reader(f).unwrap();

        let http_client = reqwest::Client::new();
        let _credentials = {
            // let username = std::env::var("").ok();
            // let password = std::env::var("").ok();
            let username = d["username"].as_str();
            let password = d["password"].as_str();
            match (username, password) {
                (Some(username), Some(password)) => Some(Credentials {
                    username: username.to_string(),
                    password: password.to_string(),
                }),
                _ => None,
            }
        };

        // println!("{}", format!("{}/Users/authenticatebyname", d["host"]).as_str());
        // without the ""
        let url: String =
            String::new() + &d["server"].as_str().unwrap() + "/Users/authenticatebyname";
        let response = http_client
            .post(url)
            .header("Content-Type", "text/json")
            .header("x-emby-authorization", "MediaBrowser Client=\"jellyfin-tui\", Device=\"jellyfin-tui\", DeviceId=\"None\", Version=\"10.4.3\"")
            // .json(&Credentials {
            //     username: "".to_string(),
            //     password: "".to_string(),
            // })
            .json(&serde_json::json!({
                "Username": d["username"].as_str().unwrap(),
                "Pw": d["password"].as_str().unwrap()
            }))
            .send()
            .await;

        // check status without moving
        let status = response.as_ref().unwrap().status();
        if !status.is_success() {
            println!("Error authenticating. Status: {}", status);
            return Self {
                base_url: base_url.to_string(),
                http_client,
                access_token: "".to_string(),
                user_id: "".to_string(),
            };
        }

        // get response data
        let response: Value = response.unwrap().json().await.unwrap();
        // get AccessToken
        let access_token = response["AccessToken"].as_str().unwrap();
        // println!("Access Token: {}", access_token);

        // get user id (User.Id)
        let user_id = response["User"]["Id"].as_str().unwrap();
        // println!("User Id: {}", user_id);

        // println!("{:#?}", response);
        Self {
            base_url: base_url.to_string(),
            http_client,
            access_token: access_token.to_string(),
            user_id: user_id.to_string(),
        }
    }

    /// Produces a list of artists, called by the main function before initializing the app
    pub async fn artists(&self) -> Result<Vec<Artist>, reqwest::Error> {
        // let url = format!("{}/Users/{}/Artists", self.base_url, self.user_id);
        let url = format!("{}/Artists", self.base_url);
        println!("url: {}", url);

        // to send some credentials we can use the basic_auth method
        // let response = self.http_client.get(url).basic_auth(&self.credentials.username, Some(&self.credentials.password)).send().await;
        let s = format!("MediaBrowser Client=\"jellyfin-tui\", Device=\"jellyfin-tui\", DeviceId=\"None\", Version=\"10.4.3\" Token=\"{}\"", self.access_token);
        println!("s: {}", s);
        let response: Result<reqwest::Response, reqwest::Error> = self.http_client
            .get(url)
            .header("X-MediaBrowser-Token", self.access_token.to_string())
            .header("x-emby-authorization", "MediaBrowser Client=\"jellyfin-tui\", Device=\"jellyfin-tui\", DeviceId=\"None\", Version=\"10.4.3\"")
            .header("Content-Type", "text/json")
            .query(&[
                ("SortBy", "SortName"),
                ("SortOrder", "Ascending"), 
                ("Recursive", "true"), 
                ("Fields", "SortName"), 
                ("ImageTypeLimit", "-1")
            ])
            .query(&[("StartIndex", "0")])
            .send()
            .await;

        // check status without moving
        let status = response.as_ref().unwrap().status();

        // check if response is ok
        if !response.as_ref().unwrap().status().is_success() {
            println!("Error getting artists. Status: {}", status);
            return Ok(vec![]);
        }

        // deseralize using our types
        let artists: Artists = response.unwrap().json().await.unwrap();

        Ok(artists.items)
    }

    /// Produces a list of songs by an artist sorted by album and index
    pub async fn discography(&self, id: &str) -> Result<Discography, reqwest::Error> {
        let url = format!("{}/Users/{}/Items", self.base_url, self.user_id);

        let response = self.http_client
            .get(url)
            .header("X-MediaBrowser-Token", self.access_token.to_string())
            .header("x-emby-authorization", "MediaBrowser Client=\"jellyfin-tui\", Device=\"jellyfin-tui\", DeviceId=\"None\", Version=\"10.4.3\"")
            .header("Content-Type", "text/json")
            .query(&[
                ("SortBy", "Album,IndexNumber"),
                ("SortOrder", "Ascending"),
                ("Recursive", "true"), 
                ("IncludeItemTypes", "Audio"),
                ("Fields", "Genres, DateCreated, MediaSources, ParentId"),
                ("StartIndex", "0"),
                ("ImageTypeLimit", "1"),
                ("ArtistIds", id)
            ])
            .query(&[("StartIndex", "0")])
            .send()
            .await;

        // check status without moving
        let status = response.as_ref().unwrap().status();

        // check if response is ok
        if !response.as_ref().unwrap().status().is_success() {
            println!("Error getting artists. Status: {}", status);
            return Ok(Discography { items: vec![] });
        }

        // artists is the json string of all artists

        // first arbitrary json
        // let artist: Value = response.unwrap().json().await.unwrap();
        // println!("{:#?}?", artist);
        let discog: Discography = response.unwrap().json().await.unwrap();
        // println!("{:#?}", discog);

        return Ok(discog);
    }

    pub async fn lyrics(&self, song_id: String) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("{}/Audio/{}/Lyrics", self.base_url, song_id);

        let response = self.http_client
            .get(url)
            .header("X-MediaBrowser-Token", self.access_token.to_string())
            .header("x-emby-authorization", "MediaBrowser Client=\"jellyfin-tui\", Device=\"jellyfin-tui\", DeviceId=\"None\", Version=\"10.4.3\"")
            .header("Content-Type", "application/json")
            .send()
            .await;

        // check status without moving
        // let status = response.as_ref().unwrap().status();

        // check if response is ok
        if !response.as_ref().unwrap().status().is_success() {
            // println!("Error getting artists. Status: {}", status);
            return Ok(vec![]);
        }

        // artists is the json string of all artists

        // first arbitrary json
        // let artist: Value = response.unwrap().json().await.unwrap();
        // println!("{:#?}?", artist);
        let lyrics: Lyrics = response.unwrap().json().await.unwrap();
        // turn into vector of strings
        let lyric = lyrics.lyrics.iter().map(|l| format!(" {}", l.text.clone())).collect();

        return Ok(lyric);
    }

    pub async fn metadata(&self, song_id: String) -> Result<MediaStream, reqwest::Error> {
        // https://jelly.danielhonus.com/Users/f9784d6dce9645d48e2b00a160a24015/Items/a276bbd638e22c54e2f765e289147734

        let url = format!("{}/Users/{}/Items/{}", self.base_url, self.user_id, song_id);

        let response = self.http_client
            .get(url)
            .header("X-MediaBrowser-Token", self.access_token.to_string())
            .header("x-emby-authorization", "MediaBrowser Client=\"jellyfin-tui\", Device=\"jellyfin-tui\", DeviceId=\"None\", Version=\"10.4.3\"")
            .header("Content-Type", "application/json")
            .send()
            .await;

        // check status without moving
        // let status = response.as_ref().unwrap().status();

        // check if response is ok
        let song: Value = response.unwrap().json().await.unwrap();
        let media_sources: Vec<MediaSource> = serde_json::from_value(song["MediaSources"].clone()).unwrap();
        
        for m in media_sources {
            for ms in m.media_streams {
                if ms.type_ == "Audio" {
                    return Ok(ms);
                }
            }
        }

        return Ok(MediaStream {
            codec: "".to_string(),
            bit_rate: 0,
            channels: 0,
            sample_rate: 0,
            type_: "".to_string(),
        });

        // artists is the json string of all artists

    }

    // https://jelly.danielhonus.com/Items/864b113ebeaa11294bc508f95b8a4b55/Images/Primary?fillHeight=64&fillWidth=64&quality=96&tag=be2a8642e97e2151ef0580fc72f3505a
    pub async fn download_cover_art(&self, album_id: String) -> Result<String, reqwest::Error> {
        let url = format!("{}/Items/{}/Images/Primary?fillHeight=512&fillWidth=512&quality=96&tag=be2a8642e97e2151ef0580fc72f3505a", self.base_url, album_id);
        let response = self.http_client
            .get(url)
            .header("X-MediaBrowser-Token", self.access_token.to_string())
            .header("x-emby-authorization", "MediaBrowser Client=\"jellyfin-tui\", Device=\"jellyfin-tui\", DeviceId=\"None\", Version=\"10.4.3\"")
            .header("Content-Type", "application/json")
            .send()
            .await;

        // check status without moving
        // let status = response.as_ref().unwrap().status();

        // check if response is ok
        // this literally sends us a png/jpeg/jpg/webp file. We will download it as cover.* and return the file name
        let mut response = response.unwrap();

        // we need to get the file extension
        let content_type = response.headers().get("Content-Type").unwrap().to_str().unwrap();
        let extension = match content_type {
            "image/png" => "png",
            "image/jpeg" => "jpeg",
            "image/jpg" => "jpg",
            "image/webp" => "webp",
            _ => "png",
        };

        std::fs::create_dir_all("covers").unwrap();

        let mut file = std::fs::File::create("covers/cover.".to_string() + extension).unwrap();
        let mut content =  Cursor::new(response.bytes().await.unwrap());
        std::io::copy(&mut content, &mut file).unwrap();

        Ok("cover.".to_string() + extension)
    }

    /// Produces URL of a song from its ID
    pub fn song_url_sync(&self, song_id: String) -> String {
        let url = format!("{}/Audio/{}/universal", self.base_url, song_id);
        let url = url + &format!("?UserId={}&Container=opus,webm|opus,mp3,aac,m4a|aac,m4b|aac,flac,webma,webm|webma,wav,ogg&TranscodingContainer=mp4&TranscodingProtocol=hls&AudioCodec=aac&api_key={}&StartTimeTicks=0&EnableRedirection=true&EnableRemoteMedia=false", self.user_id, self.access_token);
        url
    }
}

/// TYPES ///
///
/// All the jellyfin types will be defined here. These types will be used to interact with the jellyfin server.

/// ARTIST
/* {
  "Name": "Flam",
  "ServerId": "97a9003303d7461395074680d9046935",
  "Id": "a9b08901ce0884038ef2ab824e4783b5",
  "SortName": "flam",
  "ChannelId": null,
  "RunTimeTicks": 4505260770,
  "Type": "MusicArtist",
  "UserData": {
    "PlaybackPositionTicks": 0,
    "PlayCount": 0,
    "IsFavorite": false,
    "Played": false,
    "Key": "Artist-Musicbrainz-622c87fa-dc5e-45a3-9693-76933d4c6619"
  },
  "ImageTags": {},
  "BackdropImageTags": [],
  "ImageBlurHashes": {},
  "LocationType": "FileSystem",
  "MediaType": "Unknown"
} */
#[derive(Debug, Serialize, Deserialize)]
pub struct Artists {
    #[serde(rename = "Items")]
    items: Vec<Artist>,
    #[serde(rename = "StartIndex")]
    start_index: u64,
    #[serde(rename = "TotalRecordCount")]
    total_record_count: u64,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "SortName", default)]
    sort_name: String,
    #[serde(rename = "RunTimeTicks", default)]
    run_time_ticks: u64,
    #[serde(rename = "Type", default)]
    type_: String,
    #[serde(rename = "UserData")]
    user_data: UserData,
    #[serde(rename = "ImageTags", default)]
    image_tags: serde_json::Value,
    #[serde(rename = "ImageBlurHashes", default)]
    image_blur_hashes: serde_json::Value,
    #[serde(rename = "LocationType", default)]
    location_type: String,
    #[serde(rename = "MediaType", default)]
    media_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    #[serde(rename = "PlaybackPositionTicks")]
    playback_position_ticks: u64,
    #[serde(rename = "PlayCount")]
    play_count: u64,
    #[serde(rename = "IsFavorite")]
    is_favorite: bool,
    #[serde(rename = "Played")]
    played: bool,
    #[serde(rename = "Key")]
    key: String,
}

/// DISCOGRAPHY
///
/// The goal here is to mimic behavior of CMUS and get the whole discography of an artist.
/// We query jellyfin for all songs by an artist sorted by album and sort name.
/// Later we group them nicely by album.

/*
Object {
    "Album": String("Cardan [EP]"),
    "AlbumArtist": String("Agar Agar"),
    "AlbumArtists": Array [
        Object {
            "Id": String("c910b835045265897c9b1e30417937c8"),
            "Name": String("Agar Agar"),
        },
    ],
    "AlbumId": String("e66386bd52e9e13bcd53fefbe4dbfe80"),
    "AlbumPrimaryImageTag": String("728e73b82a9103d8d3bd46615f7c0786"),
    "ArtistItems": Array [
        Object {
            "Id": String("c910b835045265897c9b1e30417937c8"),
            "Name": String("Agar Agar"),
        },
    ],
    "Artists": Array [
        String("Agar Agar"),
    ],
    "BackdropImageTags": Array [],
    "ChannelId": Null,
    "DateCreated": String("2024-03-12T12:41:07.2583951Z"),
    "GenreItems": Array [
        Object {
            "Id": String("5897c94bfe512270b15fa7e6088e94d0"),
            "Name": String("Synthpop"),
        },
    ],
    "Genres": Array [
        String("Synthpop"),
    ],
    "HasLyrics": Bool(true),
    "Id": String("b26c12ffca74316396cb3d366a7f09f5"),
    "ImageBlurHashes": Object {
        "Backdrop": Object {
            "ea9ad04d014bd8317aa784ffb5676eac": String("W797hQ?bf7ofxuWU?b~qxut6t7M|-;xu%Mayj[xu-:j[xuRjRjt7"),
        },
        "Primary": Object {
            "222d9d1264b6994621fe99bb78047348": String("eQG*]WD+VD=|H?CmIoIotlM|Q,n%R*oeozVXjY$$n%WBMds.tRW=ni"),
            "728e73b82a9103d8d3bd46615f7c0786": String("eQG*]WD+VD=|H?CmIoIotlM|Q,n%R*oeozVXjY$$n%WBMds.tRW=ni"),
        },
    },
    "ImageTags": Object {
        "Primary": String("222d9d1264b6994621fe99bb78047348"),
    },
    "IndexNumber": Number(3),
    "IsFolder": Bool(false),
    "LocationType": String("FileSystem"),
    "MediaSources": Array [
        Object {
            "Bitrate": Number(321847),
            "Container": String("mp3"),
            "DefaultAudioStreamIndex": Number(0),
            "ETag": String("23dab11df466604c0b0cade1f8f814da"),
            "Formats": Array [],
            "GenPtsInput": Bool(false),
            "Id": String("b26c12ffca74316396cb3d366a7f09f5"),
            "IgnoreDts": Bool(false),
            "IgnoreIndex": Bool(false),
            "IsInfiniteStream": Bool(false),
            "IsRemote": Bool(false),
            "MediaAttachments": Array [],
            "MediaStreams": Array [
                Object {
                    "AudioSpatialFormat": String("None"),
                    "BitRate": Number(320000),
                    "ChannelLayout": String("stereo"),
                    "Channels": Number(2),
                    "Codec": String("mp3"),
                    "DisplayTitle": String("MP3 - Stereo"),
                    "Index": Number(0),
                    "IsAVC": Bool(false),
                    "IsDefault": Bool(false),
                    "IsExternal": Bool(false),
                    "IsForced": Bool(false),
                    "IsHearingImpaired": Bool(false),
                    "IsInterlaced": Bool(false),
                    "IsTextSubtitleStream": Bool(false),
                    "Level": Number(0),
                    "SampleRate": Number(44100),
                    "SupportsExternalStream": Bool(false),
                    "TimeBase": String("1/14112000"),
                    "Type": String("Audio"),
                    "VideoRange": String("Unknown"),
                    "VideoRangeType": String("Unknown"),
                },
                Object {
                    "AspectRatio": String("1:1"),
                    "AudioSpatialFormat": String("None"),
                    "BitDepth": Number(8),
                    "Codec": String("mjpeg"),
                    "ColorSpace": String("bt470bg"),
                    "Comment": String("Cover (front)"),
                    "Height": Number(500),
                    "Index": Number(1),
                    "IsAVC": Bool(false),
                    "IsAnamorphic": Bool(false),
                    "IsDefault": Bool(false),
                    "IsExternal": Bool(false),
                    "IsForced": Bool(false),
                    "IsHearingImpaired": Bool(false),
                    "IsInterlaced": Bool(false),
                    "IsTextSubtitleStream": Bool(false),
                    "Level": Number(-99),
                    "PixelFormat": String("yuvj420p"),
                    "Profile": String("Baseline"),
                    "RealFrameRate": Number(90000),
                    "RefFrames": Number(1),
                    "SupportsExternalStream": Bool(false),
                    "TimeBase": String("1/90000"),
                    "Type": String("EmbeddedImage"),
                    "VideoRange": String("Unknown"),
                    "VideoRangeType": String("Unknown"),
                    "Width": Number(500),
                },
                Object {
                    "AudioSpatialFormat": String("None"),
                    "Index": Number(2),
                    "IsDefault": Bool(false),
                    "IsExternal": Bool(false),
                    "IsForced": Bool(false),
                    "IsHearingImpaired": Bool(false),
                    "IsInterlaced": Bool(false),
                    "IsTextSubtitleStream": Bool(false),
                    "Path": String("/data/music/Agar Agar/Cardan/03 - Cuidado, Peligro, Eclipse.txt"),
                    "SupportsExternalStream": Bool(false),
                    "Type": String("Lyric"),
                    "VideoRange": String("Unknown"),
                    "VideoRangeType": String("Unknown"),
                },
            ],
            "Name": String("03 - Cuidado, Peligro, Eclipse"),
            "Path": String("/data/music/Agar Agar/Cardan/03 - Cuidado, Peligro, Eclipse.mp3"),
            "Protocol": String("File"),
            "ReadAtNativeFramerate": Bool(false),
            "RequiredHttpHeaders": Object {},
            "RequiresClosing": Bool(false),
            "RequiresLooping": Bool(false),
            "RequiresOpening": Bool(false),
            "RunTimeTicks": Number(3600979590),
            "Size": Number(14487065),
            "SupportsDirectPlay": Bool(true),
            "SupportsDirectStream": Bool(true),
            "SupportsProbing": Bool(true),
            "SupportsTranscoding": Bool(true),
            "TranscodingSubProtocol": String("http"),
            "Type": String("Default"),
        },
    ],
    "MediaType": String("Audio"),
    "Name": String("Cuidado, Peligro, Eclipse"),
    "NormalizationGain": Number(-10.45),
    "ParentBackdropImageTags": Array [
        String("ea9ad04d014bd8317aa784ffb5676eac"),
    ],
    "ParentBackdropItemId": String("c910b835045265897c9b1e30417937c8"),
    "ParentId": String("e66386bd52e9e13bcd53fefbe4dbfe80"),
    "ParentIndexNumber": Number(0),
    "PremiereDate": String("2016-01-01T00:00:00.0000000Z"),
    "ProductionYear": Number(2016),
    "RunTimeTicks": Number(3600979590),
    "ServerId": String("97a9003303d7461395074680d9046935"),
    "Type": String("Audio"),
    "UserData": Object {
        "IsFavorite": Bool(false),
        "Key": String("Agar Agar-Cardan [EP]-0000-0003Cuidado, Peligro, Eclipse"),
        "PlayCount": Number(0),
        "PlaybackPositionTicks": Number(0),
        "Played": Bool(false),
    },
}, */

#[derive(Debug, Serialize, Deserialize)]
pub struct Discography {
    #[serde(rename = "Items")]
    pub items: Vec<DiscographySong>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscographyAlbum {
    songs: Vec<DiscographySong>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscographySongUserData {
    #[serde(rename = "PlaybackPositionTicks")]
    playback_position_ticks: u64,
    #[serde(rename = "PlayCount")]
    play_count: u64,
    #[serde(rename = "IsFavorite")]
    is_favorite: bool,
    #[serde(rename = "Played")]
    played: bool,
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscographySong {
    #[serde(rename = "Album", default)]
    pub album: String,
    #[serde(rename = "AlbumArtist", default)]
    pub album_artist: String,
    // #[serde(rename = "AlbumArtists")]
    // album_artists: Vec<Artist>,
    #[serde(rename = "AlbumId", default)]
    pub album_id: String,
    // #[serde(rename = "AlbumPrimaryImageTag")]
    // album_primary_image_tag: String,
    // #[serde(rename = "ArtistItems")]
    // artist_items: Vec<Artist>,
    #[serde(rename = "Artists", default)]
    artists: Vec<String>,
    #[serde(rename = "BackdropImageTags", default)]
    backdrop_image_tags: Vec<String>,
    #[serde(rename = "ChannelId", default)]
    channel_id: Option<String>,
    #[serde(rename = "DateCreated", default)]
    date_created: String,
    // #[serde(rename = "GenreItems")]
    // genre_items: Vec<Genre>,
    #[serde(rename = "Genres", default)]
    genres: Vec<String>,
    #[serde(rename = "HasLyrics", default)]
    has_lyrics: bool,
    #[serde(rename = "Id", default)]
    pub id: String,
    // #[serde(rename = "ImageBlurHashes")]
    // image_blur_hashes: ImageBlurHashes,
    // #[serde(rename = "ImageTags")]
    // image_tags: ImageTags,
    // #[serde(rename = "IndexNumber")]
    // index_number: u64,
    #[serde(rename = "IsFolder", default)]
    is_folder: bool,
    // #[serde(rename = "LocationType")]
    // location_type: String,
    #[serde(rename = "MediaSources")]
    media_sources: Vec<MediaSource>,
    #[serde(rename = "MediaType", default)]
    media_type: String,
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "NormalizationGain", default)]
    normalization_gain: f64,
    // #[serde(rename = "ParentBackdropImageTags")]
    // parent_backdrop_image_tags: Vec<String>,
    // #[serde(rename = "ParentBackdropItemId")]
    // parent_backdrop_item_id: String,
    #[serde(rename = "ParentId", default)]
    pub parent_id: String,
    #[serde(rename = "ParentIndexNumber", default)]
    parent_index_number: u64,
    #[serde(rename = "PremiereDate", default)]
    premiere_date: String,
    #[serde(rename = "ProductionYear", default)]
    pub production_year: u64,
    #[serde(rename = "RunTimeTicks", default)]
    run_time_ticks: u64,
    #[serde(rename = "ServerId", default)]
    server_id: String,
    // #[serde(rename = "Type")]
    // type_: String,
    #[serde(rename = "UserData")]
    user_data: DiscographySongUserData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaSource {
    #[serde(rename = "Container", default)]
    container: String,
    #[serde(rename = "Size", default)]
    size: u64,
    #[serde(rename = "MediaStreams", default)]
    media_streams: Vec<MediaStream>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaStream {
    #[serde(rename = "Codec", default)]
    pub codec: String,
    #[serde(rename = "BitRate", default)]
    pub bit_rate: u64,
    #[serde(rename = "Channels", default)]
    pub channels: u64,
    #[serde(rename = "SampleRate", default)]
    pub sample_rate: u64,
    #[serde(rename = "Type", default)]
    type_: String,
}

/// Lyrics
/*
{
    "Metadata": {},
    "Lyrics": [
        {
            "Text": "Inside you\u0027re pretending"
        },
        {
            "Text": "Crimes have been swept aside"
        },
    ]
}
*/

#[derive(Debug, Serialize, Deserialize)]
pub struct Lyrics {
    #[serde(rename = "Metadata", default)]
    metadata: serde_json::Value,
    #[serde(rename = "Lyrics", default)]
    lyrics: Vec<Lyric>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Lyric {
    #[serde(rename = "Text", default)]
    text: String,
}