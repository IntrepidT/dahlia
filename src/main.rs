use leptos::prelude::*;
use leptos_config::LeptosOptions;
use leptos_meta::{MetaTags, Stylesheet, Title};
use leptos_router::*;

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix::Actor;
    use actix_files::Files;
    use actix_web::middleware::DefaultHeaders;
    use actix_web::{web, App, HttpServer};
    use dahlia::app::db::database;
    use dahlia::app::middleware::authentication::Authentication;
    use dahlia::app::routes::saml_routes::configure_saml_routes;
    use dahlia::app::websockets::lobby::Lobby;
    use dahlia::app::websockets::start_connection::start_connection;
    use leptos::prelude::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use std::env;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr.clone();

    println!("Site Root: {}", conf.leptos_options.site_root);
    println!("Site PKG Dir: {}", conf.leptos_options.site_pkg_dir);

    //Initialize the logger for reading log messages
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    //Create and make a database connection pool setup
    let pool_one = database::create_pool().await;
    println!("Database connection pool created successfully");
    let pool = web::Data::new(pool_one.clone());

    //Initialize the Chat server
    let chat_server = web::Data::new(Lobby::new(pool_one.clone()).start());

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(dahlia::app::App);
    println!("listening on http://{}", &addr);

    // Check for SAML configuration
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| format!("http://{}", &addr));
    println!("Base URL for SAML: {}", base_url);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        // We make the pool available to Leptos server functions
        let leptos_options_clone = leptos_options.clone();
        let chat_server_clone = chat_server.clone();

        App::new()
            // Make DB pool available to the app
            .app_data(pool.clone())
            //make chat server available to app
            .app_data(chat_server_clone.clone())
            // Authentication middleware
            .wrap(Authentication::new())
            // Configure SAML routes BEFORE other routes
            .configure(configure_saml_routes)
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Frame-Options", "DENY"))
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-XSS-Protection", "1; mode=block"))
                    .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
                    .add((
                        "Strict-Transport-Security",
                        "max-age=31536000; includeSubDomains; preload",
                    )),
            )
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", "./assets"))
            // serve CSS files with proper MIME type
            .service(Files::new("/style", "./style"))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .service(Files::new("/static", "./static").show_files_listing())
            .service(web::scope("/ws").service(start_connection))
            .service(web::scope("/api/ws").service(start_connection))
            .leptos_routes_with_context(
                routes.to_owned(),
                {
                    let options_clone = leptos_options_clone.clone();
                    move || {
                        provide_context(options_clone.clone());
                    }
                },
                {
                    let options_clone = leptos_options_clone.clone();
                    move || {
                        view! {
                            <!DOCTYPE html>
                            <html lang="en">
                                <head>
                                    <meta charset="utf-8"/>
                                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                    <title>"Teapot Testing"</title>
                                    <AutoReload options=options_clone.clone() />
                                    <HydrationScripts options=options_clone.clone()/>
                                    <link rel="stylesheet" href="/pkg/dahlia.css" />
                                    <link rel="stylesheet" href="/style/input.css" />
                                    <link rel="icon" href="/assets/favicon.ico" />
                                </head>
                                <body>
                                    {dahlia::app::App().into_view()}
                                </body>
                            </html>
                        }
                    }
                },
            )
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon")]
async fn favicon(
    leptos_options: actix_web::web::Data<LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/assets/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use dahlia::app::*;

    console_error_panic_hook::set_once();

    // FIXED: Use mount_to_body for CSR mode (different from hydration)
    leptos::mount::mount_to_body(App);
}
