pub struct MirrorServer {
    pub name: &'static str,
    pub url: &'static str,
    pub location: &'static str,
    pub bandwidth: &'static str,
}

pub struct CountryMirrors {
    pub country: &'static str,
    pub code: &'static str,
    pub mirrors: &'static [MirrorServer],
}

pub static COUNTRY_MIRRORS: &[CountryMirrors] = &[
    CountryMirrors {
        country: "Worldwide / Global Anycast",
        code: "GLOBAL",
        mirrors: &[
            MirrorServer {
                name: "GeoDNS Tier-1 Global Anycast",
                url: "https://geo.mirror.pkgbuild.com/$repo/os/$arch",
                location: "Global Distributed Anycast",
                bandwidth: "100 Gbps+",
            },
            MirrorServer {
                name: "Cloudflare Global CDN Mirror",
                url: "https://cloudflare.cdn.pkgbuild.com/$repo/os/$arch",
                location: "Cloudflare Edge (300+ Cities)",
                bandwidth: "Anycast Edge CDN",
            },
            MirrorServer {
                name: "Kernel.org Global Master Archive",
                url: "https://mirrors.kernel.org/archlinux/$repo/os/$arch",
                location: "Linux Foundation Tier-1",
                bandwidth: "40 Gbps",
            },
            MirrorServer {
                name: "RustOS Local High-Speed Cache",
                url: "https://local.rustos.mirror/$repo/os/$arch",
                location: "Localhost / Gateway Cache",
                bandwidth: "10 Gbps LAN",
            },
        ],
    },
    CountryMirrors {
        country: "Romania",
        code: "RO",
        mirrors: &[
            MirrorServer {
                name: "RCS & RDS (Digi) Primary Mirror",
                url: "https://archlinux.m1.rdsnet.ro/$repo/os/$arch",
                location: "Bucharest, Romania",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "Hostero Cloud & Dedicated",
                url: "https://mirrors.hostero.eu/archlinux/$repo/os/$arch",
                location: "Bucharest / Cluj, Romania",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Chroot.ro High-Speed Mirror",
                url: "https://mirror.chroot.ro/archlinux/$repo/os/$arch",
                location: "Timisoara, Romania",
                bandwidth: "1 Gbps",
            },
            MirrorServer {
                name: "NAV Communications",
                url: "https://mirrors.nav.ro/archlinux/$repo/os/$arch",
                location: "Bucharest, Romania",
                bandwidth: "1 Gbps",
            },
            MirrorServer {
                name: "Telecom Academy Community Mirror",
                url: "https://archlinux.telecomacademy.ro/$repo/os/$arch",
                location: "Bucharest, Romania",
                bandwidth: "1 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "United States",
        code: "US",
        mirrors: &[
            MirrorServer {
                name: "Massachusetts Institute of Technology (MIT)",
                url: "https://mirrors.mit.edu/archlinux/$repo/os/$arch",
                location: "Cambridge, MA",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Stanford University",
                url: "https://mirrors.stanford.edu/archlinux/$repo/os/$arch",
                location: "Stanford, CA",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Leaseweb United States",
                url: "https://mirror.us.leaseweb.net/archlinux/$repo/os/$arch",
                location: "Manassas, VA / San Jose, CA",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "University of Arizona",
                url: "https://mirror.arizona.edu/archlinux/$repo/os/$arch",
                location: "Tucson, AZ",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Kernel.org US Tier-1",
                url: "https://mirrors.kernel.org/archlinux/$repo/os/$arch",
                location: "San Jose, CA",
                bandwidth: "40 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Germany",
        code: "DE",
        mirrors: &[
            MirrorServer {
                name: "Hetzner Online Data Centers",
                url: "https://mirror.hetzner.de/archlinux/$repo/os/$arch",
                location: "Falkenstein / Nuremberg",
                bandwidth: "20 Gbps Tier-1",
            },
            MirrorServer {
                name: "University of Erlangen-Nuremberg (FAU)",
                url: "https://ftp.uni-erlangen.de/archlinux/$repo/os/$arch",
                location: "Erlangen",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "RWTH Aachen University (Halifax)",
                url: "https://ftp.halifax.rwth-aachen.de/archlinux/$repo/os/$arch",
                location: "Aachen",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "NetCologne Backbone",
                url: "https://mirror.netcologne.de/archlinux/$repo/os/$arch",
                location: "Cologne",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "United Kingdom",
        code: "GB",
        mirrors: &[
            MirrorServer {
                name: "Bytemark Hosting UK",
                url: "https://mirror.bytemark.co.uk/archlinux/$repo/os/$arch",
                location: "York / Manchester",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "University of Kent (UK2 Network)",
                url: "https://archlinux.mirrors.uk2.net/archlinux/$repo/os/$arch",
                location: "Canterbury / London",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Mythic Beasts Community",
                url: "https://mirror.mythic-beasts.com/archlinux/$repo/os/$arch",
                location: "London / Cambridge",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "France",
        code: "FR",
        mirrors: &[
            MirrorServer {
                name: "OVHcloud European Backbone",
                url: "https://archlinux.mirrors.ovh.net/archlinux/$repo/os/$arch",
                location: "Roubaix / Gravelines",
                bandwidth: "20 Gbps Tier-1",
            },
            MirrorServer {
                name: "Free.fr / Iliad Telecom",
                url: "https://mir1.ovh.net/archlinux/$repo/os/$arch",
                location: "Paris",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "IRCAM Paris Research Institute",
                url: "https://mirrors.ircam.fr/pub/archlinux/$repo/os/$arch",
                location: "Paris",
                bandwidth: "2.5 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Canada",
        code: "CA",
        mirrors: &[
            MirrorServer {
                name: "University of Waterloo Computer Science Club",
                url: "https://mirror.cs.uwaterloo.ca/archlinux/$repo/os/$arch",
                location: "Waterloo, ON",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Manitoba Unix User Group (MUUG)",
                url: "https://muug.ca/mirror/archlinux/$repo/os/$arch",
                location: "Winnipeg, MB",
                bandwidth: "1 Gbps",
            },
            MirrorServer {
                name: "Sergal Canada High-Speed",
                url: "https://mirror.sergal.org/archlinux/$repo/os/$arch",
                location: "Montreal, QC",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Australia",
        code: "AU",
        mirrors: &[
            MirrorServer {
                name: "AARNet (Australia's Academic and Research Network)",
                url: "https://mirror.aarnet.edu.au/pub/archlinux/$repo/os/$arch",
                location: "Canberra / Sydney",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Internode Network",
                url: "https://mirror.internode.on.net/pub/archlinux/$repo/os/$arch",
                location: "Adelaide / Melbourne",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "OverTheWire Telecommunications",
                url: "https://mirror.overthewire.com.au/archlinux/$repo/os/$arch",
                location: "Brisbane, QLD",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Japan",
        code: "JP",
        mirrors: &[
            MirrorServer {
                name: "Japan Advanced Institute of Science and Technology (JAIST)",
                url: "https://ftp.jaist.ac.jp/pub/Linux/archlinux/$repo/os/$arch",
                location: "Nomi, Ishikawa",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "CatNet Japan",
                url: "https://mirrors.cat.net/archlinux/$repo/os/$arch",
                location: "Tokyo",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Sensing Online Japan",
                url: "https://ftp.tsukuba.wide.ad.jp/Linux/archlinux/$repo/os/$arch",
                location: "Tsukuba / Tokyo",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Netherlands",
        code: "NL",
        mirrors: &[
            MirrorServer {
                name: "Leaseweb Netherlands",
                url: "https://mirror.nl.leaseweb.net/archlinux/$repo/os/$arch",
                location: "Amsterdam (AMS-IX)",
                bandwidth: "20 Gbps Tier-1",
            },
            MirrorServer {
                name: "WeAreTriple High Bandwidth",
                url: "https://archlinux.mirror.wearetriple.com/$repo/os/$arch",
                location: "Alkmaar / Amsterdam",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Switzerland",
        code: "CH",
        mirrors: &[
            MirrorServer {
                name: "Init7 Gigabit Backbone",
                url: "https://mirror.init7.net/archlinux/$repo/os/$arch",
                location: "Winterthur / Zurich",
                bandwidth: "25 Gbps Tier-1",
            },
            MirrorServer {
                name: "ServerForge Zurich",
                url: "https://archlinux.mirror.serverforge.org/$repo/os/$arch",
                location: "Zurich",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Sweden",
        code: "SE",
        mirrors: &[
            MirrorServer {
                name: "Academic Computer Club (Umea University)",
                url: "https://ftp.acc.umu.se/mirror/archlinux/$repo/os/$arch",
                location: "Umea",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "Lysator Academic Computer Society",
                url: "https://mirror.lysator.liu.se/archlinux/$repo/os/$arch",
                location: "Linkoping",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Brazil",
        code: "BR",
        mirrors: &[
            MirrorServer {
                name: "C3SL Federal University of Parana (UFPR)",
                url: "https://archlinux.c3sl.ufpr.br/$repo/os/$arch",
                location: "Curitiba, PR",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "University of Sao Paulo (USP)",
                url: "https://sft.if.usp.br/archlinux/$repo/os/$arch",
                location: "Sao Paulo, SP",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "India",
        code: "IN",
        mirrors: &[
            MirrorServer {
                name: "Albony Linux India Mirror",
                url: "https://mirror.albony.xyz/archlinux/$repo/os/$arch",
                location: "Bangalore",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "Piconets Mumbai Gateway",
                url: "https://mirrors.piconets.webhop.org/archlinux/$repo/os/$arch",
                location: "Mumbai",
                bandwidth: "1 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Singapore",
        code: "SG",
        mirrors: &[
            MirrorServer {
                name: "0x.sg Singapore Fast Mirror",
                url: "https://mirror.0x.sg/archlinux/$repo/os/$arch",
                location: "Singapore",
                bandwidth: "10 Gbps",
            },
            MirrorServer {
                name: "National University of Singapore (NUS)",
                url: "https://download.nus.edu.sg/mirror/archlinux/$repo/os/$arch",
                location: "Singapore",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "South Korea",
        code: "KR",
        mirrors: &[
            MirrorServer {
                name: "Kakao Corp CDN Mirror",
                url: "https://mirror.kakao.com/archlinux/$repo/os/$arch",
                location: "Seoul / Pangyo",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "KAIST Research Network",
                url: "https://ftp.kaist.ac.kr/archlinux/$repo/os/$arch",
                location: "Daejeon",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Poland",
        code: "PL",
        mirrors: &[
            MirrorServer {
                name: "ICM University of Warsaw",
                url: "https://ftp.icm.edu.pl/pub/Linux/dist/archlinux/$repo/os/$arch",
                location: "Warsaw",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "Onet Telecommunications",
                url: "https://mirror.onet.pl/pub/mirrors/archlinux/$repo/os/$arch",
                location: "Krakow",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Italy",
        code: "IT",
        mirrors: &[
            MirrorServer {
                name: "Consorzio GARR National Research",
                url: "https://archlinux.mirror.garr.it/archlinux/$repo/os/$arch",
                location: "Rome / Milan",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "CrucialX Telecommunications",
                url: "https://mirror.crucialx.net/archlinux/$repo/os/$arch",
                location: "Milan",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "Spain",
        code: "ES",
        mirrors: &[
            MirrorServer {
                name: "RedIRIS Academic Network",
                url: "https://ftp.rediris.es/mirror/archlinux/$repo/os/$arch",
                location: "Madrid",
                bandwidth: "10 Gbps Tier-1",
            },
            MirrorServer {
                name: "Universitat Politecnica de Valencia",
                url: "https://mirrors.upv.es/archlinux/$repo/os/$arch",
                location: "Valencia",
                bandwidth: "10 Gbps",
            },
        ],
    },
    CountryMirrors {
        country: "China",
        code: "CN",
        mirrors: &[
            MirrorServer {
                name: "Tsinghua University TUNA Association",
                url: "https://mirrors.tuna.tsinghua.edu.cn/archlinux/$repo/os/$arch",
                location: "Beijing",
                bandwidth: "20 Gbps Tier-1",
            },
            MirrorServer {
                name: "University of Science and Technology of China (USTC)",
                url: "https://mirrors.ustc.edu.cn/archlinux/$repo/os/$arch",
                location: "Hefei, Anhui",
                bandwidth: "20 Gbps Tier-1",
            },
            MirrorServer {
                name: "Alibaba Cloud Open Source Mirror",
                url: "https://mirrors.aliyun.com/archlinux/$repo/os/$arch",
                location: "Hangzhou",
                bandwidth: "10 Gbps",
            },
        ],
    },
];

pub fn get_country_names() -> &'static [&'static str] {
    static NAMES: &[&str] = &[
        "Worldwide / Global Anycast",
        "Romania",
        "United States",
        "Germany",
        "United Kingdom",
        "France",
        "Canada",
        "Australia",
        "Japan",
        "Netherlands",
        "Switzerland",
        "Sweden",
        "Brazil",
        "India",
        "Singapore",
        "South Korea",
        "Poland",
        "Italy",
        "Spain",
        "China",
    ];
    NAMES
}
