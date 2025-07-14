use actix_web::{App, HttpServer, post, web, Responder, HttpResponse};
use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::env;
use google_sheets4::{yup_oauth2, api::ValueRange, hyper_rustls, hyper_util, Sheets};
use google_sheets4::hyper_rustls::HttpsConnector;
use google_sheets4::hyper_util::client::legacy::connect::HttpConnector;
use google_sheets4::yup_oauth2::read_application_secret;
// use tokio_cron_scheduler::{Job, JobScheduler};
use crate::yup_oauth2::Error;

// #[derive(Deserialize, Debug)]
// struct SheetResponse {
//     values: Vec<Vec<String>>,
// }

#[derive(Debug)]
struct Contact {
    phone: String,
    name: String,
    status: String,
}
async fn connect() -> Result<google_sheets4::Sheets<HttpsConnector<HttpConnector>>, Error> {
    let account_key_path = env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .unwrap_or_else(|_| "vivid-motif-388517-f5ba673e8786.json".to_string());
    let secret = yup_oauth2::read_service_account_key(account_key_path)
        .await?;

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await?;

    let client = hyper_util::client::legacy::Client::builder(
        hyper_util::rt::TokioExecutor::new()
    )
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build()
        );
    Ok(Sheets::new(client, auth))
}
async fn fetch_contacts() -> Vec<Contact> {
    let spreadsheet_id = env::var("SPREADSHEET_ID").unwrap();
    let sheet_name = env::var("SHEET_NAME").unwrap();

    let hub = connect().await.expect("Failed to connect to Google Sheets API");
    let res = hub.spreadsheets()
        .values_get(&spreadsheet_id, &format!("{}!A2:C", sheet_name))
        .doit()
        .await
        .expect("Failed to fetch data from Google Sheets");
    let mut contacts = Vec::new();
    if let Some(values) = res.1.values {
        for row in values.iter() {
            // Ensure we have at least 3 columns: phone, status, and name
            if row.len() >= 3 {
                contacts.push(Contact {
                    phone: row[0].to_string().replace("\"", ""),
                    status: row[2].to_string().replace("\"", ""),
                    name: row[1].to_string().replace("\"", ""),
                });
            }
            println!("{}", row[0])
        }
    }
    contacts
}

async fn send_whatsapp_message(contact: &Contact) {
    let sid = env::var("TWILIO_SID").unwrap();
    let token = env::var("TWILIO_TOKEN").unwrap();
    let from = env::var("TWILIO_FROM").unwrap();

    let client = Client::new();
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        sid
    );

    let body = format!(
        "Halo {}, mohon segera melakukan pembayaran iuran bulan ini 🙏",
        contact.name
    );
    println!("{}",contact.phone);
    let params = [
        ("From", from.as_str()),
        ("To", &format!("whatsapp:{}",contact.phone), ),
        ("Body", &body),
        ("ContentSid", "HX45c0fad60564638675cf4b920561325d")

    ];

    let res = client
        .post(&url)
        .basic_auth(sid, Some(token))
        .form(&params)
        .send()
        .await;

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("✅ Reminder sent to {}", contact.phone);
            } else {
                println!("❌ Failed to send to {}: {:?}", contact.phone, resp.text().await);
            }
        }
        Err(e) => println!("❌ Error sending message: {:?}", e),
    }
}


#[derive(Deserialize, Debug)]
struct IncomingCallback {
    body: String,
    from: String,
}


#[post("/callback")]
async fn whatsapp_callback(form: web::Json<IncomingCallback>) -> impl Responder {
    println!("📩 Received callback from {}: {}", form.from, form.body);

    let status = match form.body.to_lowercase().as_str() {
        "sudah" => "Sudah Bayar",
        "belum" => "Belum Bayar",
        _ => "Tidak Diketahui",
    };

    let secret = read_application_secret("vivid-motif-388517-f5ba673e8786.json")
        .await
        .expect("Gagal baca credentials.json");
    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    ).build().await.unwrap();


    let client = hyper_util::client::legacy::Client::builder(
        hyper_util::rt::TokioExecutor::new()
    )
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build()
        );
    let hub = Sheets::new(client.clone(),
                          auth,
    );

    let spreadsheet_id = env::var("SPREADSHEET_ID").expect("SPREADSHEET_ID belum diset");
    let sheet_name = env::var("SHEET_NAME").unwrap_or_else(|_| "Sheet1".to_string());
    let range = format!("{}!A2:C", sheet_name);

    let result = hub.spreadsheets().values_get(&spreadsheet_id, &range).doit().await;

    match result {
        Ok((_, data)) => {
            if let Some(rows) = data.values {
                for (i, row) in rows.iter().enumerate() {
                    let phone_number = form.from.replace("whatsapp:", "");
                    if row.len() > 0 && row[0] == phone_number {
                        let update_range = format!("{}!B{}", sheet_name, i + 2);
                        let body = ValueRange {
                            range: Some(update_range.clone()),
                            major_dimension: Some("ROWS".to_string()),
                            values: Some(vec![vec![serde_json::value::Value::String(status.to_string())]]),
                        };

                        let _ = hub
                            .spreadsheets()
                            .values_update(body, &spreadsheet_id, &update_range)
                            .value_input_option("RAW")
                            .doit()
                            .await;

                        println!("✅ Status {} diupdate untuk {}", status, form.from);
                        break;
                    }
                }
            }
        }
        Err(e) => println!("❌ Gagal ambil sheet: {:?}", e),
    }

    HttpResponse::Ok().body(format!("Status '{}' dicatat untuk {}", status, form.from))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // let sched = JobScheduler::new().await.unwrap();
    //
    // sched
    //     .add(Job::new_async("0 0 * * * *", |_uuid, _l| {
    //         Box::pin(async {
    //             println!("🔁 Fetching and sending reminders...");
    //             let contacts = fetch_contacts().await;
    //             let pending: Vec<_> = contacts
    //                 .into_iter()
    //                 .filter(|c| c.status.to_lowercase() == "belum bayar")
    //                 .collect();
    //
    //             for contact in pending {
    //                 send_whatsapp_message(&contact).await;
    //             }
    //         })
    //     })
    //         .unwrap())
    //     .await
    //     .unwrap();
    //
    // sched.start().await.unwrap();
    let contacts = fetch_contacts().await;
    println!("📊 Fetched {} contacts from Google Sheets", contacts.len());
    let pending: Vec<_> = contacts
        .into_iter()
        .filter(|c| c.status.to_lowercase() == "belum bayar")
        .collect();
    println!("📋 Found {} pending contacts", pending.len());
    for contact in pending {
        send_whatsapp_message(&contact).await;
    }
    println!("🚀 Server running...");
    HttpServer::new(|| App::new().service(whatsapp_callback))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
