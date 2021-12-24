use serde_json::json;
use urlfreezer_client::LinkAction;

#[async_std::test]
#[cfg(feature = "async")]
async fn test_async() {
    use urlfreezer_client::non_blocking::Client;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    // Start a background HTTP server on a random local port
    let mock_server = MockServer::start().await;

    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    Mock::given(method("POST"))
        .and(path("/api/fetch_links_v2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"links":[{"link":"http://exp.com/bla", "link_label":"nana","action":"Redirect","link_id":"ASXDAERERE"}], "base":"https://example.com"})))
        // Mounting the mock on the mock server - it's now effective!
        .mount(&mock_server)
        .await;
    let uri = mock_server.uri();
    let client = Client::connect_host(&uri, "nothing").await.unwrap();
    let fetched = client
        .fetch_link(
            &"http://exp.com/bla",
            Some("http://local.com/page.html"),
            Some("nana"),
        )
        .await
        .unwrap();
    let data = fetched.expect("there is some info");
    assert_eq!(&data.original, "http://exp.com/bla");
    assert_eq!(
        data.page.as_ref().map(String::as_str),
        Some("http://local.com/page.html")
    );
    assert_eq!(data.label.as_ref().map(String::as_str), Some("nana"));
    assert_eq!(data.link, "https://example.com/ASXDAERERE");
    assert_eq!(data.action, LinkAction::Redirect);
    //assert_eq!(status, 404);
}

#[test]
fn test_blocking() {
    use httpmock::{Method, MockServer};
    use urlfreezer_client::blocking::Client;
    // Start a lightweight mock server.
    let server = MockServer::start();

    // Create a mock on the server.
    let _hello_mock = server.mock(|when, then| {
    when.method(Method::POST)
        .path("/api/fetch_links_v2")
        ;
    then.status(200)
        .header("content-type", "application/json")
        .json_body(json!({"links":[{"link":"http://exp.com/bla", "link_label":"nana","action":"Redirect","link_id":"ASXDAERERE"}], "base":"https://example.com"}));
});
    let url = server.base_url();
    let client = Client::connect_host(&url, "nothing").unwrap();
    let fetched = client
        .fetch_link(
            &"http://exp.com/bla",
            Some("http://local.com/page.html"),
            Some("nana"),
        )
        .unwrap();
    let data = fetched.expect("there is some info");
    assert_eq!(&data.original, "http://exp.com/bla");
    assert_eq!(
        data.page.as_ref().map(String::as_str),
        Some("http://local.com/page.html")
    );
    assert_eq!(data.label.as_ref().map(String::as_str), Some("nana"));
    assert_eq!(data.link, "https://example.com/ASXDAERERE");
    assert_eq!(data.action, LinkAction::Redirect);
}
