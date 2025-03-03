#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::{web, App, HttpServer};
    use dahlia::app::db::database;
    use dahlia::app::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use std::env;

    let conf = get_configuration(None).await.unwrap();
    let addr = "0.0.0.0:3000".to_string();

    //Initialize the logger for reading log messages
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    //Create and make a database connection pool setup
    let pool_one = database::create_pool().await;
    println!("Database connection pool created successfully");
    let pool = web::Data::new(pool_one);
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    //    println!("listening on http://{}", &addr);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        //wire up the database pool
        App::new()
            //I am putting these first to see if they have any effect on the order
            .app_data(pool.clone())
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .service(Files::new("/static", "./static").show_files_listing())
            //.app_data(web::Data::new(pool.clone()))
            //.leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
            .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)

        //.wrap(middleware::Compress::default())
    })
    .bind("0.0.0.0:3000")?
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
