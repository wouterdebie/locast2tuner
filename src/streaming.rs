use crate::utils::base_url;
use futures::Stream;
use log::info;
use reqwest::Url;
use std::collections::VecDeque;
use std::{convert::TryFrom, io::Error};

pub struct StreamBody {
    url: String,
    segments: VecDeque<Segment>,
}

#[derive(Debug)]
struct Segment {
    url: String,
    played: bool,
}
impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl StreamBody {
    pub fn new(url: String) -> StreamBody {
        StreamBody {
            url,
            segments: VecDeque::new(),
        }
    }
}

impl Stream for StreamBody {
    type Item = Result<actix_web::web::Bytes, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let m3u_data = crate::utils::get(&self.url, None).text().unwrap();
        let media_playlist = hls_m3u8::MediaPlaylist::try_from(m3u_data.as_str()).unwrap();
        let base_url = base_url(Url::parse(&self.url).unwrap());

        for media_segment in media_playlist.segments {
            let (_i, ms) = media_segment;
            let absolute_uri = base_url.join(ms.uri()).unwrap();
            let s = Segment {
                url: absolute_uri.to_string(),
                played: false,
            };
            if !self.segments.contains(&s) {
                info!("Added segment {:?}", &s);
                self.segments.push_back(s);
            }
        }

        // Find first unplayed segment
        let first = self.segments.iter_mut().find(|s| !s.played).unwrap();

        let chunk = crate::utils::get(&first.url, None)
            .bytes()
            .unwrap()
            .to_vec();
        first.played = true;
        info!("Playing: segment {:?}", first);

        return std::task::Poll::Ready(Some(Ok(actix_web::web::Bytes::from(chunk))));
    }
}
