//! Test suite for the Web and headless browsers.

use std::{fs, thread};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use fantoccini::{Client, Locator};

#[tokio::test]
async fn test_println() {
    println!("test_println output");
}

#[tokio::test]
async fn test_browser() {
    let joinable = thread::spawn(move || {
        let _process = match Command::new("npm").args(&["run", "start:dev"]).spawn() {
            Ok(process) => process,
            Err(err) => panic!("Running process error: {}", err),
        };
    });
    let joinable_two = thread::spawn(move || {
        let _process = match Command::new("geckodriver")
            .args(&["--port=4444"])
            .spawn()
        {
            Ok(process) => process,
            Err(err) => panic!("Running process error: {}", err),
        };
    });

    println!("Waiting 2min...");
    sleep(Duration::from_secs(120));
    println!("Starting browser...");
    let mut c = Client::new("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    println!("Open Page...");
    c.goto("http://localhost:8000").await.unwrap();

    let url = c.current_url().await.unwrap();
    assert_eq!(url.as_ref(), "http://localhost:8888");

    println!("Select Hostname Field");
    c.find(Locator::Css("input#homeserver"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    let mut f = c.form(Locator::Css(".login_form")).await.unwrap();
    f.set_by_name("homeserver", "http://localhost:8448")
        .await
        .unwrap()
        .submit()
        .await
        .unwrap();


    let jpeg_data = c.screenshot().await.unwrap();

    fs::write("tests/homeserver_input.jpg", &jpeg_data).unwrap();

    println!("Select MXID Field");
    c.find(Locator::Css("input#username"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    f.set_by_name("username", "@carl:example.com")
        .await
        .unwrap()
        .submit()
        .await
        .unwrap();

    let jpeg_data = c.screenshot().await.unwrap();

    fs::write("tests/username_input.jpg", &jpeg_data).unwrap();

    println!("Select Password Field");
    c.find(Locator::Css("input#password"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    f.set_by_name("password", "12345")
        .await
        .unwrap()
        .submit()
        .await
        .unwrap();

    let jpeg_data = c.screenshot().await.unwrap();
    fs::write("tests/password_input.jpg", &jpeg_data).unwrap();

    println!("Press Login");
    f.submit().await.unwrap();

    c.wait_for_find(Locator::Css("div.scrollable")).await;

    //sleep(Duration::from_secs(5));

    let jpeg_data = c.screenshot().await.unwrap();

    fs::write("tests/main_view.jpg", &jpeg_data).unwrap();
    joinable.join();
    joinable_two.join();
    Command::new("bash")
        .args(&["-c", "\"pkill -f npm\""])
        .spawn()
        .unwrap();
    Command::new("bash")
        .args(&["-c", "\"pkill -f node\""])
        .spawn()
        .unwrap();
}
