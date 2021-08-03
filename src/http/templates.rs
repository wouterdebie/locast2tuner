use crate::utils::aspect_ratio;
use crate::utils::format_date;
use crate::utils::format_date_iso;
use crate::utils::format_time;
use crate::utils::format_time_local_iso;
use crate::utils::quality;
use crate::utils::split;
use crate::{config::Config, service::station::Station, service::stationprovider::StationProvider};
use chrono_tz::Tz;
use format_xml::xml;
use htmlescape::encode_minimal;

pub fn device_xml<T: StationProvider>(config: &Config, service: &T, host: String) -> String {
    let r = xml! {
        <root xmlns="urn:schemas-upnp-org:device-1-0">
        <specVersion>
          <major>1</major>
          <minor>0</minor>
        </specVersion>
        <device>
          <deviceType>{"urn:schemas-upnp-org:device:MediaServer:1"}</deviceType>
          <friendlyName>{service.geo().name}</friendlyName>
          <manufacturer>{"locast2tuner"}</manufacturer>
          <modelName>{config.device_model}</modelName>
          <modelNumber>{config.device_version}</modelNumber>
          <serialNumber/>
          <UDN>{"uuid:"}{service.uuid()}</UDN>
        </device>
        <URLBase>{"http://"}{host}</URLBase>
      </root>
    }
    .to_string();
    r
}

pub fn lineup_xml(stations: &[Station], host: String) -> String {
    let r = xml! {
        <Lineup>
            for station in (stations.iter().filter(|s| s.active)) {
                <Program>
                    <GuideNumber>{encode_minimal(station.channel_remapped.as_ref().unwrap_or_else(|| station.channel.as_ref().unwrap()))}</GuideNumber>
                    <GuideName>{encode_minimal(&station.name)}</GuideName>
                    <URL>{"http://"}{host}{"/watch/"}{station.id}</URL>
                </Program>
            }
        </Lineup>
    }.to_string();
    r
}
pub fn epg_xml(stations: &[Station]) -> String {
    let xml_version = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n";
    let doctype =
        "<!DOCTYPE tv SYSTEM \"https://raw.githubusercontent.com/XMLTV/xmltv/master/xmltv.dtd\">\n";
    let r = xml! {
        <tv generator-info-name="locast2tuner">
        for station in (stations.iter().filter(|s| s.active)) {
            <channel id={format!("channel.{}",station.id)}>
                <display-name lang="en">{encode_minimal(station.callSign_remapped.as_ref().unwrap_or(&station.callSign))}</display-name>
                <display-name lang="en">{format!("{} {}", encode_minimal(station.channel_remapped.as_ref().unwrap_or_else(|| station.channel.as_ref().unwrap())), encode_minimal(station.callSign_remapped.as_ref().unwrap_or(&station.callSign)))}</display-name>
                <display-name lang="en">{encode_minimal(&station.name)}</display-name>
                <display-name lang="en">{encode_minimal(station.channel_remapped.as_ref().unwrap_or_else(|| station.channel.as_ref().unwrap()))}</display-name>
                <display-name lang="en">{station.id}</display-name>
                <icon src={encode_minimal(station.logoUrl.as_ref().unwrap())} />
            </channel>
        }
        for station in (stations){
            let timezone = station.timezone.as_ref().unwrap().parse::<Tz>().unwrap();
            for program in (&station.listings) {
                <programme start={format_time(program.startTime)}  stop={format_time(program.startTime + program.duration * 1000)} channel={format!("channel.{}",station.id)}>
                    <title lang="en">{encode_minimal(&program.title)}</title>
                    if let Some(description) = (&program.description) {
                        <desc lang="en">{encode_minimal(description)}</desc>
                    }
                    if (program.directors.is_some() || program.topCast.is_some()){
                        <credits>
                            if let Some(directors) = (&program.directors) {
                                for director in (split(directors, ", ")){
                                    <director>{encode_minimal(&director)}</director>
                                }
                            }
                            if let Some(actors) = (&program.topCast) {
                                for actor in (split(actors, ", ")){
                                    <actor>{encode_minimal(&actor)}</actor>
                                }
                            }
                        </credits>
                    }
                    if let Some(release_date) = (program.releaseDate) {
                        <date>{format_date(release_date)}</date>
                    }
                    if let Some(genres) = (&program.genres) {
                        for genre in (split(genres, ", ")){
                            <category lang="en">{encode_minimal(&genre)}</category>
                        }
                    }
                    <category lang="en">{encode_minimal(program.showType.as_ref().unwrap_or(&"unknown".to_string()))}</category>
                    <length units="seconds">{program.duration}</length>

                    if (program.preferredImage.is_some() && program.preferredImageHeight.is_some() && program.preferredImageWidth.is_some()){
                        <icon src={encode_minimal(program.preferredImage.as_ref().unwrap())} height={program.preferredImageHeight.unwrap()} width={program.preferredImageWidth.unwrap()}/>
                    }

                    if (program.episodeNumber.is_some() && program.seasonNumber.is_some()) {
                        <episode-num system="xmltv_ns">{format!("{}.{}.", program.seasonNumber.unwrap() - 1, program.episodeNumber.unwrap() - 1)}</episode-num>
                        <episode-num>{format!("S{:02}E{:02}", program.seasonNumber.unwrap() - 1, program.episodeNumber.unwrap() - 1)}</episode-num>
                    } else if (program.episodeNumber.is_some()) {
                        <episode-num system="xmltv_ns">{format!("0.{}.", program.episodeNumber.unwrap() - 1)}</episode-num>
                    } else if (program.genres.is_some() && &program.genres.as_ref().unwrap().to_owned() == "News" || (program.entityType != "Movie" && program.isNew.is_some() && program.isNew.unwrap())) {
                        <episode-num system="original-air-date">{format_time_local_iso(program.startTime, &timezone)}</episode-num>
                    } else if (program.entityType != "Movie" && program.airdate.is_some()){
                        <episode-num system="original-air-date">{format_date_iso(*program.airdate.as_ref().unwrap())}</episode-num>
                    }

                    <episode-num system="dd_progid">{program.programId}</episode-num>
                    if let Some(video_properties) = (&program.videoProperties){
                        <video>
                            <present>{"yes"}</present>
                            <aspect>{aspect_ratio(video_properties)}</aspect>
                            <quality>{quality(video_properties)}</quality>
                        </video>
                    }

                    <audio>
                    <present>{"yes"}</present>
                    <stereo>{"stereo"}</stereo>
                    </audio>

                    if (program.isNew.is_some() && *program.isNew.as_ref().unwrap()){
                        <new />
                    } else if (program.airdate.is_some()) {
                        <previously-shown start={format_date_iso(*program.airdate.as_ref().unwrap())}/>
                    } else {
                        <previously-shown />
                    }

                    if let Some(rating) = (&program.rating) {
                        <rating system="VCHIP">
                        <value>{rating}</value>
                        </rating>
                    }
                </programme>
            }
        }
        </tv>
    }
    .to_string();
    format!("{}{}{}", xml_version, doctype, r)
}
