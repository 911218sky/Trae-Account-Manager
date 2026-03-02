use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::{oneshot, Mutex};
use warp::Filter;

use crate::account::AccountManager;

pub async fn start_login_flow(
    app: AppHandle,
    state: Arc<Mutex<AccountManager>>,
) -> Result<(), String> {
    // If a login window already exists, focus it
    if let Some(win) = app.get_webview_window("trae-login") {
        let _ = win.set_focus();
        return Ok(());
    }

    // Create oneshot channel to notify warp server to stop
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));

    let app_clone = app.clone();
    let state_clone = state.clone();

    // POST /callback — Receive token and cookies
    let callback = warp::post()
        .and(warp::path("callback"))
        .and(warp::body::json())
        .and_then(move |body: serde_json::Value| {
            let app = app_clone.clone();
            let state = state_clone.clone();
            async move {
                let token = body["token"].as_str().unwrap_or("");
                if token.is_empty() {
                    return Ok::<_, warp::Rejection>(warp::reply::json(
                        &serde_json::json!({"status": "waiting"}),
                    ));
                }

                // Extract cookies (if present)
                let cookies = body["cookies"].as_str().map(|s| s.to_string());

                let mut manager = state.lock().await;
                match manager.add_account_by_token(token.to_string(), cookies).await {
                    Ok(account) => {
                        let _ = app.emit("login-success", &account.email);
                        // Delay closing the window to allow warp to return the response
                        let app2 = app.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                            if let Some(win) = app2.get_webview_window("trae-login") {
                                let _ = win.close();
                            }
                        });
                        Ok(warp::reply::json(&serde_json::json!({"status": "ok"})))
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("已存在") {
                            let _ = app.emit("login-failed", "该账号已存在");
                            let app2 = app.clone();
                            tokio::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                                if let Some(win) = app2.get_webview_window("trae-login") {
                                    let _ = win.close();
                                }
                            });
                        }
                        Ok(warp::reply::json(
                            &serde_json::json!({"status": "error", "message": msg}),
                        ))
                    }
                }
            }
        });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["content-type"]);

    let routes = callback.with(cors);

    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
            let _ = shutdown_rx.await;
        });
    let port = addr.port();

    tokio::spawn(server);

    // Inject JavaScript to hook fetch/XHR and intercept trae.ai frontend's GetUserToken request responses
    // Note: document.cookie cannot access HttpOnly cookies, so only the token is sent here
    // Complete cookies need to be obtained on the Rust side via webview API
    let init_script = format!(
        r#"
        (function() {{
            var __sent = false;
            var __callbackUrl = "http://127.0.0.1:{port}/callback";

            function sendToken(token) {{
                if (__sent || !token || token.length < 50) return;
                __sent = true;

                // Note: document.cookie can only access non-HttpOnly cookies
                // Most authentication cookies (like sessionid, sid_guard, etc.) are HttpOnly and cannot be accessed via JS
                var cookies = document.cookie;

                console.log("[Trae Auto] Captured token, length:", token.length);
                console.log("[Trae Auto] document.cookie length:", cookies.length);
                console.log("[Trae Auto] Note: HttpOnly cookies cannot be accessed via JS");

                var xhr = new XMLHttpRequest();
                xhr.open("POST", __callbackUrl, true);
                xhr.setRequestHeader("Content-Type", "application/json");
                xhr.send(JSON.stringify({{
                    token: token,
                    cookies: cookies || ""
                }}));
            }}

            function tryExtractToken(text) {{
                try {{
                    var data = typeof text === "string" ? JSON.parse(text) : text;
                    if (data && data.Result && data.Result.Token) {{
                        return data.Result.Token;
                    }}
                }} catch(e) {{}}
                return null;
            }}

            // Hook fetch
            var origFetch = window.fetch;
            window.fetch = function() {{
                var url = arguments[0];
                if (typeof url === "object" && url.url) url = url.url;
                var p = origFetch.apply(this, arguments);
                if (typeof url === "string" && url.indexOf("GetUserToken") !== -1) {{
                    p.then(function(resp) {{
                        return resp.clone().text();
                    }}).then(function(text) {{
                        var token = tryExtractToken(text);
                        if (token) sendToken(token);
                    }}).catch(function() {{}});
                }}
                return p;
            }};

            // Hook XMLHttpRequest
            var origOpen = XMLHttpRequest.prototype.open;
            var origSend = XMLHttpRequest.prototype.send;
            XMLHttpRequest.prototype.open = function(method, url) {{
                this.__url = url;
                return origOpen.apply(this, arguments);
            }};
            XMLHttpRequest.prototype.send = function() {{
                var self = this;
                if (self.__url && self.__url.indexOf("GetUserToken") !== -1) {{
                    self.addEventListener("load", function() {{
                        var token = tryExtractToken(self.responseText);
                        if (token) sendToken(token);
                    }});
                }}
                return origSend.apply(this, arguments);
            }};
        }})();
    "#,
        port = port
    );

    // Do not use incognito mode to allow access to all cookies
    let window = WebviewWindowBuilder::new(
        &app,
        "trae-login",
        WebviewUrl::External("https://www.trae.ai".parse().unwrap()),
    )
    .title("登录 Trae 账号")
    .inner_size(500.0, 700.0)
    .center()
    .incognito(false)  // Changed to false to allow access to complete cookies
    .initialization_script(&init_script)
    .build()
    .map_err(|e| e.to_string())?;

    // Listen for window close event, stop warp server and notify frontend
    let shutdown_on_close = shutdown_tx.clone();
    let app_for_close = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            let shutdown = shutdown_on_close.clone();
            let app = app_for_close.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(tx) = shutdown.lock().await.take() {
                    // If shutdown is still present, the window was closed by the user, not by successful login
                    let _ = app.emit("login-cancelled", ());
                    let _ = tx.send(());
                }
            });
        }
    });

    Ok(())
}
