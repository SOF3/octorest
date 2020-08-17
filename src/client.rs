//! The GitHub API client

use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use futures::lock::Mutex;
use getset::Getters;
use reqwest::header;
use reqwest::multipart::Form;

/// An authenticated GitHub API client
#[derive(Getters)]
pub struct Client {
    /// The backing reqwest client
    #[getset(get = "pub")]
    reqwest: reqwest::Client,
    /// The authentication provider
    #[getset(get = "pub")]
    auth: Mutex<Auth>,
}

impl Client {
    /// Creates an unauthenticated API client.
    ///
    /// Unauthenticated clients are limited to [60 requests per hour][rate-limiting] per IP address.
    /// Consider creating a token for a dummy user account and use
    /// [`from_token`](#method.from_token) to increase rate limit to 5000 per hour.
    ///
    /// [rate-limiting]: https://developer.github.com/v3/#rate-limiting
    pub fn create_unauthenticated(reqwest: reqwest::Client) -> Self {
        Self {
            reqwest,
            auth: Mutex::new(Auth(AuthImpl::None)),
        }
    }

    /// Creates an API client associated with a static access token.
    ///
    /// The token can be created from the GitHub settings page directly,
    /// obtained via OAuth web/non-web flow or
    /// obtained via integrations/installations API.
    /// However, there are dedicated methods for the latter methods.
    pub fn create_from_token(reqwest: reqwest::Client, token: &str) -> Self {
        Self {
            reqwest,
            auth: Mutex::new(Auth(AuthImpl::Static(Arc::from(format!(
                "token {}",
                token
            ))))),
        }
    }

    /// Creates an API client authenticated by web flow.
    ///
    /// # Usage
    /// To authorize a web user, first redirect the user to
    /// `https://github.com/login/oauth/authorize`.
    /// When the user gets redirected back from GitHub,
    /// the request should contain a GET parameter `code`.
    /// Pass this parameter along with the app cient ID and secret to create the client.
    pub async fn create_from_oauth_web_flow(
        reqwest: reqwest::Client,
        client_id: impl Into<Cow<'static, str>>,
        client_secret: impl Into<Cow<'static, str>>,
        code: String,
    ) -> reqwest::Result<Self> {
        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let resp = reqwest
            .post("https://github.com/login/oauth/access_token")
            .header(header::ACCEPT, "application/json")
            .multipart(
                Form::new()
                    .text("client_id", client_id)
                    .text("client_secret", client_secret)
                    .text("code", code),
            )
            .send()
            .await?
            .error_for_status()?
            .json::<TokenResponse>()
            .await?;
        Ok(Self::create_from_token(reqwest, &resp.access_token))
    }

    /// Creates an API client authenticated as a GitHub app (**not** an installation)
    #[cfg(feature = "github-app")]
    #[cfg_attr(feature = "internal-docsrs", doc(cfg("github-app")))]
    pub async fn create_as_app(
        reqwest: reqwest::Client,
        app_id: &str,
        private_pem: &[u8],
    ) -> jsonwebtoken::errors::Result<Self> {
        use std::time::UNIX_EPOCH;

        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

        let private_key = EncodingKey::from_rsa_pem(private_pem)?;
        let iat = UNIX_EPOCH
            .elapsed()
            .expect("system clock was tuned before 1970 :(")
            .as_secs();
        // I don't have to write more code to support stupid users who play with system clocks right?
        // Alright I am already writing a lot of prose to explain why I don't write code.
        let exp = iat + 600;

        #[derive(serde::Serialize)]
        struct JwtPayload<'t> {
            iat: u64,
            exp: u64,
            iss: &'t str,
        }

        let jwt = encode(
            &Header::new(Algorithm::RS256),
            &JwtPayload {
                iat,
                exp,
                iss: app_id,
            },
            &private_key,
        )?;

        Ok(Self {
            reqwest,
            auth: Mutex::new(Auth(AuthImpl::Expiring(
                format!("Bearer {}", jwt).into(),
                UNIX_EPOCH + Duration::from_secs(exp),
            ))),
        })
    }

    pub async fn get_auth_header(&self) -> reqwest::Result<Option<Arc<str>>> {
        let mut lock = self.auth.lock().await;
        lock.get(&self.reqwest).await
    }
}

pub struct Auth(AuthImpl);

impl Auth {
    /// Obtains (and refreshes if possible and necessary) an Authorization header line.
    pub async fn get(&mut self, reqwest: &reqwest::Client) -> reqwest::Result<Option<Arc<str>>> {
        self.0.get(reqwest).await
    }
}
enum AuthImpl {
    None,
    Expired,
    Static(Arc<str>),
    Expiring(Arc<str>, SystemTime),
    Refreshing(RefreshingAuth),
}
impl AuthImpl {
    async fn get(&mut self, reqwest: &reqwest::Client) -> reqwest::Result<Option<Arc<str>>> {
        Ok(match self {
            Self::None | Self::Expired => None,
            Self::Static(token) => Some(Arc::clone(token)),
            Self::Expiring(token, expiry) => {
                if *expiry <= SystemTime::now() {
                    *self = Self::Expired;
                    return Ok(None);
                }
                Some(Arc::clone(token))
            }
            Self::Refreshing(ra) => {
                if ra.access_expires < Instant::now() {
                    #[derive(serde::Deserialize)]
                    struct RefreshResponse {
                        access_token: String,
                        expires_in: u64,
                        refresh_token: String,
                        refresh_token_expires_in: u64,
                    }

                    if ra.refresh_expires < Instant::now() + Duration::from_secs(5) {
                        // grace period of 5 seconds to minimize race conditions
                        *self = Self::Expired;
                        return Ok(None);
                    }

                    let RefreshResponse {
                        access_token,
                        expires_in,
                        refresh_token,
                        refresh_token_expires_in,
                    } = reqwest
                        .post("https://github.com/login/oauth/access_token")
                        .header(header::ACCEPT, "application/json")
                        .multipart(
                            Form::new()
                                .text("refresh_token", ra.refresh.to_owned())
                                .text("grant_type", "refresh_token")
                                .text("client_id", ra.app_id.to_owned())
                                .text("client_secret", ra.app_secret.to_owned()),
                        )
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<RefreshResponse>()
                        .await?;

                    ra.access = Arc::from(format!("token {}", access_token));
                    ra.access_expires = Instant::now() + Duration::from_secs(expires_in);
                    ra.refresh = refresh_token;
                    ra.refresh_expires =
                        Instant::now() + Duration::from_secs(refresh_token_expires_in);
                }
                Some(Arc::clone(&ra.access))
            }
        })
    }
}

pub struct RefreshingAuth {
    access: Arc<str>,
    access_expires: Instant,
    refresh: String,
    refresh_expires: Instant,
    app_id: String,
    app_secret: String,
}
