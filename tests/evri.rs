mod support;

use wreq::{Client, Proxy};
use wreq_util::Emulation;

#[tokio::test]
async fn test_evri() {
    let emulation = Emulation::random_ja4();
    let client = Client::builder()
        .emulation(emulation)
        // .emulation(EmulationOption::builder().emulation(Emulation::Safari26_1).emulation_os(EmulationOS::MacOS).build())
        .build()
        .unwrap();
    let resp = client.get("https://tracking.platform-apis.evri.com/v1/parcels?uniqueIds=urn:parcel_id:barcode:date:1124719100:H04AQA0004726589:2025-12-02")
        .header("sec-ch-ua-platform", "macOS")
        .header("Referer", "https://www.evri.com/")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .header("sec-ch-ua", "Chromium;v=142, Google Chrome;v=142, Not_A Brand;v=99")
        .header("apikey", "tUdU06nvt4RaUPcqwadhHIpsTdQvWpqt")
        .header("sec-ch-ua-mobile", "?0")
        .proxy(Proxy::all("http://brd-customer-hl_4001c46a-zone-datacenter_proxy_by_usage-country-uk:v0h7lvlwtdjm@brd.superproxy.io:33335").unwrap())   
        .send()
        .await.unwrap();

    assert_eq!(resp.status(), wreq::StatusCode::OK);
    // let content = resp.text().await.unwrap();
    // println!("{}", content);
}