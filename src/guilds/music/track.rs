use invidious::CommonVideo;

#[derive(PartialEq)]
pub struct Track {
    pub title: String,
    pub source: Source,
    pub thumbnail: Option<String>,
}

impl Clone for Track {
    fn clone(&self) -> Self {
        Track {
            title: self.title.clone(),
            source: self.source.clone(),
            thumbnail: self.thumbnail.clone(),
        }
    }
}

impl Track {
    pub fn from_youtube(video: CommonVideo) -> Self {
        Track {
            thumbnail: video.thumbnails.first().map(|t| t.url.clone()),
            title: video.title.clone(),
            source: Source::Youtube(video),
        }
    }
}

/// T is the original resource for the source
#[derive(Clone)]
pub enum Source {
    Youtube(CommonVideo),
}

impl Source {
    pub fn get_url(&self) -> String {
        match self {
            Source::Youtube(source) => format!("https://www.youtube.com/watch?v={}", source.id),
        }
    }
}

impl PartialEq<Source> for Source {
    fn eq(&self, other: &Source) -> bool {
        match (self, other) {
            (Source::Youtube(a), Source::Youtube(b)) => a.id == b.id,
        }
    }
}
