//! ASN Lookup — Map IP Address to Iranian ISP
//!
//! Uses an embedded database of ASN→ISP mappings derived from
//! isp-profiles.json. For production, pair with MaxMind GeoLite2-ASN
//! or ip-api.com for more complete coverage.

use std::collections::HashMap;

/// Build the static ASN→ISP map from isp-profiles.json data.
pub fn build_asn_map() -> HashMap<u32, &'static str> {
    let mut m = HashMap::new();
    // MCI
    for asn in [41689u32, 197207, 43754] { m.insert(asn, "mci"); }
    // Irancell
    for asn in [39074u32, 43235, 44400, 49581] { m.insert(asn, "irancell"); }
    // Rightel
    for asn in [48434u32, 49501, 51074, 52193] { m.insert(asn, "rightel"); }
    // Shatel
    for asn in [34918u32, 57218] { m.insert(asn, "shatel"); }
    // Asiatech
    m.insert(56402, "asiatech");
    // ParsOnline
    for asn in [16322u32, 42337] { m.insert(asn, "pars_online"); }
    // Afranet
    m.insert(25184, "afranet");
    // Mobinnet
    m.insert(31549, "mobinnet");
    // Fanava
    m.insert(24631, "fanava");
    // Mokhaberat/TCI
    for asn in [12880u32, 44889, 47262, 58224] { m.insert(asn, "mokhaberat"); }
    // IRIB
    m.insert(44244, "irib");
    // ITC/DCI
    for asn in [48159u32] { m.insert(asn, "itc"); }
    m
}

/// Lookup ISP ID from ASN number.
pub fn lookup_isp_by_asn(asn: u32) -> Option<&'static str> {
    build_asn_map().get(&asn).copied()
}
