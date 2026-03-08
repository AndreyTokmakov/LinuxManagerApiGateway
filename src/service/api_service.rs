
use actix_web::{App, web, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::*;
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[derive(OpenApi)]
#[openapi(
    paths(
        system_info,
        memory_info,
        disk_info,
        services_status,
        processes,
        interfaces
    ),
    components(
        schemas(
            SystemInfo,
            MemoryInfo,
            DiskInfo,
            ServiceStatus,
            ProcessInfo,
            NetworkInterface
        )
    )
)]
pub struct ApiDoc;

pub async fn run_server(host: &str, port: u16, runner: SshCommandRunner) -> std::io::Result<()>
{
    let runner_data = web::Data::new(runner);
    println!("Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(runner_data.clone())
            .service(system_info)
            .service(memory_info)
            .service(disk_info)
            .service(services_status)
            .service(processes)
            .service(interfaces)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi())
            )
    })
        .bind((host, port))?
        .run()
        .await
}