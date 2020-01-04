#![allow(dead_code)]

// Agent identifiers
const ANDROID5_NEXUS7: &str =
    "Mozilla/5.0 (Linux; Android 5.1.1; Nexus 7 Build/LMY47V) AppleWebKit/537.36 (KHTML like Gecko) Chrome/47.0.2526.76 Safari/537.36";
const ANDROID7_GALAXYS8: &str = "Mozilla/5.0 (Linux; Android 7.0; SM-G892A Build/NRD90M; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/60.0.3112.107 Mobile Safari/537.36";
const ANDROID6_NOTE4: &str = "Mozilla/5.0 (Linux; Android 6.0.1; SAMSUNG SM-N910F Build/MMB29M) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/4.0 Chrome/44.0.2403.133 Mobile Safari/537.36";
const BING_BOT: &str = "Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)";
const IPAD_IOS9: &str = "Mozilla/5.0 (iPad; CPU OS 9_1 like Mac OS X) AppleWebKit/601.1.46 (KHTML like Gecko) Version/9.0 Mobile/13B143 Safari/601.1";
const IPAD_IOS12: &str = "Mozilla/5.0 (iPad; CPU OS 12_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/12.0 EdgiOS/44.5.3 Mobile/15E148 Safari/605.1.15";
const IPHONE_IOS8: &str =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 8_0_2 like Mac OS X) AppleWebKit/600.1.4 (KHTML, like Gecko) Version/8.0 Mobile/12A366 Safari/600.1.4";
const IPHONE_IOS9: &str =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 9_1 like Mac OS X) AppleWebKit/601.1.46 (KHTML like Gecko) Version/9.0 Mobile/13B5110e Safari/601.1";
const IPHONE_IOS10: &str =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 10_3_1 like Mac OS X) AppleWebKit/603.1.30 (KHTML, like Gecko) Version/10.0 Mobile/14E304 Safari/602.1";
const IPHONE_IOS11: &str =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 11_0_9 like Mac OS X) AppleWebKit/602.5.3 (KHTML, like Gecko) Version/11.0.3 Mobile/8F48a Safari/6533.18.5";
const IPHONE_IOS12: &str =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 12_3_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/12.0 Mobile/15E148 Safari/605.1.15";
const LINUX_FIREFOX43: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:43.0) Gecko/20100101 Firefox/43.0";
const LINUX_KONQUEROR: &str = "Mozilla/5.0 (compatible; Konqueror/3; Linux)";
const LINUX_MOZILLA: &str = "Mozilla/5.0 (X11; U; Linux i686; en-US; rv:1.4) Gecko/20030624";
const MAC_FIREFOX: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.11; rv:43.0) Gecko/20100101 Firefox/43.0";
const MAC_MOZILLA: &str = "Mozilla/5.0 (Macintosh; U; PPC Mac OS X Mach-O; en-US; rv:1.4a) Gecko/20030401";
const MAC_SAFARI4: &str =
    "Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_6_2; de-at) AppleWebKit/531.21.8 (KHTML like Gecko) Version/4.0.4 Safari/531.21.10";
const MAC_SAFARI9: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_2) AppleWebKit/601.3.9 (KHTML like Gecko) Version/9.0.2 Safari/601.3.9";
const WINDOWS10_CHROME58: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36";
const WINDOWS_CHROME43: &str = "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML like Gecko) Chrome/43.0.2357.125 Safari/537.36";
const WINDOWS_FIREFOX53: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0";
const WINDOWSIE6: &str = "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1)";
const WINDOWSIE7: &str = "Mozilla/4.0 (compatible; MSIE 7.0; Windows NT 5.1; .NET CLR 1.1.4322; .NET CLR 2.0.50727)";
const WINDOWSIE8: &str = "Mozilla/5.0 (compatible; MSIE 8.0; Windows NT 5.1; Trident/4.0; .NET CLR 1.1.4322; .NET CLR 2.0.50727)";
const WINDOWSIE9: &str = "Mozilla/5.0 (compatible; MSIE 9.0; Windows NT 6.1; Trident/5.0)";
const WINDOWSIE10: &str = "Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.2; WOW64; Trident/6.0)";
const WINDOWSIE11: &str = "Mozilla/5.0 (Windows NT 6.3; WOW64; Trident/7.0; rv:11.0) like Gecko";
const WINDOWS_EDGE13: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML like Gecko) Chrome/46.0.2486.0 Safari/537.36 Edge/13.10586";
const WINDOWS_EDGE14: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.79 Safari/537.36 Edge/14.14393";
const WINDOWS_MOZILLA: &str = "Mozilla/5.0 (Windows; U; Windows NT 5.0; en-US; rv:1.4b) Gecko/20030516 Mozilla Firebird/0.6";
const WINDOWS_FIREFOX43: &str = "Mozilla/5.0 (Windows NT 6.3; WOW64; rv:43.0) Gecko/20100101 Firefox/43.0";

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::net::agent;
    use colored::*;
    use std::path::PathBuf;

    #[test]
    fn test_agents() {
        println!("{}", PathBuf::from("foo").cyan());
        assert_ne!(agent::ANDROID5_NEXUS7, "");
        assert_ne!(agent::ANDROID7_GALAXYS8, "");
        assert_ne!(agent::ANDROID6_NOTE4, "");
        assert_ne!(agent::BING_BOT, "");
        assert_ne!(agent::IPAD_IOS9, "");
        assert_ne!(agent::IPAD_IOS12, "");
        assert_ne!(agent::IPHONE_IOS8, "");
        assert_ne!(agent::IPHONE_IOS9, "");
        assert_ne!(agent::IPHONE_IOS10, "");
        assert_ne!(agent::IPHONE_IOS11, "");
        assert_ne!(agent::IPHONE_IOS12, "");
        assert_ne!(agent::LINUX_FIREFOX43, "");
        assert_ne!(agent::LINUX_KONQUEROR, "");
        assert_ne!(agent::LINUX_MOZILLA, "");
        assert_ne!(agent::MAC_FIREFOX, "");
        assert_ne!(agent::MAC_MOZILLA, "");
        assert_ne!(agent::MAC_SAFARI4, "");
        assert_ne!(agent::MAC_SAFARI9, "");
        assert_ne!(agent::WINDOWS10_CHROME58, "");
        assert_ne!(agent::WINDOWS_CHROME43, "");
        assert_ne!(agent::WINDOWS_FIREFOX53, "");
        assert_ne!(agent::WINDOWSIE6, "");
        assert_ne!(agent::WINDOWSIE7, "");
        assert_ne!(agent::WINDOWSIE8, "");
        assert_ne!(agent::WINDOWSIE9, "");
        assert_ne!(agent::WINDOWSIE10, "");
        assert_ne!(agent::WINDOWSIE11, "");
        assert_ne!(agent::WINDOWS_EDGE13, "");
        assert_ne!(agent::WINDOWS_EDGE14, "");
        assert_ne!(agent::WINDOWS_MOZILLA, "");
        assert_ne!(agent::WINDOWS_FIREFOX43, "");
    }
}
