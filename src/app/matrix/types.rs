use url::Url;

pub fn get_media_download_url(homeserver: &Url, mxc_url: &str) -> Url {
    let url_parts_raw = mxc_url.replace("mxc://", "");
    let url_parts: Vec<&str> = url_parts_raw.split('/').collect();
    let server_name = (*url_parts.first().unwrap()).to_string();
    let media_id = (*url_parts.last().unwrap()).to_string();
    let new_path = format!(
        "_matrix/media/r0/download/{}/{}/fix.jpg",
        server_name, media_id,
    );
    let mut new_url = homeserver.clone();
    new_url.set_path(new_path.as_str());
    new_url
}

pub fn get_video_media_download_url(homeserver: &Url, mxc_url: String) -> Url {
    let url_parts_raw = mxc_url.replace("mxc://", "");
    let url_parts: Vec<&str> = url_parts_raw.split('/').collect();
    let server_name = (*url_parts.first().unwrap()).to_string();
    let media_id = (*url_parts.last().unwrap()).to_string();
    let new_path = format!(
        "_matrix/media/r0/download/{}/{}/fix.mp4",
        server_name, media_id,
    );
    let mut new_url = homeserver.clone();
    new_url.set_path(new_path.as_str());
    new_url
}
