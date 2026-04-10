//! Kokoro Web — SSR server with Axum + Leptos.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::services::ServeDir;
    use kokoro_ui::app::App;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Static file serving: site_root contains pkg/ (WASM, JS, CSS) and fonts/
    let site_root = leptos_options.site_root.clone();

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || {
                use leptos::prelude::*;
                view! {
                    <!DOCTYPE html>
                    <html lang="en">
                        <head>
                            <meta charset="utf-8"/>
                            <meta name="viewport" content="width=device-width, initial-scale=1"/>
                            <link rel="stylesheet" href="/pkg/kokoro-ui.css"/>
                            <AutoReload options=leptos_options.clone()/>
                            <HydrationScripts options=leptos_options.clone()/>
                            <leptos_meta::MetaTags/>
                        </head>
                        <body>
                            <App/>
                        </body>
                    </html>
                }
            }
        })
        // Serve static files (CSS, JS, WASM, fonts) from the site root
        .fallback_service(ServeDir::new(site_root.to_string()))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Kokoro Web running on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {}
