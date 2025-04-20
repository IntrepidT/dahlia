#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix::Actor;
    use actix_files::Files;
    use actix_web::{web, App, HttpServer};
    use argon2::password_hash;
    use dahlia::app::db::database;
    use dahlia::app::middleware::authentication::Authentication;
    use dahlia::app::websockets::lobby::Lobby;
    use dahlia::app::websockets::start_connection::start_connection;
    use dahlia::app::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use std::env;

    let conf = get_configuration(None).await.unwrap();
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
    let chat_server = web::Data::new(Lobby::default().start());

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    //println!("Generated routes: {:?}", routes);
    println!("listening on http://{}", &addr);

    // Create a secret key for cookie encryption
    let secret_key = env::var("SECRET_KEY").unwrap_or_else(|_| {
        println!("WARNING: Using default secret key. Set the SECRET_KEY environment variable in production.");
        "this_is_a_default_key_and_should_be_changed_in_production".to_string()
    });

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        // We make the pool available to Leptos server functions
        let pool_clone = pool_one.clone();
        let leptos_options_clone = leptos_options.clone();
        let chat_server_clone = chat_server.clone();

        App::new()
            // Make DB pool available to the app
            .app_data(pool.clone())
            //make chat server available to app
            .app_data(chat_server_clone.clone())
            // Authentication middleware
            .wrap(Authentication::new(secret_key.clone()))
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .service(Files::new("/static", "./static").show_files_listing())
            .service(web::scope("/ws").service(start_connection))
            // Leptos routes (this must be last)
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(leptos_options_clone.to_owned(), routes.to_owned(), App)
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
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

    leptos::mount_to_body(App);
}
