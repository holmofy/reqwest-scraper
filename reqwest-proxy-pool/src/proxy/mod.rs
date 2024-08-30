mod proxy_fetch;

pub struct Proxy {
    ip: String,
    port: u16,
    ty: ProxyType,
    pri: Privacy,
}

enum ProxyType {
    Http,
    Https,
    Socks,
}

enum Privacy {
    Unknown,
    Anonymity,
    HighAnonymity,
}

pub(crate) trait IntoProxy {
    fn make_proxy() -> Proxy;
}
