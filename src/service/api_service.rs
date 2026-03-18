
use actix_web::{App, web, HttpServer};
use actix_web::web::Data;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::*;
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[derive(OpenApi)]
#[openapi(
    paths(
        exec_command,
        system_info,
        memory_info,
        disk_info,
        services_status,
        service_details,
        service_action,
        service_logs,
        process_list,
        interfaces,
        open_ports,
        routes,
        journal_logs,
        journal_errors,
        journal_service,
    ),
    components(
        schemas(
            SystemInfo,
            MemoryInfo,
            DiskInfo,
            ServiceStatus,
            ServiceAction,
            ProcessInfo,
            NetworkInterface,
            OpenPort,
            NetworkRoute,
            CommandRequest,
            CommandResponse,
            JournalEntry
        )
    )
)]
pub struct ApiDoc;

pub async fn run_server(host: &str, port: u16, runner: SshCommandRunner) -> std::io::Result<()>
{
    let ssh_command_runner: Data<SshCommandRunner> = web::Data::new(runner);
    println!("Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {  App::new()
        .app_data(ssh_command_runner.clone())
        .service(exec_command)
        .service(system_info)
        .service(memory_info)
        .service(disk_info)
        .service(services_status)
        .service(service_details)
        .service(service_action)
        .service(service_logs)
        .service(process_list)
        .service(interfaces)
        .service(open_ports)
        .service(routes)
        .service(journal_logs)
        .service(journal_errors)
        .service(journal_service)
        .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
    }).bind((host, port))?.run().await
}