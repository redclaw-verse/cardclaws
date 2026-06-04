//! IP → country/city resolution for analytics geo distribution (PRD §6.7.1,
//! §18). Behind a trait so the real MaxMind lookup (which needs a GeoLite2 mmdb
//! file) is optional and tests can inject a deterministic fake.
//!
//! The real resolver is gated behind the `geoip` feature; without it the
//! `NullGeoResolver` is used and country/city stay unresolved (analytics still
//! work, just without geo).

use std::net::IpAddr;

pub struct GeoLocation {
    pub country: Option<String>,
    pub city: Option<String>,
}

pub trait GeoResolver: Send + Sync {
    /// Resolve a raw IP string to a location. Never errors — unknown IPs simply
    /// return empty fields.
    fn resolve(&self, ip: &str) -> GeoLocation;
}

/// Default resolver: resolves nothing. Used when `geoip` is disabled.
pub struct NullGeoResolver;

impl GeoResolver for NullGeoResolver {
    fn resolve(&self, _ip: &str) -> GeoLocation {
        GeoLocation {
            country: None,
            city: None,
        }
    }
}

#[cfg(feature = "geoip")]
pub use maxmind::MaxMindGeoResolver;

#[cfg(feature = "geoip")]
mod maxmind {
    use std::net::IpAddr;
    use std::sync::Arc;

    use maxminddb::{geoip2, Reader};

    use super::{GeoLocation, GeoResolver};

    /// Production resolver backed by a MaxMind GeoLite2/GeoIP2 City database.
    pub struct MaxMindGeoResolver {
        reader: Arc<Reader<Vec<u8>>>,
    }

    impl MaxMindGeoResolver {
        pub fn open(mmdb_path: &str) -> Result<Self, String> {
            let reader = Reader::open_readfile(mmdb_path).map_err(|e| e.to_string())?;
            Ok(Self {
                reader: Arc::new(reader),
            })
        }
    }

    impl GeoResolver for MaxMindGeoResolver {
        fn resolve(&self, ip: &str) -> GeoLocation {
            let Ok(addr) = ip.parse::<IpAddr>() else {
                return GeoLocation {
                    country: None,
                    city: None,
                };
            };
            match self.reader.lookup::<geoip2::City>(addr) {
                Ok(city) => GeoLocation {
                    country: city.country.and_then(|c| c.iso_code).map(|s| s.to_string()),
                    city: city
                        .city
                        .and_then(|c| c.names)
                        .and_then(|n| n.get("en").map(|s| s.to_string())),
                },
                Err(_) => GeoLocation {
                    country: None,
                    city: None,
                },
            }
        }
    }
}

/// Validate that a string parses as an IP (used to avoid storing junk).
pub fn is_ip(s: &str) -> bool {
    s.parse::<IpAddr>().is_ok()
}
