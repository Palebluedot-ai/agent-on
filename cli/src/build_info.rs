use serde::Serialize;

#[derive(Serialize)]
pub struct BuildInfo {
    pub version: &'static str,
    pub commit: &'static str,
    pub release_tag: Option<&'static str>,
    pub source_dirty: Option<bool>,
    pub source_fingerprint: &'static str,
    pub built_from: &'static str,
}

pub fn current() -> BuildInfo {
    let version = env!("CARGO_PKG_VERSION");
    let tag = env!("AGENT_ON_BUILD_TAG");
    let source_dirty = env!("AGENT_ON_SOURCE_DIRTY").parse::<bool>().ok();
    BuildInfo {
        version,
        commit: env!("AGENT_ON_BUILD_COMMIT"),
        release_tag: (source_dirty == Some(false) && tag == format!("v{version}")).then_some(tag),
        source_dirty,
        source_fingerprint: env!("AGENT_ON_SOURCE_ID"),
        built_from: env!("AGENT_ON_SOURCE_ROOT"),
    }
}

pub fn report(json: bool) -> String {
    let info = current();
    if json {
        format!("{}\n", serde_json::to_string_pretty(&info).unwrap())
    } else {
        format!(
            "agent-on {}\ncommit: {}\nrelease tag: {}\nsource dirty: {:?}\nsource: {}\nbuilt from: {}\n",
            info.version,
            info.commit,
            info.release_tag.unwrap_or("unreleased/unverified"),
            info.source_dirty,
            info.source_fingerprint,
            info.built_from,
        )
    }
}
