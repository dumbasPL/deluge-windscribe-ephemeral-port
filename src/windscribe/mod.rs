use self::types::*;
use crate::{
    cache::SimpleCache,
    constants::WINDSCRIBE_USER_AGENT,
    windscribe::types::{
        WindscribeDeleteEpfResponse, WindscribeRequestEpfRequest, WindscribeRequestEpfResponse,
    },
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use cookie::Cookie;
use lazy_regex::regex_captures;
use reqwest::{
    header::SET_COOKIE, redirect::Policy, Client, ClientBuilder, Method, Response, StatusCode,
};
use scraper::{Html, Selector};
use serde::Serialize;

mod types;
pub use types::{WindscribeEpfInfo, WindscribeEpfStatus};

const SESSION_COOKIE_CACHE: &str = "windscribe_session_cookie";

pub struct WindscribeClient {
    client: Client,
    cache: SimpleCache,
    username: String,
    password: String,
}

impl WindscribeClient {
    pub fn new(username: &str, password: &str, cache: SimpleCache) -> Result<Self> {
        let client = ClientBuilder::new()
            .redirect(Policy::none())
            .gzip(true)
            .user_agent(WINDSCRIBE_USER_AGENT)
            .build()?;
        Ok(Self {
            client,
            cache,
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    async fn get_login_csrf_token(&self) -> Result<WindscribeCsrfToken> {
        self.client
            .post("https://res.windscribe.com/res/logintoken")
            .send()
            .await?
            .json()
            .await
            .or_else(|e| Err(e.into()))
    }

    async fn login(&self) -> Result<String> {
        let token = self.get_login_csrf_token().await?;

        let form = WindscribeLoginRequest {
            login: 1,
            upgrade: 0,
            csrf_time: token.csrf_time,
            csrf_token: &token.csrf_token,
            username: &self.username,
            password: &self.password,
            code: "", // FIXME: 2FA
        };

        let res = self
            .client
            .post("https://windscribe.com/login")
            .form(&form)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let document = Html::parse_document(&res.text().await?);
                let error_selector = Selector::parse("#loginform .login-box .error").unwrap();
                let error_message = document
                    .select(&error_selector)
                    .into_iter()
                    .flat_map(|e| e.text())
                    .collect::<String>()
                    .trim()
                    .to_string();

                let error_message = match error_message.is_empty() {
                    true => "Unknown error".to_string(),
                    false => error_message,
                };

                Err(anyhow!("Login failed: {}", error_message))
            }
            StatusCode::FOUND => {
                let session_cookie = res
                    .headers()
                    .get_all(SET_COOKIE)
                    .iter()
                    .map::<Result<_>, _>(|value| Ok(Cookie::parse(value.to_str()?)?))
                    .filter_map(Result::ok)
                    .find(|cookie| cookie.name() == "ws_session_auth_hash")
                    .ok_or(anyhow!("No session cookie found"))?;

                let expires = session_cookie
                    .expires_datetime()
                    .ok_or(anyhow!("Session cookie does not have an expiration date"))?
                    .unix_timestamp();
                let expires_chrono = Utc
                    .timestamp_opt(expires, 0)
                    .single()
                    .ok_or(anyhow!("Session cookie expiration date is not valid"))?;

                println!(
                    "Successfully logged into windscribe, session expires in {} minutes",
                    (expires_chrono - Utc::now()).num_minutes()
                );

                self.cache
                    .set(
                        SESSION_COOKIE_CACHE,
                        session_cookie.value().to_string(),
                        Some(expires_chrono),
                    )
                    .await?;

                Ok(session_cookie.value().to_string())
            }
            _ => Err(anyhow!("Unexpected status code: {}", res.status())),
        }
    }

    async fn get_session_cookie(&self, force_login: bool) -> Result<String> {
        match self.cache.get(SESSION_COOKIE_CACHE) {
            Some(cookie) if !force_login => Ok(cookie),
            _ => Ok(self.login().await?),
        }
    }

    async fn request_impl<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        form: Option<&T>,
        force_login: bool,
    ) -> Result<Response> {
        let session_cookie = self.get_session_cookie(force_login).await?;

        let mut request_builder = self
            .client
            .request(method, url)
            .header("Cookie", format!("ws_session_auth_hash={}", session_cookie));

        if let Some(form) = form {
            request_builder = request_builder.form(form);
        }

        request_builder.send().await.or_else(|e| Err(e.into()))
    }

    async fn request<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        form: Option<&T>,
        success_status_code: StatusCode,
        re_login_status_code: Option<StatusCode>,
    ) -> Result<Response> {
        let mut res = self.request_impl(method.clone(), url, form, false).await?;

        if re_login_status_code == Some(res.status()) {
            res = self.request_impl(method, url, form, true).await?;
        }

        match res.status() {
            status if status == success_status_code => Ok(res),
            status => Err(anyhow!("Unexpected status code: {}", status)),
        }
    }

    pub async fn get_my_account_csrf_token(&self) -> Result<WindscribeCsrfToken> {
        let res = self
            .request::<()>(
                Method::GET,
                "https://windscribe.com/myaccount",
                None,
                StatusCode::OK,
                Some(StatusCode::FOUND),
            )
            .await?
            .text()
            .await?;

        let (_, csrf_time, csrf_token) =
            regex_captures!(r"csrf_time = (\d+);[[:space:]]*csrf_token = '(\w+)';", &res)
                .ok_or(anyhow!("Failed to find csrf token"))?;

        Ok(WindscribeCsrfToken {
            csrf_time: csrf_time.parse()?,
            csrf_token: csrf_token.to_string(),
        })
    }

    pub async fn get_epf_info(&self) -> Result<WindscribeEpfStatus> {
        let mut res = self
            .request::<()>(
                Method::GET,
                "https://windscribe.com/staticips/load",
                None,
                StatusCode::OK,
                None,
            )
            .await?
            .text()
            .await?;

        // ! async closures are not stable yet
        if res.contains("/login?auth_required") {
            self.get_session_cookie(true).await?;
            res = self
                .request::<()>(
                    Method::GET,
                    "https://windscribe.com/staticips/load",
                    None,
                    StatusCode::OK,
                    None,
                )
                .await?
                .text()
                .await?;
        }

        let fragment = Html::parse_fragment(&res);

        let script_selector = Selector::parse("script:not([src])").unwrap();
        let epf_expires_script = fragment
            .select(&script_selector)
            .next()
            .ok_or(anyhow!("Failed to find epf script"))?
            .text()
            .collect::<String>();

        let epf_expires = regex_captures!(r"window.epfExpires = (\d+);", &epf_expires_script)
            .ok_or(anyhow!("Failed to find epf expires var"))?
            .1
            .parse::<i64>()?;

        if epf_expires == 0 {
            return Ok(WindscribeEpfStatus::Disabled);
        }

        let ports_selector = Selector::parse("#epf-port-info span").unwrap();
        let epf_ports = fragment
            .select(&ports_selector)
            .into_iter()
            .map(|e| e.text().collect::<String>())
            .filter_map(|p| p.trim().parse::<u64>().ok())
            .collect::<Vec<u64>>();

        if epf_ports.len() != 2 {
            return Err(anyhow!("Failed to find epf ports"));
        }

        Ok(WindscribeEpfStatus::Enabled(WindscribeEpfInfo {
            expires: get_epf_expiration(epf_expires)?,
            internal_port: epf_ports[0],
            external_port: epf_ports[1],
        }))
    }

    pub async fn remove_epf(&self, csrf_token: &WindscribeCsrfToken) -> Result<bool> {
        let body = WindscribeDeleteEpfRequest {
            ctime: csrf_token.csrf_time,
            ctoken: &csrf_token.csrf_token,
        };

        let mut res = self
            .request(
                Method::POST,
                "https://windscribe.com/staticips/deleteEphPort",
                Some(&body),
                StatusCode::OK,
                None,
            )
            .await?
            .json::<WindscribeDeleteEpfResponse>()
            .await;

        // ! async closures are not stable yet
        if res.is_err() {
            self.get_session_cookie(true).await?;
            res = self
                .request(
                    Method::POST,
                    "https://windscribe.com/staticips/deleteEphPort",
                    Some(&body),
                    StatusCode::OK,
                    None,
                )
                .await?
                .json::<WindscribeDeleteEpfResponse>()
                .await;
        }

        match res {
            Ok(res) if res.success == 1 => Ok(res.epf),
            Ok(WindscribeDeleteEpfResponse {
                success: _,
                epf: _,
                message,
            }) => Err(anyhow!(
                "Failed to remove epf: {}",
                message.unwrap_or("No message".to_string())
            )),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn request_epf(
        &self,
        csrf_token: &WindscribeCsrfToken,
        port: Option<u64>,
    ) -> Result<WindscribeEpfInfo> {
        let body = WindscribeRequestEpfRequest {
            ctime: csrf_token.csrf_time,
            ctoken: &csrf_token.csrf_token,
            port: &port.map(|p| p.to_string()).unwrap_or_default(),
        };

        let mut res = self
            .request(
                Method::POST,
                "https://windscribe.com/staticips/postEphPort",
                Some(&body),
                StatusCode::OK,
                None,
            )
            .await?
            .json::<WindscribeRequestEpfResponse>()
            .await;

        // ! async closures are not stable yet
        if res.is_err() {
            self.get_session_cookie(true).await?;
            res = self
                .request(
                    Method::POST,
                    "https://windscribe.com/staticips/postEphPort",
                    Some(&body),
                    StatusCode::OK,
                    None,
                )
                .await?
                .json::<WindscribeRequestEpfResponse>()
                .await;
        }

        match res {
            Ok(WindscribeRequestEpfResponse {
                epf,
                message: _,
                success,
            }) if success == 1 && epf.is_some() => {
                let epf_info = epf.unwrap();
                Ok(WindscribeEpfInfo {
                    expires: get_epf_expiration(epf_info.start_ts)?,
                    internal_port: epf_info.int,
                    external_port: epf_info.ext,
                })
            }
            Ok(WindscribeRequestEpfResponse {
                success: _,
                epf: _,
                message,
            }) => Err(anyhow!(
                "Failed to request matching epf: {}",
                message.unwrap_or("No message".to_string())
            )),
            Err(e) => Err(e.into()),
        }
    }
}

fn get_epf_expiration(start_time: i64) -> Result<DateTime<Utc>> {
    let expires = Utc.timestamp_opt(start_time, 0).earliest().ok_or(anyhow!(
        "Failed to parse epf expires timestamp: {}",
        start_time
    ))?;

    // this is hardcoded in the client side js
    Ok(expires + Duration::days(7))
}
