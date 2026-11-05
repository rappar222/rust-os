pub struct TimezoneInfo {
    pub name: &'static str,
    pub offset_minutes: i32,
    pub description: &'static str,
}

pub struct RegionInfo {
    pub name: &'static str,
    pub timezones: &'static [TimezoneInfo],
}

pub static REGIONS: &[RegionInfo] = &[
    RegionInfo {
        name: "Europe",
        timezones: &[
            TimezoneInfo { name: "Europe/Bucharest", offset_minutes: 120, description: "Romania (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/London", offset_minutes: 0, description: "United Kingdom (GMT/BST, UTC+0)" },
            TimezoneInfo { name: "Europe/Berlin", offset_minutes: 60, description: "Germany (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Paris", offset_minutes: 60, description: "France (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Rome", offset_minutes: 60, description: "Italy (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Madrid", offset_minutes: 60, description: "Spain (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Amsterdam", offset_minutes: 60, description: "Netherlands (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Athens", offset_minutes: 120, description: "Greece (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Belgrade", offset_minutes: 60, description: "Serbia (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Bratislava", offset_minutes: 60, description: "Slovakia (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Brussels", offset_minutes: 60, description: "Belgium (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Budapest", offset_minutes: 60, description: "Hungary (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Chisinau", offset_minutes: 120, description: "Moldova (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Copenhagen", offset_minutes: 60, description: "Denmark (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Dublin", offset_minutes: 0, description: "Ireland (GMT/IST, UTC+0)" },
            TimezoneInfo { name: "Europe/Helsinki", offset_minutes: 120, description: "Finland (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Istanbul", offset_minutes: 180, description: "Turkey (TRT, UTC+3)" },
            TimezoneInfo { name: "Europe/Kyiv", offset_minutes: 120, description: "Ukraine (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Lisbon", offset_minutes: 0, description: "Portugal (WET/WEST, UTC+0)" },
            TimezoneInfo { name: "Europe/Ljubljana", offset_minutes: 60, description: "Slovenia (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Luxembourg", offset_minutes: 60, description: "Luxembourg (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Monaco", offset_minutes: 60, description: "Monaco (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Moscow", offset_minutes: 180, description: "Russia (MSK, UTC+3)" },
            TimezoneInfo { name: "Europe/Oslo", offset_minutes: 60, description: "Norway (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Prague", offset_minutes: 60, description: "Czech Republic (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Riga", offset_minutes: 120, description: "Latvia (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Sarajevo", offset_minutes: 60, description: "Bosnia & Herzegovina (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Sofia", offset_minutes: 120, description: "Bulgaria (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Stockholm", offset_minutes: 60, description: "Sweden (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Tallinn", offset_minutes: 120, description: "Estonia (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Tirane", offset_minutes: 60, description: "Albania (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Vienna", offset_minutes: 60, description: "Austria (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Vilnius", offset_minutes: 120, description: "Lithuania (EET/EEST, UTC+2)" },
            TimezoneInfo { name: "Europe/Warsaw", offset_minutes: 60, description: "Poland (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Zagreb", offset_minutes: 60, description: "Croatia (CET/CEST, UTC+1)" },
            TimezoneInfo { name: "Europe/Zurich", offset_minutes: 60, description: "Switzerland (CET/CEST, UTC+1)" },
        ],
    },
    RegionInfo {
        name: "Americas (North, Central & South)",
        timezones: &[
            TimezoneInfo { name: "America/New_York", offset_minutes: -300, description: "US Eastern (EST/EDT, UTC-5)" },
            TimezoneInfo { name: "America/Chicago", offset_minutes: -360, description: "US Central (CST/CDT, UTC-6)" },
            TimezoneInfo { name: "America/Denver", offset_minutes: -420, description: "US Mountain (MST/MDT, UTC-7)" },
            TimezoneInfo { name: "America/Los_Angeles", offset_minutes: -480, description: "US Pacific (PST/PDT, UTC-8)" },
            TimezoneInfo { name: "America/Anchorage", offset_minutes: -540, description: "Alaska (AKST/AKDT, UTC-9)" },
            TimezoneInfo { name: "America/Honolulu", offset_minutes: -600, description: "Hawaii (HST, UTC-10)" },
            TimezoneInfo { name: "America/Toronto", offset_minutes: -300, description: "Canada Eastern (EST/EDT, UTC-5)" },
            TimezoneInfo { name: "America/Vancouver", offset_minutes: -480, description: "Canada Pacific (PST/PDT, UTC-8)" },
            TimezoneInfo { name: "America/Montreal", offset_minutes: -300, description: "Canada Montreal (EST/EDT, UTC-5)" },
            TimezoneInfo { name: "America/Edmonton", offset_minutes: -420, description: "Canada Mountain (MST/MDT, UTC-7)" },
            TimezoneInfo { name: "America/Halifax", offset_minutes: -240, description: "Canada Atlantic (AST/ADT, UTC-4)" },
            TimezoneInfo { name: "America/Mexico_City", offset_minutes: -360, description: "Mexico (CST, UTC-6)" },
            TimezoneInfo { name: "America/Bogota", offset_minutes: -300, description: "Colombia (COT, UTC-5)" },
            TimezoneInfo { name: "America/Lima", offset_minutes: -300, description: "Peru (PET, UTC-5)" },
            TimezoneInfo { name: "America/Sao_Paulo", offset_minutes: -180, description: "Brazil (BRT, UTC-3)" },
            TimezoneInfo { name: "America/Buenos_Aires", offset_minutes: -180, description: "Argentina (ART, UTC-3)" },
            TimezoneInfo { name: "America/Santiago", offset_minutes: -240, description: "Chile (CLT, UTC-4)" },
            TimezoneInfo { name: "America/Caracas", offset_minutes: -240, description: "Venezuela (VET, UTC-4)" },
            TimezoneInfo { name: "America/Havana", offset_minutes: -300, description: "Cuba (CST/CDT, UTC-5)" },
            TimezoneInfo { name: "America/Panama", offset_minutes: -300, description: "Panama (EST, UTC-5)" },
            TimezoneInfo { name: "America/Phoenix", offset_minutes: -420, description: "US Arizona (MST, UTC-7, no DST)" },
            TimezoneInfo { name: "America/Costa_Rica", offset_minutes: -360, description: "Costa Rica (CST, UTC-6)" },
            TimezoneInfo { name: "America/Guatemala", offset_minutes: -360, description: "Guatemala (CST, UTC-6)" },
            TimezoneInfo { name: "America/Jamaica", offset_minutes: -300, description: "Jamaica (EST, UTC-5)" },
            TimezoneInfo { name: "America/Montevideo", offset_minutes: -180, description: "Uruguay (UYT, UTC-3)" },
            TimezoneInfo { name: "America/La_Paz", offset_minutes: -240, description: "Bolivia (BOT, UTC-4)" },
            TimezoneInfo { name: "America/Asuncion", offset_minutes: -240, description: "Paraguay (PYT, UTC-4)" },
            TimezoneInfo { name: "America/Santo_Domingo", offset_minutes: -240, description: "Dominican Republic (AST, UTC-4)" },
            TimezoneInfo { name: "America/Puerto_Rico", offset_minutes: -240, description: "Puerto Rico (AST, UTC-4)" },
        ],
    },
    RegionInfo {
        name: "Asia",
        timezones: &[
            TimezoneInfo { name: "Asia/Tokyo", offset_minutes: 540, description: "Japan (JST, UTC+9)" },
            TimezoneInfo { name: "Asia/Shanghai", offset_minutes: 480, description: "China (CST, UTC+8)" },
            TimezoneInfo { name: "Asia/Hong_Kong", offset_minutes: 480, description: "Hong Kong (HKT, UTC+8)" },
            TimezoneInfo { name: "Asia/Taipei", offset_minutes: 480, description: "Taiwan (CST, UTC+8)" },
            TimezoneInfo { name: "Asia/Singapore", offset_minutes: 480, description: "Singapore (SGT, UTC+8)" },
            TimezoneInfo { name: "Asia/Seoul", offset_minutes: 540, description: "South Korea (KST, UTC+9)" },
            TimezoneInfo { name: "Asia/Bangkok", offset_minutes: 420, description: "Thailand / Indochina (ICT, UTC+7)" },
            TimezoneInfo { name: "Asia/Jakarta", offset_minutes: 420, description: "Indonesia Western (WIB, UTC+7)" },
            TimezoneInfo { name: "Asia/Kolkata", offset_minutes: 330, description: "India (IST, UTC+5:30)" },
            TimezoneInfo { name: "Asia/Dubai", offset_minutes: 240, description: "United Arab Emirates / Gulf (GST, UTC+4)" },
            TimezoneInfo { name: "Asia/Riyadh", offset_minutes: 180, description: "Saudi Arabia (AST, UTC+3)" },
            TimezoneInfo { name: "Asia/Tehran", offset_minutes: 210, description: "Iran (IRST, UTC+3:30)" },
            TimezoneInfo { name: "Asia/Jerusalem", offset_minutes: 120, description: "Israel (IST, UTC+2)" },
            TimezoneInfo { name: "Asia/Beirut", offset_minutes: 120, description: "Lebanon (EET, UTC+2)" },
            TimezoneInfo { name: "Asia/Baghdad", offset_minutes: 180, description: "Iraq (AST, UTC+3)" },
            TimezoneInfo { name: "Asia/Almaty", offset_minutes: 300, description: "Kazakhstan (ALMT, UTC+5)" },
            TimezoneInfo { name: "Asia/Tashkent", offset_minutes: 300, description: "Uzbekistan (UZT, UTC+5)" },
            TimezoneInfo { name: "Asia/Manila", offset_minutes: 480, description: "Philippines (PHT, UTC+8)" },
            TimezoneInfo { name: "Asia/Kuala_Lumpur", offset_minutes: 480, description: "Malaysia (MYT, UTC+8)" },
            TimezoneInfo { name: "Asia/Karachi", offset_minutes: 300, description: "Pakistan (PKT, UTC+5)" },
            TimezoneInfo { name: "Asia/Dhaka", offset_minutes: 360, description: "Bangladesh (BST, UTC+6)" },
            TimezoneInfo { name: "Asia/Kathmandu", offset_minutes: 345, description: "Nepal (NPT, UTC+5:45)" },
            TimezoneInfo { name: "Asia/Colombo", offset_minutes: 330, description: "Sri Lanka (IST, UTC+5:30)" },
            TimezoneInfo { name: "Asia/Yangon", offset_minutes: 390, description: "Myanmar (MMT, UTC+6:30)" },
            TimezoneInfo { name: "Asia/Ho_Chi_Minh", offset_minutes: 420, description: "Vietnam (ICT, UTC+7)" },
            TimezoneInfo { name: "Asia/Baku", offset_minutes: 240, description: "Azerbaijan (AZT, UTC+4)" },
            TimezoneInfo { name: "Asia/Tbilisi", offset_minutes: 240, description: "Georgia (GET, UTC+4)" },
            TimezoneInfo { name: "Asia/Yerevan", offset_minutes: 240, description: "Armenia (AMT, UTC+4)" },
            TimezoneInfo { name: "Asia/Ulaanbaatar", offset_minutes: 480, description: "Mongolia (ULAT, UTC+8)" },
            TimezoneInfo { name: "Asia/Vladivostok", offset_minutes: 600, description: "Russia Far East (VLAT, UTC+10)" },
            TimezoneInfo { name: "Asia/Novosibirsk", offset_minutes: 420, description: "Russia Siberia (NOVT, UTC+7)" },
        ],
    },
    RegionInfo {
        name: "Africa",
        timezones: &[
            TimezoneInfo { name: "Africa/Cairo", offset_minutes: 120, description: "Egypt (EEST, UTC+2)" },
            TimezoneInfo { name: "Africa/Johannesburg", offset_minutes: 120, description: "South Africa (SAST, UTC+2)" },
            TimezoneInfo { name: "Africa/Lagos", offset_minutes: 60, description: "Nigeria (WAT, UTC+1)" },
            TimezoneInfo { name: "Africa/Nairobi", offset_minutes: 180, description: "Kenya (EAT, UTC+3)" },
            TimezoneInfo { name: "Africa/Casablanca", offset_minutes: 60, description: "Morocco (WEST, UTC+1)" },
            TimezoneInfo { name: "Africa/Accra", offset_minutes: 0, description: "Ghana (GMT, UTC+0)" },
            TimezoneInfo { name: "Africa/Addis_Ababa", offset_minutes: 180, description: "Ethiopia (EAT, UTC+3)" },
            TimezoneInfo { name: "Africa/Algiers", offset_minutes: 60, description: "Algeria (CET, UTC+1)" },
            TimezoneInfo { name: "Africa/Tunis", offset_minutes: 60, description: "Tunisia (CET, UTC+1)" },
            TimezoneInfo { name: "Africa/Khartoum", offset_minutes: 120, description: "Sudan (CAT, UTC+2)" },
            TimezoneInfo { name: "Africa/Maputo", offset_minutes: 120, description: "Mozambique (CAT, UTC+2)" },
            TimezoneInfo { name: "Africa/Dakar", offset_minutes: 0, description: "Senegal (GMT, UTC+0)" },
            TimezoneInfo { name: "Africa/Kinshasa", offset_minutes: 60, description: "DR Congo (WAT, UTC+1)" },
            TimezoneInfo { name: "Africa/Luanda", offset_minutes: 60, description: "Angola (WAT, UTC+1)" },
            TimezoneInfo { name: "Africa/Dar_es_Salaam", offset_minutes: 180, description: "Tanzania (EAT, UTC+3)" },
            TimezoneInfo { name: "Africa/Harare", offset_minutes: 120, description: "Zimbabwe (CAT, UTC+2)" },
            TimezoneInfo { name: "Africa/Lusaka", offset_minutes: 120, description: "Zambia (CAT, UTC+2)" },
            TimezoneInfo { name: "Africa/Abidjan", offset_minutes: 0, description: "Ivory Coast (GMT, UTC+0)" },
            TimezoneInfo { name: "Africa/Windhoek", offset_minutes: 120, description: "Namibia (CAT, UTC+2)" },
        ],
    },
    RegionInfo {
        name: "Australia & Pacific",
        timezones: &[
            TimezoneInfo { name: "Australia/Sydney", offset_minutes: 600, description: "Australia Eastern (AEST, UTC+10)" },
            TimezoneInfo { name: "Australia/Melbourne", offset_minutes: 600, description: "Australia Victoria (AEST, UTC+10)" },
            TimezoneInfo { name: "Australia/Brisbane", offset_minutes: 600, description: "Australia Queensland (AEST, UTC+10)" },
            TimezoneInfo { name: "Australia/Adelaide", offset_minutes: 570, description: "Australia Central (ACST, UTC+9:30)" },
            TimezoneInfo { name: "Australia/Perth", offset_minutes: 480, description: "Australia Western (AWST, UTC+8)" },
            TimezoneInfo { name: "Australia/Darwin", offset_minutes: 570, description: "Northern Territory (ACST, UTC+9:30)" },
            TimezoneInfo { name: "Australia/Hobart", offset_minutes: 600, description: "Tasmania (AEST, UTC+10)" },
            TimezoneInfo { name: "Pacific/Auckland", offset_minutes: 720, description: "New Zealand (NZST, UTC+12)" },
            TimezoneInfo { name: "Pacific/Fiji", offset_minutes: 720, description: "Fiji (FJT, UTC+12)" },
            TimezoneInfo { name: "Pacific/Guam", offset_minutes: 600, description: "Guam (ChST, UTC+10)" },
            TimezoneInfo { name: "Pacific/Honolulu", offset_minutes: -600, description: "Hawaii (HST, UTC-10)" },
            TimezoneInfo { name: "Pacific/Port_Moresby", offset_minutes: 600, description: "Papua New Guinea (PGT, UTC+10)" },
            TimezoneInfo { name: "Pacific/Noumea", offset_minutes: 660, description: "New Caledonia (NCT, UTC+11)" },
            TimezoneInfo { name: "Pacific/Tahiti", offset_minutes: -600, description: "French Polynesia (TAHT, UTC-10)" },
            TimezoneInfo { name: "Pacific/Tongatapu", offset_minutes: 780, description: "Tonga (TOT, UTC+13)" },
            TimezoneInfo { name: "Pacific/Samoa", offset_minutes: 780, description: "Samoa (WST, UTC+13)" },
        ],
    },
    RegionInfo {
        name: "Atlantic & Antarctica",
        timezones: &[
            TimezoneInfo { name: "Atlantic/Reykjavik", offset_minutes: 0, description: "Iceland (GMT, UTC+0)" },
            TimezoneInfo { name: "Atlantic/Azores", offset_minutes: -60, description: "Azores (AZOT, UTC-1)" },
            TimezoneInfo { name: "Atlantic/Canary", offset_minutes: 0, description: "Canary Islands (WET, UTC+0)" },
            TimezoneInfo { name: "Atlantic/Bermuda", offset_minutes: -240, description: "Bermuda (AST, UTC-4)" },
            TimezoneInfo { name: "Atlantic/Cape_Verde", offset_minutes: -60, description: "Cape Verde (CVT, UTC-1)" },
            TimezoneInfo { name: "Atlantic/Madeira", offset_minutes: 0, description: "Madeira (WET, UTC+0)" },
            TimezoneInfo { name: "Atlantic/South_Georgia", offset_minutes: -120, description: "South Georgia (GST, UTC-2)" },
            TimezoneInfo { name: "Antarctica/McMurdo", offset_minutes: 720, description: "McMurdo Station (NZST, UTC+12)" },
            TimezoneInfo { name: "Antarctica/Palmer", offset_minutes: -180, description: "Palmer Station (CLT, UTC-3)" },
            TimezoneInfo { name: "Antarctica/Troll", offset_minutes: 0, description: "Troll Station (UTC+0)" },
            TimezoneInfo { name: "Antarctica/Casey", offset_minutes: 480, description: "Casey Station (UTC+8)" },
        ],
    },
    RegionInfo {
        name: "Universal & POSIX (UTC / GMT)",
        timezones: &[
            TimezoneInfo { name: "UTC", offset_minutes: 0, description: "Coordinated Universal Time (UTC+0)" },
            TimezoneInfo { name: "GMT", offset_minutes: 0, description: "Greenwich Mean Time (UTC+0)" },
            TimezoneInfo { name: "Etc/GMT-1", offset_minutes: 60, description: "POSIX East 1 (UTC+1)" },
            TimezoneInfo { name: "Etc/GMT-2", offset_minutes: 120, description: "POSIX East 2 (UTC+2)" },
            TimezoneInfo { name: "Etc/GMT-3", offset_minutes: 180, description: "POSIX East 3 (UTC+3)" },
            TimezoneInfo { name: "Etc/GMT-4", offset_minutes: 240, description: "POSIX East 4 (UTC+4)" },
            TimezoneInfo { name: "Etc/GMT-5", offset_minutes: 300, description: "POSIX East 5 (UTC+5)" },
            TimezoneInfo { name: "Etc/GMT-6", offset_minutes: 360, description: "POSIX East 6 (UTC+6)" },
            TimezoneInfo { name: "Etc/GMT-7", offset_minutes: 420, description: "POSIX East 7 (UTC+7)" },
            TimezoneInfo { name: "Etc/GMT-8", offset_minutes: 480, description: "POSIX East 8 (UTC+8)" },
            TimezoneInfo { name: "Etc/GMT-9", offset_minutes: 540, description: "POSIX East 9 (UTC+9)" },
            TimezoneInfo { name: "Etc/GMT-10", offset_minutes: 600, description: "POSIX East 10 (UTC+10)" },
            TimezoneInfo { name: "Etc/GMT-11", offset_minutes: 660, description: "POSIX East 11 (UTC+11)" },
            TimezoneInfo { name: "Etc/GMT-12", offset_minutes: 720, description: "POSIX East 12 (UTC+12)" },
            TimezoneInfo { name: "Etc/GMT+1", offset_minutes: -60, description: "POSIX West 1 (UTC-1)" },
            TimezoneInfo { name: "Etc/GMT+2", offset_minutes: -120, description: "POSIX West 2 (UTC-2)" },
            TimezoneInfo { name: "Etc/GMT+3", offset_minutes: -180, description: "POSIX West 3 (UTC-3)" },
            TimezoneInfo { name: "Etc/GMT+4", offset_minutes: -240, description: "POSIX West 4 (UTC-4)" },
            TimezoneInfo { name: "Etc/GMT+5", offset_minutes: -300, description: "POSIX West 5 (UTC-5)" },
            TimezoneInfo { name: "Etc/GMT+6", offset_minutes: -360, description: "POSIX West 6 (UTC-6)" },
            TimezoneInfo { name: "Etc/GMT+7", offset_minutes: -420, description: "POSIX West 7 (UTC-7)" },
            TimezoneInfo { name: "Etc/GMT+8", offset_minutes: -480, description: "POSIX West 8 (UTC-8)" },
            TimezoneInfo { name: "Etc/GMT+9", offset_minutes: -540, description: "POSIX West 9 (UTC-9)" },
            TimezoneInfo { name: "Etc/GMT+10", offset_minutes: -600, description: "POSIX West 10 (UTC-10)" },
            TimezoneInfo { name: "Etc/GMT+11", offset_minutes: -660, description: "POSIX West 11 (UTC-11)" },
            TimezoneInfo { name: "Etc/GMT+12", offset_minutes: -720, description: "POSIX West 12 (UTC-12)" },
        ],
    },
];

pub fn find_timezone(name: &str) -> Option<&'static TimezoneInfo> {
    let clean = name.trim();
    for reg in REGIONS {
        for tz in reg.timezones {
            if tz.name.eq_ignore_ascii_case(clean) {
                return Some(tz);
            }
            // Also check match against suffix, e.g. "Bucharest", "New_York"
            if let Some(suffix) = tz.name.split('/').last() {
                if suffix.eq_ignore_ascii_case(clean) {
                    return Some(tz);
                }
            }
        }
    }
    None
}

pub fn format_offset_string(minutes: i32) -> alloc::string::String {
    let sign = if minutes >= 0 { '+' } else { '-' };
    let abs_m = minutes.abs();
    let hours = abs_m / 60;
    let mins = abs_m % 60;
    alloc::format!("UTC{}{:02}:{:02}", sign, hours, mins)
}

pub fn convert_utc_to_local(utc: &crate::rtc::DateTime, offset_minutes: i32) -> crate::rtc::DateTime {
    let total_mins = (utc.hour as i32 * 60 + utc.minute as i32) + offset_minutes;
    let mut day_adjust = 0;
    let adjusted_mins = if total_mins < 0 {
        day_adjust = -1;
        total_mins + 24 * 60
    } else if total_mins >= 24 * 60 {
        day_adjust = 1;
        total_mins - 24 * 60
    } else {
        total_mins
    };

    let hour = (adjusted_mins / 60) as u8;
    let minute = (adjusted_mins % 60) as u8;
    let mut day = utc.day as i32 + day_adjust;
    let mut month = utc.month;
    let mut year = utc.year;

    if day <= 0 {
        day = 28; // simplify day underflow fallback
    } else if day > 31 {
        day = 1;
        month = (month % 12) + 1;
        if month == 1 {
            year += 1;
        }
    }

    crate::rtc::DateTime {
        year,
        month,
        day: day as u8,
        hour,
        minute,
        second: utc.second,
    }
}
