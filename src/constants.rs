use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::Duration;

pub const BASE_URL: &str = "https://yopmail.com";
pub const VERSION: &str = "9.2";
pub const YJ_TOKEN: &str = "IZwx0AGH1BQxjBQx1ZmNmBQR";
pub const AD_PARAM: i32 = 0;
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

// Updated from the Python client snapshot (October 14, 2025)
pub const DEFAULT_COOKIES: &[(&str, &str)] = &[
    ("yc", "EAGNlBGD2Awx4ZmpkZGN4ZQV"),
    ("yses", "zz6dtenHstru+L/GLPPQD4a5iJbTzoLzBsyP3HkfhNIwBQRWRdGPgRYto8uoBVoi"),
    ("FCNEC", "%5B%5B%22AKsRol_6F42HOVxM6EaK5AzHHz6pBZ_s5IEy0SEsxyy-uoiU8y8_xL4dEzrZFin7v--j4O2_PFq0BRr3_VLVsDP3GGZGL2OsU1ZWEokkR_RZ_jBrvi4Xp4fFvcD1SJdlzRJsLemj_U5VBJ7SdrdAe49PIX4OE2eYyA%3D%3D%22%5D%5D"),
    ("FCCDCF", "%5Bnull%2Cnull%2Cnull%2C%5B%22CQWjP4AQWjP4AEsACBENB5FoAP_gAEPgAAqIK1IB_C7EbCFCiDp3IKMEMAhHABBAYsAwAAYBAwAADBIQIAQCgkEYBASAFCACCAAAKASBAAAgCAAAAUAAIAAVAABAAAwAIBAIIAAAgAAAAEAIAAAACIAAEQCAAAAEAEAAkAgAAAIASAAAAAAAAACBAAAAAAAAAAAAAAAABAAAAQAAQAAAAAAAiAAAAAAAABAIAAAAAAAAAAAAAAAAAAAAAAgAAAAAAAAAABAAAAAAAQR2QD-F2I2EKFEHCuQUYIYBCuACAAxYBgAAwCBgAAGCQgQAgFJIIkCAEAIEAAEAAAQAgCAABQEBAAAIAAAAAqAACAABgAQCAQQIABAAAAgIAAAAAAEQAAIgEAAAAIAIABABAAAAQAkAAAAAAAAAECAAAAAAAAAAAAAAAAAAAAAEABgAAAAAABEAAAAAAAACAQIAAA.cAAAAAAAAAA%22%2C%222~61.89.122.184.196.230.314.442.445.494.550.576.827.1029.1033.1046.1047.1051.1097.1126.1166.1301.1342.1415.1725.1765.1942.1958.1987.2068.2072.2074.2107.2213.2219.2223.2224.2328.2331.2387.2416.2501.2567.2568.2575.2657.2686.2778.2869.2878.2908.2920.2963.3005.3023.3100.3126.3219.3234.3235.3253.3309.3731.6931.8931.13731.15731~dv.%22%2C%220300B232-3BBA-4065-9DF9-FA0EF3FB75D7%22%5D%5D"),
    ("compte", "testuserauto2:righthandpath:testuserauto3:testuserx:testuserauto1:test_agent_a_20251012t221230z:test_agent_b_20251012t221230z:owner:advertiser:asdhsdaq"),
    ("__eoi", "ID=35600ad0fb561277:T=1755902461:RT=1760393936:S=AA-AfjbMAapjaFGpb5UM0DMBDBj6"),
    ("__gads", "ID=50e1fb970662d3ce:T=1755902461:RT=1760393936:S=ALNI_MY3XfyUKaP7QZR4LLOzYjceTyPLsg"),
    ("__gpi", "UID=0000126838c12cc0:T=1755902461:RT=1758731437:S=ALNI_MZv-G62t2o_oOP7-v_-_5Odq56sWA"),
    ("ywm", "testuserauto2"),
];

pub fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (k, v) in DEFAULT_HEADERS {
        let name = HeaderName::from_static(k);
        if let Ok(val) = HeaderValue::from_str(v) {
            headers.insert(name, val);
        }
    }
    headers
}

pub const DEFAULT_HEADERS: &[(&str, &str)] = &[
    ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
    ("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    ("accept-language", "en-US,en;q=0.5"),
    ("accept-encoding", "gzip, deflate"),
    ("connection", "keep-alive"),
    ("upgrade-insecure-requests", "1"),
];

pub const INBOX_HEADERS: &[(&str, &str)] = &[
    ("referer", "https://yopmail.com/wm"),
    ("sec-fetch-dest", "iframe"),
    ("sec-fetch-mode", "navigate"),
    ("sec-fetch-site", "same-origin"),
];

// Mail fetch headers (closer to browser)
pub const MAIL_HEADERS: &[(&str, &str)] = &[
    ("referer", "https://yopmail.com/en/wm"),
    ("sec-fetch-dest", "iframe"),
    ("sec-fetch-mode", "navigate"),
    ("sec-fetch-site", "same-origin"),
    ("sec-fetch-user", "?1"),
    ("upgrade-insecure-requests", "1"),
];

pub const SEND_HEADERS: &[(&str, &str)] = &[
    ("content-type", "application/x-www-form-urlencoded"),
    ("origin", "https://yopmail.com"),
    ("referer", "https://yopmail.com/wm"),
    ("accept", "*/*"),
    ("accept-language", "en-US,en;q=0.9,de-DE;q=0.8,de;q=0.7"),
    ("accept-encoding", "gzip, deflate, br, zstd"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "cors"),
    ("sec-fetch-site", "same-origin"),
    ("priority", "u=1, i"),
];

pub const MESSAGE_ID_PREFIX: &str = "me_";
pub const FALLBACK_YP_TOKEN: &str = "ZAGplZmp0ZmR3ZQN4ZGx1ZGR";

pub fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_TIMEOUT_SECS)
}
